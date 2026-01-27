use std::sync::Arc;

use tokio::{select, sync::mpsc::Receiver};
use tokio_util::sync::CancellationToken;

use super::Subscriber;
use crate::{Config, Envelope, Error, Event, Result, Topic};

#[cfg(feature = "monitoring")]
use crate::monitoring::{MonitoringEvent, MonitoringSink};

pub struct Broker<E: Event, T: Topic<E>> {
    receiver: Receiver<Arc<Envelope<E>>>,
    subscribers: Vec<Subscriber<E, T>>,
    cancel_token: Arc<CancellationToken>,
    config: Arc<Config>,

    #[cfg(feature = "monitoring")]
    monitoring: MonitoringSink<E, T>,
}

impl<E: Event, T: Topic<E>> Broker<E, T> {
    pub fn new(
        receiver: Receiver<Arc<Envelope<E>>>,
        cancel_token: Arc<CancellationToken>,
        config: Arc<Config>,
        #[cfg(feature = "monitoring")] monitoring: MonitoringSink<E, T>,
    ) -> Broker<E, T> {
        Broker {
            receiver,
            subscribers: Vec::new(),
            cancel_token,
            config,
            #[cfg(feature = "monitoring")]
            monitoring,
        }
    }

    pub(crate) fn add_subscriber(&mut self, subscriber: Subscriber<E, T>) -> Result<()> {
        if self.subscribers.contains(&subscriber) {
            return Err(Error::SubscriberAlreadyExists(subscriber.actor_id.clone()));
        }
        self.subscribers.push(subscriber);
        Ok(())
    }

    fn send_event(&mut self, e: &Arc<Envelope<E>>) -> Result<()> {
        let topic = T::from_event(e.event());

        #[cfg(feature = "monitoring")]
        let (is_recording, topic_for_monitor) = {
            let active = self.monitoring.is_active();
            let t = if active {
                Some(Arc::new(topic.clone()))
            } else {
                None
            };
            (active, t)
        };

        self.subscribers
            .iter()
            .filter(|s| s.topics.contains(&topic))
            .filter(|s| !s.sender.is_closed())
            .filter(|s| s.actor_id != *e.meta().actor_id())
            .try_for_each(|subscriber| {
                let res = subscriber.sender.try_send(e.clone());

                #[cfg(feature = "monitoring")]
                if is_recording {
                    if let Some(topic_for_monitor) = &topic_for_monitor {
                        self.monitoring.send(MonitoringEvent::EventDispatched(
                            e.clone(),
                            topic_for_monitor.clone(),
                            subscriber.actor_id.clone(),
                        ));
                    }
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
    use crate::{Event, Topic, internal::Subscription, internal::broker::Broker};
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
        use crate::ActorId;

        let (tx, rx) = mpsc::channel(10);
        let config = Arc::new(crate::Config::default());
        let cancel_token = Arc::new(CancellationToken::new());

        #[cfg(feature = "monitoring")]
        let monitoring = {
            let registry = crate::monitoring::MonitorRegistry::<TestEvent, TestTopic>::new(&config);
            registry.sink()
        };

        let mut broker = Broker::<TestEvent, TestTopic>::new(
            rx,
            cancel_token,
            config,
            #[cfg(feature = "monitoring")]
            monitoring,
        );
        let actor_id = ActorId::new(Arc::from("subscriber1"));
        let subscriber = super::Subscriber::new(
            actor_id.clone(),
            Subscription::Topics(HashSet::from([TestTopic::A])),
            tx.clone(),
        );
        assert!(broker.add_subscriber(subscriber).is_ok());
        let duplicate_subscriber = super::Subscriber::new(
            actor_id,
            Subscription::Topics(HashSet::from([TestTopic::B])),
            tx.clone(),
        );
        assert!(broker.add_subscriber(duplicate_subscriber).is_err());
    }
}
