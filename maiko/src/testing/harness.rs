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
    snapshot: EventRecords<E, T>,
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
            snapshot: Arc::new(Vec::new()),
        }
    }

    pub async fn exit(&self) {
        let _ = self.test_sender.send(TestEvent::Exit).await;
    }

    async fn records(&self) -> EventRecords<E, T> {
        let entries = self.entries.lock().await;
        Arc::new(entries.clone())
    }

    pub async fn reset(&mut self) {
        let _ = self.test_sender.send(TestEvent::Reset).await;
        self.snapshot = Arc::new(Vec::new());
    }

    pub async fn start_recording(&mut self) {
        let _ = self.test_sender.send(TestEvent::StartRecording).await;
    }

    pub async fn stop_recording(&mut self) {
        self.settle().await;
        self.snapshot = self.records().await;
        let _ = self.test_sender.send(TestEvent::StopRecording).await;
    }

    pub async fn settle(&self) {
        // FIXME: This is a naive implementation.
        sleep(Duration::from_millis(20)).await
    }

    pub async fn send_as(&self, actor: &ActorHandle, event: E) -> crate::Result<EventId> {
        let envelope = Envelope::new(event, actor.name());
        let id = envelope.id();
        self.actor_sender.send(Arc::new(envelope)).await?;
        Ok(id)
    }

    pub fn event(&self, id: EventId) -> EventSpy<E, T> {
        EventSpy::new(self.snapshot.clone(), id)
    }

    pub fn topic(&self, topic: T) -> TopicSpy<E, T> {
        TopicSpy::new(self.snapshot.clone(), topic)
    }

    pub fn actor(&self, actor: &ActorHandle) -> ActorSpy<E, T> {
        ActorSpy::new(self.snapshot.clone(), actor.clone())
    }

    pub fn dump(&self) {
        for (i, entry) in self.snapshot.iter().enumerate() {
            println!(
                "{}: [{}] -> [{}]: {}",
                i,
                entry.event.meta().actor_name(),
                entry.actor_name,
                entry.event.id()
            );
        }
    }
}
