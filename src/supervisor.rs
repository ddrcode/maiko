use std::sync::{Arc, atomic::AtomicBool};

use tokio::{
    sync::mpsc::{Sender, channel},
    task::JoinHandle,
};
use tokio_util::sync::CancellationToken;

use crate::internal::{ActorHandler, Broker, Subscriber};
use crate::{Actor, Config, Context, Envelope, Event, Result, Topic};

/// Coordinates actors and the broker, and owns the top-level runtime.
///
/// - Register actors with `add_actor(name, |ctx| Actor, topics)`.
/// - Start the runtime with `start()`, which blocks until cancelled via `stop()`.
/// - Emit events into the broker with `send(event)`.
pub struct Supervisor<E: Event, T: Topic<E>> {
    config: Config,
    broker: Broker<E, T>,
    sender: Sender<Envelope<E>>,
    handles: Vec<JoinHandle<Result<()>>>,
    cancel_token: Arc<CancellationToken>,
}

impl<E: Event + 'static, T: Topic<E>> Supervisor<E, T> {
    /// Create a new supervisor with the given runtime configuration.
    pub fn new(config: Config) -> Self {
        let (tx, rx) = channel::<Envelope<E>>(config.channel_size);
        let cancel_token = Arc::new(CancellationToken::new());
        Self {
            broker: Broker::new(rx, cancel_token.clone()),
            config,
            sender: tx,
            handles: Vec::new(),
            cancel_token,
        }
    }

    /// Register a new actor with a factory that receives a `Context<E>`.
    ///
    /// The `name` is used for metadata and (by default) to avoid self-routing.
    /// `topics` declare which event topics the actor subscribes to.
    pub fn add_actor<A, F>(&mut self, name: &str, factory: F, topics: Vec<T>) -> Result<()>
    where
        A: Actor<Event = E> + 'static,
        F: FnOnce(Context<E>) -> A,
    {
        let name: Arc<str> = Arc::from(name);
        let alive = Arc::new(AtomicBool::new(true));
        let (tx, rx) = tokio::sync::mpsc::channel::<Envelope<E>>(self.config.channel_size);
        let ctx = Context::<E> {
            name: name.clone(),
            sender: self.sender.clone(),
            alive: alive.clone(),
        };
        let actor = factory(ctx.clone());

        let subscriber = Subscriber::<E, T>::new(name.clone(), topics, tx);
        self.broker.add_subscriber(subscriber);

        let mut handler = ActorHandler {
            actor,
            receiver: rx,
            cancel_token: self.cancel_token.clone(),
            ctx,
        };

        let join_handle = tokio::spawn(async move { handler.run().await });
        self.handles.push(join_handle);

        Ok(())
    }

    /// Run the broker loop. This method blocks until `stop()` is called
    /// (or the cancellation token is triggered), then waits for all actors
    /// to finish.
    pub async fn start(&mut self) -> Result<()> {
        self.broker.run().await?;
        Ok(())
    }

    /// Emit an event into the broker from the supervisor.
    pub async fn send(&self, event: E) -> Result<()> {
        self.sender.send(Envelope::new(event, "supervisor")).await?;
        Ok(())
    }

    /// Request a graceful shutdown, then await all actor tasks.
    pub async fn stop(&mut self) -> Result<()> {
        self.cancel_token.cancel();
        while let Some(handle) = self.handles.pop() {
            handle.await??;
        }
        Ok(())
    }
}

impl<E: Event + 'static, T: Topic<E>> Default for Supervisor<E, T> {
    fn default() -> Self {
        Self::new(Config::default())
    }
}
