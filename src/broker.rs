use tokio::sync::mpsc::Receiver;

use crate::{Envelope, Error, Event, Result, Subscriber, Topic};

#[derive(Debug)]
pub struct Broker<E: Event, T: Topic<E>> {
    channel_size: usize,
    receiver: Receiver<Envelope<E>>,
    subscribers: Vec<Subscriber<E, T>>,
}

impl<E: Event, T: Topic<E>> Broker<E, T> {
    pub fn new(size: usize, receiver: Receiver<Envelope<E>>) -> Broker<E, T> {
        Broker {
            channel_size: size,
            receiver,
            subscribers: Vec::new(),
        }
    }

    pub(crate) fn add_subscriber(&mut self, subscriber: Subscriber<E, T>) {
        self.subscribers.push(subscriber);
    }

    async fn send(&self, event: E) {
        let topic = T::from_event(&event);
        let event = Envelope::new(event);

        for subscriber in &self.subscribers {
            if subscriber.topics.contains(&topic) {
                let _ = subscriber.sender.send(event.clone()).await;
            }
        }
    }

    pub async fn run(&mut self) -> Result<()> {
        while let Some(e) = self.receiver.recv().await {
            let topic = Topic::from_event(&e.event);
            for s in self
                .subscribers
                .iter()
                .filter(|s| s.topics.contains(&topic))
            {
                s.sender
                    .send(e.clone())
                    .await
                    .map_err(|_| Error::SendError)?;
            }
        }
        Ok(())
    }
}
