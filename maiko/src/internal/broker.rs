use std::sync::Arc;

use tokio::{select, sync::mpsc::Receiver};
use tokio_util::sync::CancellationToken;

use super::Subscriber;
use crate::{Config, Envelope, Error, Event, Result, Topic};

#[cfg(feature = "test-harness")]
use crate::test_harness::{EventEntry, TestEvent};

pub struct Broker<E: Event, T: Topic<E>> {
    receiver: Receiver<Arc<Envelope<E>>>,
    subscribers: Vec<Subscriber<E, T>>,
    cancel_token: Arc<CancellationToken>,
    config: Arc<Config>,

    #[cfg(feature = "test-harness")]
    test_sender: Option<tokio::sync::mpsc::Sender<TestEvent<E, T>>>,
}

impl<E: Event, T: Topic<E>> Broker<E, T> {
    pub fn new(
        receiver: Receiver<Arc<Envelope<E>>>,
        cancel_token: Arc<CancellationToken>,
        config: Arc<Config>,
    ) -> Broker<E, T> {
        Broker {
            receiver,
            subscribers: Vec::new(),
            cancel_token,
            config,
            #[cfg(feature = "test-harness")]
            test_sender: None,
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
        let topic = Topic::from_event(e.event());
        self.subscribers
            .iter()
            .filter(|s| s.topics.contains(&topic))
            .filter(|s| !s.sender.is_closed())
            .filter(|s| !Arc::ptr_eq(&s.name, &e.meta().actor_name))
            .try_for_each(|subscriber| {
                let res = subscriber.sender.try_send(e.clone());
                #[cfg(feature = "test-harness")]
                if let Some(ref sender) = self.test_sender {
                    let _ = sender.try_send(TestEvent::Event(EventEntry::new(
                        e.clone(),
                        topic.clone(),
                        subscriber.name.clone(),
                    )));
                }
                res
            })?;
        Ok(())
    }

    pub async fn run(&mut self) -> Result<()> {
        let mut cleanup_interval = tokio::time::interval(self.config.maintenance_interval);
        loop {
            select! {
                _ = self.cancel_token.cancelled() => break,
                Some(e) = self.receiver.recv() => {
                    self.send_event(&e)?;
                },
                _ = cleanup_interval.tick() => {
                    self.cleanup();
                }
            }
        }
        self.shutdown().await;
        Ok(())
    }

    fn cleanup(&mut self) {
        self.subscribers.retain(|s| !s.sender.is_closed());
    }

    async fn shutdown(&mut self) {
        use tokio::time::*;

        // Send messages that were in the queue at the time of shutdown
        for _ in 0..self.receiver.len() {
            if let Ok(e) = self.receiver.try_recv() {
                let _ = self.send_event(&e); // Best effort
            } else {
                break; // Queue drained faster than expected
            }
        }

        tokio::task::yield_now().await;

        // Wait the inner channels to be consumed by the actors
        let start = Instant::now();
        let timeout = Duration::from_millis(10);
        while !self.is_empty() && start.elapsed() < timeout {
            sleep(Duration::from_micros(100)).await;
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
    use crate::{Event, Topic, internal::broker::Broker, test_harness::init_harness};
    use std::{collections::HashSet, sync::Arc};
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
        let config = Arc::new(crate::Config::default());
        let cancel_token = Arc::new(CancellationToken::new());
        let (harness, harness_tx) = init_harness();
        let mut broker = Broker::<TestEvent, TestTopic>::new(rx, cancel_token, config, harness_tx);
        let subscriber = super::Subscriber::new(
            Arc::from("subscriber1"),
            HashSet::from([TestTopic::A]),
            tx.clone(),
        );
        assert!(broker.add_subscriber(subscriber).is_ok());
        let duplicate_subscriber = super::Subscriber::new(
            Arc::from("subscriber1"),
            HashSet::from([TestTopic::B]),
            tx.clone(),
        );
        assert!(broker.add_subscriber(duplicate_subscriber).is_err());
    }
}
