use std::{sync::Arc, time::Duration};

use tokio::{
    sync::{Mutex, mpsc::Sender},
    time::sleep,
};

use crate::{
    ActorHandle, Envelope, Event, EventId, Topic,
    testing::{ActorSpy, EventEntry, EventRecords, EventSpy, TestEvent, TopicSpy},
};

pub struct Harness<E: Event, T: Topic<E>> {
    pub(crate) test_sender: Sender<TestEvent<E, T>>,
    actor_sender: Sender<Arc<Envelope<E>>>,
    entries: Arc<Mutex<Vec<EventEntry<E, T>>>>,
}

impl<E: Event, T: Topic<E>> Harness<E, T> {
    pub fn new(
        test_sender: Sender<TestEvent<E, T>>,
        actor_sender: Sender<Arc<Envelope<E>>>,
        entries: Arc<Mutex<Vec<EventEntry<E, T>>>>,
    ) -> Self {
        Self {
            test_sender,
            actor_sender,
            entries,
        }
    }

    async fn records(&self) -> EventRecords<E, T> {
        let entries = self.entries.lock().await;
        Arc::new(entries.clone())
    }

    pub async fn reset(&self) {
        let _ = self.test_sender.send(TestEvent::Reset).await;
    }

    pub async fn stop(&self) {
        let _ = self.test_sender.send(TestEvent::Exit).await;
    }

    pub async fn settle(&self) {
        // FIXME: This is a naive implementation.
        sleep(Duration::from_millis(20)).await
    }

    pub async fn send_as(&self, actor: &ActorHandle, event: E) -> crate::Result<EventSpy<E, T>> {
        let envelope = Envelope::new(event, actor.name());
        let id = envelope.id();
        self.actor_sender.send(Arc::new(envelope)).await?;
        self.settle().await;

        Ok(self.event(id).await)
    }

    pub async fn event(&self, id: EventId) -> EventSpy<E, T> {
        let records = self.records().await;
        EventSpy::new(records, id)
    }

    pub async fn topic(&self, topic: T) -> TopicSpy<E, T> {
        let records = self.records().await;
        TopicSpy::new(records, topic)
    }

    pub async fn actor(&self, actor: &ActorHandle) -> ActorSpy<E, T> {
        let records = self.records().await;
        ActorSpy::new(records, actor.clone())
    }
}
