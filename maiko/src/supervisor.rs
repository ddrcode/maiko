use std::{
    collections::HashSet,
    sync::{Arc, atomic::AtomicBool},
};

use tokio::{
    sync::{
        Mutex, Notify,
        mpsc::{Sender, channel},
    },
    task::JoinSet,
};
use tokio_util::sync::CancellationToken;

use crate::{
    Actor, ActorBuilder, ActorId, Config, Context, DefaultTopic, Envelope, Error, Event, Result,
    Topic,
    internal::{ActorController, Broker, Subscriber},
};

#[cfg(feature = "monitoring")]
use crate::monitoring::MonitorRegistry;

/// Coordinates actors and the broker, and owns the top-level runtime.
///
/// # Actor Registration
///
/// Two ways to register actors:
///
/// **Simple API** (for common cases):
/// ```ignore
/// supervisor.add_actor("my-actor", |ctx| MyActor::new(ctx), &[MyTopic::Data])?;
/// ```
///
/// **Builder API** (for advanced configuration):
/// ```ignore
/// supervisor.build_actor::<MyActor>("my-actor")
///     .actor(|ctx| MyActor::new(ctx))
///     .topics(&[MyTopic::Data])
///     .build()?;
/// ```
///
/// # Runtime Control
///
/// - [`start()`](Self::start) spawns the broker loop and returns immediately (non-blocking).
/// - [`join()`](Self::join) awaits all actor tasks to finish; typically used after `start()`.
/// - [`run()`](Self::run) combines `start()` and `join()`, blocking until shutdown.
/// - [`stop()`](Self::stop) graceful shutdown; lets actors consume active events
/// - [`send(event)`](Self::send) emits events into the broker.
///
/// See also: [`Actor`], [`Context`], [`Topic`].
pub struct Supervisor<E: Event, T: Topic<E> = DefaultTopic> {
    config: Arc<Config>,
    broker: Arc<Mutex<Broker<E, T>>>,
    sender: Sender<Arc<Envelope<E>>>,
    tasks: JoinSet<Result<()>>,
    cancel_token: Arc<CancellationToken>,
    broker_cancel_token: Arc<CancellationToken>,
    start_notifier: Arc<Notify>,
    supervisor_id: ActorId,

    #[cfg(feature = "test-harness")]
    harness: Option<crate::testing::Harness<E, T>>,

    #[cfg(feature = "monitoring")]
    monitoring: MonitorRegistry<E, T>,
}

impl<E: Event, T: Topic<E>> Supervisor<E, T> {
    /// Create a new supervisor with the given runtime configuration.
    pub fn new(config: Config) -> Self {
        let config = Arc::new(config);
        let (tx, rx) = channel::<Arc<Envelope<E>>>(config.channel_size);
        let cancel_token = Arc::new(CancellationToken::new());

        #[cfg(feature = "monitoring")]
        let monitoring = {
            let mut monitoring = MonitorRegistry::new();
            monitoring.start();
            monitoring
        };

        let broker_cancel_token = Arc::new(CancellationToken::new());
        let broker = Broker::new(
            rx,
            broker_cancel_token.clone(),
            config.clone(),
            #[cfg(feature = "monitoring")]
            monitoring.provider(),
        );

        Self {
            broker: Arc::new(Mutex::new(broker)),
            config,
            sender: tx,
            tasks: JoinSet::new(),
            cancel_token,
            broker_cancel_token,
            start_notifier: Arc::new(Notify::new()),
            supervisor_id: ActorId::new(Arc::from("supervisor")),

            #[cfg(feature = "test-harness")]
            harness: None,

            #[cfg(feature = "monitoring")]
            monitoring,
        }
    }

    /// Register a new actor with a factory that receives a [`Context<E>`].
    ///
    /// This is a convenience method that wraps the builder API. For advanced
    /// configuration, use [`build_actor()`](Self::build_actor) instead.
    ///
    /// # Arguments
    ///
    /// * `name` - Actor identifier used for metadata and routing
    /// * `factory` - Closure that receives a Context and returns the actor
    /// * `topics` - Slice of topics the actor subscribes to
    ///
    /// # Example
    ///
    /// ```ignore
    /// supervisor.add_actor(
    ///     "processor",
    ///     |ctx| DataProcessor::new(ctx),
    ///     &[MyTopic::Data, MyTopic::Control]
    /// )?;
    /// ```
    pub fn add_actor<A, F>(&mut self, name: &str, factory: F, topics: &[T]) -> Result<ActorId>
    where
        A: Actor<Event = E>,
        F: FnOnce(Context<E>) -> A,
    {
        self.build_actor(name).actor(factory).topics(topics).build()
    }

    /// Begin building an actor registration with fine-grained control.
    ///
    /// Returns an [`ActorBuilder`] that allows setting topics and (future)
    /// configuration separately. For simple cases, prefer [`add_actor()`](Self::add_actor).
    ///
    /// # Example
    ///
    /// ```ignore
    /// supervisor.build_actor::<MyActor>("processor")
    ///     .actor(|ctx| MyActor::new(ctx))
    ///     .topics(&[MyTopic::Data])
    ///     .build()?;
    /// ```
    pub fn build_actor<A>(&mut self, name: &str) -> ActorBuilder<'_, E, T, A>
    where
        A: Actor<Event = E>,
    {
        let ctx = self.create_context(name);
        ActorBuilder::new(self, ctx)
    }

    /// Internal method to register an actor with the supervisor.
    ///
    /// This is called by both `add_actor()` and `ActorBuilder.build()` to perform
    /// the actual registration. It:
    /// 1. Creates a Subscriber and registers it with the broker
    /// 2. Creates an ActorHandler wrapping the actor
    /// 3. Spawns the actor task (which waits for start notification)
    pub(crate) fn register_actor<A>(
        &mut self,
        ctx: Context<E>,
        actor: A,
        topics: HashSet<T>,
    ) -> Result<ActorId>
    where
        A: Actor<Event = E>,
    {
        let actor_id = ctx.actor_id().clone();

        let mut broker = self
            .broker
            .try_lock()
            .map_err(|_| Error::BrokerAlreadyStarted)?;

        let (tx, rx) = tokio::sync::mpsc::channel::<Arc<Envelope<E>>>(self.config.channel_size);

        let subscriber = Subscriber::<E, T>::new(actor_id.clone(), topics, tx);
        broker.add_subscriber(subscriber)?;

        let mut controller = ActorController::<A, T> {
            actor,
            receiver: rx,
            ctx,
            max_events_per_tick: self.config.max_events_per_tick,
            cancel_token: self.cancel_token.clone(),

            #[cfg(feature = "monitoring")]
            monitor: self.monitoring.provider(),

            _topic: std::marker::PhantomData,
        };

        let notified = self.start_notifier.clone().notified_owned();
        self.tasks.spawn(async move {
            notified.await;
            controller.run().await
        });

        Ok(actor_id)
    }

    /// Create a new Context for an actor.
    ///
    /// Internal helper used by `add_actor` and `ActorBuilder` to create actor contexts.
    pub(crate) fn create_context(&self, name: &str) -> Context<E> {
        Context::<E> {
            actor_id: ActorId::new(Arc::<str>::from(name)),
            sender: self.sender.clone(),
            alive: Arc::new(AtomicBool::new(true)),
        }
    }

    /// Start the broker loop in a background task. This returns immediately.
    pub async fn start(&mut self) -> Result<()> {
        let broker = self.broker.clone();
        self.tasks
            .spawn(async move { broker.lock().await.run().await });
        self.start_notifier.notify_waiters();
        Ok(())
    }

    /// Waits until at least one of the actor tasks completes then
    /// triggers a shutdown if not already requested.
    pub async fn join(&mut self) -> Result<()> {
        while let Some(res) = self.tasks.join_next().await {
            if !self.cancel_token.is_cancelled() {
                self.stop().await?;
                break;
            }
            res??;
        }
        Ok(())
    }

    /// Convenience method to start and then await completion of all tasks.
    /// Blocks until shutdown.
    pub async fn run(&mut self) -> Result<()> {
        self.start().await?;
        self.join().await
    }

    /// Emit an event into the broker from the supervisor.
    pub async fn send(&self, event: E) -> Result<()> {
        self.sender
            .send(Envelope::new(event, self.supervisor_id.clone()).into())
            .await?;
        Ok(())
    }

    /// Request a graceful shutdown, then await all actor tasks.
    ///
    /// # Shutdown Process
    ///
    /// 1. Waits for the broker to receive all pending events (up to 10 ms)
    /// 2. Stops the broker and waits for it to drain actor queues
    /// 3. Cancels all actors and waits for tasks t
    pub async fn stop(&mut self) -> Result<()> {
        use tokio::time::*;
        let start = Instant::now();
        let timeout = Duration::from_millis(10);
        let max = self.sender.max_capacity();

        #[cfg(feature = "test-harness")]
        if let Some(harness) = self.harness.as_ref() {
            harness.exit().await;
        }

        #[cfg(feature = "monitoring")]
        self.monitoring.stop().await;

        // 1. Wait for the main channle to drain
        while start.elapsed() < timeout {
            if self.sender.capacity() == max {
                break;
            }
            sleep(Duration::from_micros(100)).await;
        }

        // 2. Wait the the broker to shutdown gracefully
        self.broker_cancel_token.cancel();
        let _ = self.broker.lock().await;

        // 3. Stop the actors
        self.cancel_token.cancel();
        while let Some(res) = self.tasks.join_next().await {
            res??;
        }
        Ok(())
    }

    pub fn config(&self) -> &Config {
        self.config.as_ref()
    }

    /// Initialize the test harness for observing event flow.
    ///
    /// This should be called before [`start()`](Self::start). The harness enables
    /// event recording, injection, and assertions for testing.
    ///
    /// If called multiple times, returns a clone of the existing harness.
    ///
    /// # Example
    ///
    /// ```ignore
    /// let mut sup = Supervisor::<MyEvent>::default();
    /// sup.add_actor("test", |ctx| MyActor::new(ctx), &[DefaultTopic])?;
    ///
    /// let mut test = sup.init_test_harness().await;
    /// sup.start().await?;
    ///
    /// test.start_recording().await;
    /// // ... run test ...
    /// test.stop_recording().await;
    /// ```
    #[cfg(feature = "test-harness")]
    pub async fn init_test_harness(&mut self) -> crate::testing::Harness<E, T> {
        // Return existing harness if already initialized
        if let Some(ref harness) = self.harness {
            return harness.clone();
        }

        let (harness, mut collector, recording) =
            crate::testing::init_harness::<E, T>(self.sender.clone());
        self.tasks.spawn(async move { collector.run().await });
        self.broker
            .lock()
            .await
            .set_test_sender(harness.test_sender.clone(), recording);
        self.harness = Some(harness.clone());
        harness
    }

    #[cfg(feature = "monitoring")]
    pub fn monitors(&mut self) -> &mut MonitorRegistry<E, T> {
        &mut self.monitoring
    }
}

impl<E: Event, T: Topic<E>> Default for Supervisor<E, T> {
    fn default() -> Self {
        Self::new(Config::default())
    }
}

impl<E: Event, T: Topic<E>> Drop for Supervisor<E, T> {
    fn drop(&mut self) {
        if !self.cancel_token.is_cancelled() {
            self.cancel_token.cancel();
        }
        if !self.broker_cancel_token.is_cancelled() {
            self.broker_cancel_token.cancel();
        }
        #[cfg(feature = "monitoring")]
        self.monitoring.cancel();
    }
}
