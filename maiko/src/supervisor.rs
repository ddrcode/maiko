use std::{collections::HashSet, sync::Arc};

use tokio::{
    sync::{
        Mutex, Notify,
        mpsc::{Sender, channel},
    },
    task::JoinSet,
};
use tokio_util::sync::CancellationToken;

use crate::{Actor, ActorBuilder, Config, Context, Envelope, Error, Event, Result, Topic};
use crate::{
    DefaultTopic,
    internal::{ActorHandler, Broker, Subscriber},
};

/// Coordinates actors and the broker, and owns the top-level runtime.
///
/// - Register actors with `add_actor(name, |ctx| Actor, topics)`.
/// - `start()` spawns the broker loop and returns immediately (non-blocking).
/// - `join()` awaits all actor tasks to finish; typically used after `start()`.
/// - `run()` combines `start()` and `join()`, blocking until shutdown.
/// - `stop()` graceful shutdown; lets actors to consumed active events
/// - Emit events into the broker with `send(event)`.
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
}

impl<E: Event + Sync + 'static, T: Topic<E> + Send + Sync + 'static> Supervisor<E, T> {
    /// Create a new supervisor with the given runtime configuration.
    pub fn new(config: Config) -> Self {
        let config = Arc::new(config);
        let (tx, rx) = channel::<Arc<Envelope<E>>>(config.channel_size);
        let cancel_token = Arc::new(CancellationToken::new());
        let broker_cancel_token = Arc::new(CancellationToken::new());
        let broker = Broker::new(rx, broker_cancel_token.clone(), config.clone());
        Self {
            broker: Arc::new(Mutex::new(broker)),
            config,
            sender: tx,
            tasks: JoinSet::new(),
            cancel_token,
            broker_cancel_token,
            start_notifier: Arc::new(Notify::new()),
        }
    }

    /// Register a new actor with a factory that receives a `Context<E>`.
    ///
    /// The `name` is used for metadata and (by default) to avoid self-routing.
    /// `topics` declare which event topics the actor subscribes to.
    pub fn add_actor<A, F>(&mut self, name: &str, factory: F, topics: &[T]) -> Result<()>
    where
        A: Actor<Event = E> + 'static,
        F: FnOnce(Context<E>) -> A,
    {
        self.build_actor(name).actor(factory).topics(topics).build()
    }

    pub fn build_actor<A>(&mut self, name: &str) -> ActorBuilder<'_, E, T, A>
    where
        A: Actor<Event = E> + 'static,
    {
        ActorBuilder::new(self, name)
    }

    pub(crate) fn register_actor<A>(
        &mut self,
        ctx: Context<E>,
        actor: A,
        topics: HashSet<T>,
    ) -> Result<()>
    where
        A: Actor<Event = E> + 'static,
    {
        let mut broker = self
            .broker
            .try_lock()
            .map_err(|_| Error::BrokerAlreadyStarted)?;

        let (tx, rx) = tokio::sync::mpsc::channel::<Arc<Envelope<E>>>(self.config.channel_size);

        let subscriber = Subscriber::<E, T>::new(ctx.name.clone(), topics, tx);
        broker.add_subscriber(subscriber)?;

        let mut handler = ActorHandler {
            actor,
            receiver: rx,
            ctx,
            max_events_per_tick: self.config.max_events_per_tick,
            cancel_token: self.cancel_token.clone(),
        };

        let notified = self.start_notifier.clone().notified_owned();
        self.tasks.spawn(async move {
            notified.await;
            handler.run().await
        });

        Ok(())
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
            .send(Envelope::new(event, "supervisor").into())
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
}

impl<E: Event + Sync + 'static, T: Topic<E> + Send + Sync + 'static> Default for Supervisor<E, T> {
    fn default() -> Self {
        Self::new(Config::default())
    }
}
