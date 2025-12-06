use std::sync::Arc;

use tokio::{
    sync::{
        Mutex,
        mpsc::{Sender, channel},
    },
    task::JoinHandle,
};

use crate::{Actor, Broker, Config, Context, Envelope, Event, Result, Subscriber, Topic};

pub struct Supervisor<E: Event, T: Topic<E>> {
    config: Config,
    broker: Broker<E, T>,
    sender: Sender<Envelope<E>>,
    actors: Vec<Arc<Mutex<Box<dyn Actor<Event = E>>>>>,
    handles: Vec<JoinHandle<Result<()>>>,
}

impl<E: Event + 'static, T: Topic<E>> Supervisor<E, T> {
    pub fn new(config: Config) -> Self {
        let (tx, rx) = channel::<Envelope<E>>(config.channel_size);
        Self {
            broker: Broker::new(rx),
            config,
            sender: tx,
            actors: Vec::new(),
            handles: Vec::new(),
        }
    }

    pub fn add_actor<A: Actor<Event = E> + 'static>(
        &mut self,
        mut actor: A,
        topics: Vec<T>,
    ) -> Result<()> {
        let (tx, rx) = tokio::sync::mpsc::channel::<Envelope<E>>(self.config.channel_size);
        actor.set_ctx(Context::<E> {
            sender: tx.clone(),
            receiver: rx,
        })?;
        let subscriber = Subscriber::<E, T>::new(Arc::from(actor.name()), topics, tx);
        self.broker.add_subscriber(subscriber);
        self.actors.push(Arc::new(Mutex::new(Box::new(actor))));
        Ok(())
    }

    pub async fn start(&mut self) -> Result<()> {
        let handles = self
            .actors
            .iter()
            .cloned()
            .map(|actor| {
                let h = tokio::spawn(async move { actor.lock().await.run().await });
                h
            })
            .collect::<Vec<_>>();
        self.handles = handles;
        self.broker.run().await?;
        Ok(())
    }

    pub async fn send(&self, event: E) -> Result<()> {
        self.sender.send(Envelope::new(event)).await?;
        Ok(())
    }
}

impl<E: Event + 'static, T: Topic<E>> Default for Supervisor<E, T> {
    fn default() -> Self {
        Self::new(Config::default())
    }
}
