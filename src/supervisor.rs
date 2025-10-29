use std::sync::Arc;

use tokio::sync::mpsc::{Receiver, Sender, channel};

use crate::{Actor, Broker, Context, Envelope, Event, Subscriber, Topic, subscriber};

#[derive(Debug)]
pub struct Supervisor<E: Event, T: Topic<E>> {
    channel_size: usize,
    broker: Broker<E, T>,
    sender: Sender<Envelope<E>>,
}

impl<E: Event, T: Topic<E>> Supervisor<E, T> {
    pub fn new(channel_size: usize) -> Self {
        let (tx, rx) = channel::<Envelope<E>>(channel_size);
        Self {
            channel_size,
            broker: Broker::new(channel_size, rx),
            sender: tx,
        }
    }

    pub fn add_actor<A: Actor<Event = E>>(&mut self, mut actor: A, topics: Vec<T>) {
        let (tx, rx) = tokio::sync::mpsc::channel::<Envelope<E>>(self.channel_size);
        actor.set_ctx(Context::<E> {
            sender: tx.clone(),
            receiver: rx,
        });
        let subscriber = Subscriber::<E, T>::new(Arc::from(actor.name()), topics, tx);
        self.broker.add_subscriber(subscriber);
    }

    pub fn start(&mut self) {}
}
