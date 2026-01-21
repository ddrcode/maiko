use std::{sync::Arc, time::Duration};

use tokio::{
    sync::{Mutex, mpsc::Sender},
    time::sleep,
};

use crate::{
    Envelope, Event, EventId, Topic,
    testing::{ActorSpy, EventEntry, EventSpy, TestEvent, TopicSpy},
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

    pub async fn send_as<'a>(
        &self,
        actor_name: impl Into<&'a str>,
        event: E,
    ) -> crate::Result<EventSpy<E, T>> {
        let envelope = Envelope::new(event, actor_name.into());
        let id = envelope.id();
        self.actor_sender.send(Arc::new(envelope)).await?;
        self.settle().await;

        Ok(self.event(id).await)
    }

    pub async fn event(&self, id: EventId) -> EventSpy<E, T> {
        let entries = self.entries.lock().await;
        EventSpy::new(&entries, id)
    }

    pub async fn topic(&self, topic: &T) -> TopicSpy<E, T> {
        let entries = self.entries.lock().await;
        TopicSpy::new(&entries, topic)
    }

    pub async fn actor<'a>(&self, actor_name: impl Into<&'a str>) -> ActorSpy<E, T> {
        let entries = self.entries.lock().await;
        ActorSpy::new(&entries, actor_name)
    }
}
