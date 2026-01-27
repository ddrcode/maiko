use std::sync::{Arc, atomic::AtomicBool};

use tokio::{
    sync::{
        Mutex, Notify,
        mpsc::{Sender, channel},
    },
    task::JoinSet,
};
use tokio_util::sync::CancellationToken;

use crate::{
    Actor, ActorId, Config, Context, DefaultTopic, Envelope, Error, Event, Result, Subscribe,
    Topic,
    internal::{ActorController, Broker, Subscriber, Subscription},
};

#[cfg(feature = "monitoring")]
use crate::monitoring::MonitorRegistry;

/// Coordinates actors and the broker, and owns the top-level runtime.
///
/// # Actor Registration
///
/// ```ignore
/// // Subscribe to specific topics
/// supervisor.add_actor("processor", |ctx| Processor::new(ctx), &[MyTopic::Data])?;
///
/// // Subscribe to all topics (e.g., monitoring)
/// supervisor.add_actor("monitor", |ctx| Monitor::new(ctx), Subscribe::all())?;
///
/// // Subscribe to no topics (pure event producer)
/// supervisor.add_actor("producer", |ctx| Producer::new(ctx), Subscribe::none())?;
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
    pub(crate) sender: Sender<Arc<Envelope<E>>>,
    tasks: JoinSet<Result<()>>,
    cancel_token: Arc<CancellationToken>,
    broker_cancel_token: Arc<CancellationToken>,
    start_notifier: Arc<Notify>,
    supervisor_id: ActorId,

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
            let mut monitoring = MonitorRegistry::new(&config);
            monitoring.start();
            monitoring
        };

        let broker_cancel_token = Arc::new(CancellationToken::new());
        let broker = Broker::new(
            rx,
            broker_cancel_token.clone(),
            config.clone(),
            #[cfg(feature = "monitoring")]
            monitoring.sink(),
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

            #[cfg(feature = "monitoring")]
            monitoring,
        }
    }

    /// Register a new actor with a factory that receives a [`Context<E>`].
    ///
    /// This is the primary way to register actors with the supervisor.
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
    pub fn add_actor<A, F, S>(&mut self, name: &str, factory: F, topics: S) -> Result<ActorId>
    where
        A: Actor<Event = E>,
        F: FnOnce(Context<E>) -> A,
        S: Into<Subscribe<E, T>>,
    {
        let ctx = self.create_context(name);
        let actor = factory(ctx.clone());
        let topics = topics.into().0;
        self.register_actor(ctx, actor, topics)
    }

    /// Internal method to register an actor with the supervisor.
    ///
    /// Called by `add_actor()` to perform the actual registration. It:
    /// 1. Creates a Subscriber and registers it with the broker
    /// 2. Creates an ActorHandler wrapping the actor
    /// 3. Spawns the actor task (which waits for start notification)
    pub(crate) fn register_actor<A>(
        &mut self,
        ctx: Context<E>,
        actor: A,
        topics: Subscription<T>,
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
            monitoring: self.monitoring.sink(),

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
    /// Internal helper used by `add_actor` to create actor contexts.
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
