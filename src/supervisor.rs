use std::sync::Arc;

use tokio::{
    sync::{
        Mutex,
        mpsc::{Receiver, Sender, channel},
    },
    task::{JoinError, JoinHandle},
};

use crate::{Actor, Broker, Context, Envelope, Event, Result, Subscriber, Topic, subscriber};

pub struct Supervisor<E: Event, T: Topic<E>> {
    channel_size: usize,
    broker: Broker<E, T>,
    _sender: Sender<Envelope<E>>,
    actors: Vec<Arc<Mutex<Box<dyn Actor<Event = E>>>>>,
    handles: Vec<JoinHandle<Result<()>>>,
}

impl<E: Event + 'static, T: Topic<E>> Supervisor<E, T> {
    pub fn new(channel_size: usize) -> Self {
        let (tx, rx) = channel::<Envelope<E>>(channel_size);
        Self {
            channel_size,
            broker: Broker::new(channel_size, rx),
            _sender: tx,
            actors: Vec::new(),
            handles: Vec::new(),
        }
    }

    pub fn add_actor<A: Actor<Event = E>>(&mut self, mut actor: A, topics: Vec<T>) -> Result<()> {
        let (tx, rx) = tokio::sync::mpsc::channel::<Envelope<E>>(self.channel_size);
        actor.set_ctx(Context::<E> {
            sender: tx.clone(),
            receiver: rx,
        })?;
        let subscriber = Subscriber::<E, T>::new(Arc::from(actor.name()), topics, tx);
        self.broker.add_subscriber(subscriber);
        Ok(())
    }

    pub async fn start(&mut self) -> Result<()> {
        let handles = self
            .actors
            .iter()
            .cloned()
            .map(|actor| tokio::spawn(async move { actor.lock().await.run().await }))
            .collect::<Vec<_>>();
        self.handles = handles;
        self.broker.run().await;
        Ok(())
    }
}
