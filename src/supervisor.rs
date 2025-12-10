use std::sync::{Arc, atomic::AtomicBool};

use tokio::{
    sync::mpsc::{Sender, channel},
    task::JoinHandle,
};
use tokio_util::sync::CancellationToken;

use crate::internal::{ActorHandler, Broker, Subscriber};
use crate::{Actor, Config, Context, Envelope, Event, Result, Topic};

pub struct Supervisor<E: Event, T: Topic<E>> {
    config: Config,
    broker: Broker<E, T>,
    sender: Sender<Envelope<E>>,
    handles: Vec<JoinHandle<Result<()>>>,
    cancel_token: Arc<CancellationToken>,
}

impl<E: Event + 'static, T: Topic<E>> Supervisor<E, T> {
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

    pub async fn start(&mut self) -> Result<()> {
        self.broker.run().await?;
        Ok(())
    }

    pub async fn send(&self, event: E) -> Result<()> {
        self.sender.send(Envelope::new(event, "supervisor")).await?;
        Ok(())
    }

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
