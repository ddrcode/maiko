use std::sync::Arc;

use tokio::{select, sync::mpsc::Receiver};
use tokio_util::sync::CancellationToken;

use super::Subscriber;
use crate::{Envelope, Event, Result, Topic};

#[derive(Debug)]
pub struct Broker<E: Event, T: Topic<E>> {
    receiver: Receiver<Envelope<E>>,
    subscribers: Vec<Subscriber<E, T>>,
    cancel_token: Arc<CancellationToken>,
}

impl<E: Event, T: Topic<E>> Broker<E, T> {
    pub fn new(
        receiver: Receiver<Envelope<E>>,
        cancel_token: Arc<CancellationToken>,
    ) -> Broker<E, T> {
        Broker {
            receiver,
            subscribers: Vec::new(),
            cancel_token,
        }
    }

    pub(crate) fn add_subscriber(&mut self, subscriber: Subscriber<E, T>) {
        self.subscribers.push(subscriber);
    }

    async fn send_event(&mut self, e: &Envelope<E>) -> Result<()> {
        let topic = Topic::from_event(&e.event);
        for s in self
            .subscribers
            .iter()
            .filter(|s| s.topics.contains(&topic) && s.name != e.meta.sender().into())
        {
            s.sender.send(e.clone()).await?;
        }
        Ok(())
    }

    pub async fn run(&mut self) -> Result<()> {
        loop {
            select! {
                _ = self.cancel_token.cancelled() => break,
                Some(e) = self.receiver.recv() => self.send_event(&e).await?,
            }
        }
        if !self.receiver.is_closed() {
            self.receiver.close();
        }
        Ok(())
    }
}
