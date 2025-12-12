use std::sync::{Arc, atomic::AtomicBool};

use tokio::{
    sync::{
        Mutex,
        mpsc::{Sender, channel},
    },
    task::JoinSet,
};
use tokio_util::sync::CancellationToken;

use crate::{Actor, Config, Context, Envelope, Event, Result, Topic};
use crate::{
    DefaultTopic,
    internal::{ActorHandler, Broker, Subscriber},
};

/// Coordinates actors and the broker, and owns the top-level runtime.
///
/// - Register actors with `add_actor(name, |ctx| Actor, topics)`.
/// - Start the runtime with `start()`, which blocks until cancelled via `stop()`.
/// - Emit events into the broker with `send(event)`.
pub struct Supervisor<E: Event, T: Topic<E> = DefaultTopic> {
    config: Config,
    broker: Arc<Mutex<Broker<E, T>>>,
    sender: Sender<Envelope<E>>,
    tasks: JoinSet<Result<()>>,
    cancel_token: Arc<CancellationToken>,
}

impl<E: Event + Sync + 'static, T: Topic<E> + Send + Sync + 'static> Supervisor<E, T> {
    /// Create a new supervisor with the given runtime configuration.
    pub fn new(config: Config) -> Self {
        let (tx, rx) = channel::<Envelope<E>>(config.channel_size);
        let cancel_token = Arc::new(CancellationToken::new());
        Self {
            broker: Arc::new(Mutex::new(Broker::new(rx, cancel_token.clone()))),
            config,
            sender: tx,
            tasks: JoinSet::new(),
            cancel_token,
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
        let mut broker = self.broker.try_lock().unwrap();
        let name: Arc<str> = Arc::from(name);
        let alive = Arc::new(AtomicBool::new(true));
        let (tx, rx) = tokio::sync::mpsc::channel::<Envelope<E>>(self.config.channel_size);
        let ctx = Context::<E> {
            name: name.clone(),
            sender: self.sender.clone(),
            alive: alive.clone(),
            cancel_token: self.cancel_token.clone(),
        };
        let actor = factory(ctx.clone());

        let subscriber = Subscriber::<E, T>::new(name.clone(), topics, tx);
        broker.add_subscriber(subscriber)?;

        let mut handler = ActorHandler {
            actor,
            receiver: rx,
            ctx,
            drain_limit: self.config.drain_limit,
        };

        self.tasks.spawn(async move { handler.run().await });

        Ok(())
    }

    /// Run the broker loop. This method blocks until `stop()` is called
    /// (or the cancellation token is triggered), then waits for all actors
    /// to finish.
    pub async fn start(&mut self) -> Result<()> {
        let broker = self.broker.clone();
        // let mut broker = self.broker.take().unwrap();
        // self.tasks
        tokio::task::spawn(async move { broker.lock().await.run().await });
        Ok(())
    }

    pub async fn join(&mut self) -> Result<()> {
        while let Some(res) = self.tasks.join_next().await {
            res??;
        }
        Ok(())
    }

    pub async fn run(&mut self) -> Result<()> {
        self.start().await?;
        self.join().await
    }

    /// Emit an event into the broker from the supervisor.
    pub async fn send(&self, event: E) -> Result<()> {
        self.sender.send(Envelope::new(event, "supervisor")).await?;
        Ok(())
    }

    /// Request a graceful shutdown, then await all actor tasks.
    pub async fn stop(&mut self) -> Result<()> {
        self.cancel_token.cancel();
        self.join().await?;
        Ok(())
    }
}

impl<E: Event + Sync + 'static, T: Topic<E> + Send + Sync + 'static> Default for Supervisor<E, T> {
    fn default() -> Self {
        Self::new(Config::default())
    }
}
