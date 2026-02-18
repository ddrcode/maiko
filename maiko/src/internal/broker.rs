use std::{collections::HashMap, sync::Arc};

use futures_util::future::join_all;
use tokio::{
    select,
    sync::mpsc::{Receiver, error::TrySendError},
};
use tokio_util::sync::CancellationToken;

use super::Subscriber;
use crate::{ActorId, Config, Envelope, Error, Event, OverflowPolicy, Result, Topic};

#[cfg(feature = "monitoring")]
use crate::monitoring::{MonitoringEvent, MonitoringSink};

type Payload<E> = Arc<Envelope<E>>;
type Recv<E> = Receiver<Payload<E>>;

pub struct Broker<E: Event, T: Topic<E>> {
    senders: HashMap<ActorId, Recv<E>>,
    subscribers: Vec<Subscriber<E, T>>,
    cancel_token: Arc<CancellationToken>,
    config: Arc<Config>,

    #[cfg(feature = "monitoring")]
    monitoring: MonitoringSink<E, T>,
}

impl<E: Event, T: Topic<E>> Broker<E, T> {
    pub fn new(
        cancel_token: Arc<CancellationToken>,
        config: Arc<Config>,
        #[cfg(feature = "monitoring")] monitoring: MonitoringSink<E, T>,
    ) -> Broker<E, T> {
        Broker {
            senders: HashMap::new(),
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

    pub(crate) fn add_sender(&mut self, actor: ActorId, receiver: Recv<E>) {
        self.senders.insert(actor, receiver);
    }

    async fn send_event(&mut self, e: &Arc<Envelope<E>>) -> Result<Option<Vec<ActorId>>> {
        let topic = T::from_event(e.event());
        let mut blocked = None;
        let mut to_be_closed = None;

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

        for subscriber in self
            .subscribers
            .iter()
            .filter(|s| s.topics.contains(&topic))
            .filter(|s| !s.is_closed())
            .filter(|s| s.actor_id != *e.meta().actor_id())
        {
            match subscriber.sender.try_send(e.clone()) {
                Ok(_) => {
                    #[cfg(feature = "monitoring")]
                    self.record_event_dispatched(
                        is_recording,
                        e,
                        &topic_for_monitor,
                        &subscriber.actor_id,
                    );
                }
                Err(TrySendError::Full(event)) => {
                    let policy = topic.overflow_policy();
                    #[cfg(feature = "monitoring")]
                    self.record_overflow(
                        is_recording,
                        e,
                        &topic_for_monitor,
                        &subscriber.actor_id,
                        policy,
                    );
                    match policy {
                        OverflowPolicy::Fail => {
                            tracing::error!(actor=%subscriber.actor_id.name(), event_id=%e.id(), "closing channel due to OverflowPolicy Fail");
                            to_be_closed
                                .get_or_insert(Vec::new())
                                .push(subscriber.actor_id.clone());
                            continue;
                        }
                        OverflowPolicy::Drop => {
                            continue;
                        }
                        OverflowPolicy::Block => {
                            let fut = subscriber.sender.send(event);
                            blocked.get_or_insert(Vec::new()).push(fut);
                        }
                    };
                }
                Err(TrySendError::Closed(_)) => {
                    // Channel is closed, will be cleaned up in the next maintenance cycle
                    tracing::warn!(actor=%subscriber.actor_id.name(), "subscriber channel closed, will be removed in cleanup");
                }
            }
        }

        if let Some(b) = blocked.take() {
            join_all(b).await;
        }

        Ok(to_be_closed)
    }

    async fn recv(senders: &mut HashMap<ActorId, Recv<E>>) -> Option<Vec<Payload<E>>> {
        let mut events = Vec::with_capacity(senders.len() >> 1);
        for receiver in senders.values_mut() {
            while let Ok(event) = receiver.try_recv() {
                events.push(event);
            }
        }
        if !events.is_empty() {
            Some(events)
        } else {
            None
        }
    }

    pub async fn run(&mut self) -> Result<()> {
        let mut cleanup_interval = tokio::time::interval(self.config.maintenance_interval());
        let mut senders = std::mem::take(&mut self.senders);
        loop {
            select! {
                biased;
                _ = self.cancel_token.cancelled() => break,
                _ = cleanup_interval.tick() => {
                    self.cleanup();
                }
                Some(events) = Self::recv(&mut senders) => {
                    for e in events{
                        let tbc = self.send_event(&e).await?;

                        // Close channels for subscribers that overflown with Fail policy
                        if let Some(to_be_closed) = tbc {
                            self.subscribers.retain(|s|
                                !to_be_closed.contains(&s.actor_id)
                            );
                        }
                    }
                },
            }
        }
        self.shutdown().await;
        Ok(())
    }

    fn cleanup(&mut self) {
        self.subscribers.retain(|s| !s.is_closed());
    }

    async fn shutdown(&mut self) {
        use tokio::time::*;

        // Send messages that were in the queue at the time of shutdown
        // for _ in 0..self.receiver.len() {
        //     if let Ok(e) = self.receiver.try_recv() {
        //         let _ = self.send_event(&e).await; // Best effort
        //     } else {
        //         break; // Queue drained faster than expected
        //     }
        // }

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
            .all(|s| s.is_closed() || s.sender.capacity() == s.sender.max_capacity())
    }
}

#[cfg(feature = "monitoring")]
impl<E: Event, T: Topic<E>> Broker<E, T> {
    #[inline]
    fn record_event_dispatched(
        &self,
        is_recording: bool,
        e: &Arc<Envelope<E>>,
        topic: &Option<Arc<T>>,
        actor_id: &ActorId,
    ) {
        if is_recording {
            if let Some(topic_for_monitor) = topic {
                self.monitoring.send(MonitoringEvent::EventDispatched(
                    e.clone(),
                    topic_for_monitor.clone(),
                    actor_id.clone(),
                ));
            }
        }
    }

    #[inline]
    fn record_overflow(
        &self,
        is_recording: bool,
        e: &Arc<Envelope<E>>,
        topic: &Option<Arc<T>>,
        actor_id: &ActorId,
        policy: OverflowPolicy,
    ) {
        if is_recording {
            if let Some(topic_for_monitor) = topic {
                self.monitoring.send(MonitoringEvent::Overflow(
                    e.clone(),
                    topic_for_monitor.clone(),
                    actor_id.clone(),
                    policy,
                ));
            }
        }
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
            if event.id % 2 == 0 {
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
