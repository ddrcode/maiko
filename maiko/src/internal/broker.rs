use std::sync::Arc;

use tokio::{select, sync::mpsc::Receiver};
use tokio_util::sync::CancellationToken;

use super::Subscriber;
use crate::{Envelope, Error, Event, Result, Topic};

#[derive(Debug)]
pub struct Broker<E: Event, T: Topic<E>> {
    receiver: Receiver<Arc<Envelope<E>>>,
    subscribers: Vec<Subscriber<E, T>>,
    cancel_token: Arc<CancellationToken>,
}

impl<E: Event, T: Topic<E>> Broker<E, T> {
    pub fn new(
        receiver: Receiver<Arc<Envelope<E>>>,
        cancel_token: Arc<CancellationToken>,
    ) -> Broker<E, T> {
        Broker {
            receiver,
            subscribers: Vec::new(),
            cancel_token,
        }
    }

    pub(crate) fn add_subscriber(&mut self, subscriber: Subscriber<E, T>) -> Result<()> {
        if self.subscribers.iter().any(|s| s.name == subscriber.name) {
            return Err(Error::SubscriberAlreadyExists(subscriber.name.clone()));
        }
        self.subscribers.push(subscriber);
        Ok(())
    }

    fn send_event(&mut self, e: &Arc<Envelope<E>>) -> Result<()> {
        let topic = Topic::from_event(&e.event);
        self.subscribers
            .iter()
            .filter(|s| s.topics.contains(&topic) && s.name != e.meta.actor_name().into())
            .try_for_each(|subscriber| subscriber.sender.try_send(e.clone()))?;
        Ok(())
    }

    pub async fn run(&mut self) -> Result<()> {
        loop {
            select! {
                _ = self.cancel_token.cancelled() => break,
                Some(e) = self.receiver.recv() => {
                    self.send_event(&e)?;
                },
                else => break
            }
        }
        self.shutdown().await;
        Ok(())
    }

    async fn shutdown(&mut self) {
        use tokio::time::*;

        for _ in 0..self.receiver.len() {
            if let Ok(e) = self.receiver.try_recv() {
                let _ = self.send_event(&e); // Best effort
            } else {
                break; // Queue drained faster than expected
            }
        }

        tokio::task::yield_now().await;

        let start = Instant::now();
        let timeout = Duration::from_millis(10);
        while !self.is_empty() && start.elapsed() < timeout {
            tokio::time::sleep(tokio::time::Duration::from_micros(100)).await;
        }
    }

    pub fn is_empty(&self) -> bool {
        self.subscribers
            .iter()
            .all(|s| s.sender.is_closed() || s.sender.capacity() == s.sender.max_capacity())
    }
}

#[cfg(test)]
mod tests {
    use crate::{Event, Topic, internal::broker::Broker};
    use std::sync::Arc;
    use tokio::sync::mpsc;
    use tokio_util::sync::CancellationToken;

    #[derive(Debug, Clone)]
    struct TestEvent {
        pub id: u32,
    }
    impl Event for TestEvent {}

    #[derive(Debug, Clone, PartialEq, Eq, Hash)]
    enum TestTopic {
        A,
        B,
    }
    impl Topic<TestEvent> for TestTopic {
        fn from_event(event: &TestEvent) -> Self {
            if event.id.is_multiple_of(2) {
                TestTopic::A
            } else {
                TestTopic::B
            }
        }
    }

    #[tokio::test]
    async fn test_add_subscriber() {
        let (tx, rx) = mpsc::channel(10);
        let cancel_token = Arc::new(CancellationToken::new());
        let mut broker = Broker::<TestEvent, TestTopic>::new(rx, cancel_token);
        let subscriber =
            super::Subscriber::new(Arc::from("subscriber1"), &[TestTopic::A], tx.clone());
        assert!(broker.add_subscriber(subscriber).is_ok());
        let duplicate_subscriber =
            super::Subscriber::new(Arc::from("subscriber1"), &[TestTopic::B], tx.clone());
        assert!(broker.add_subscriber(duplicate_subscriber).is_err());
    }
}
