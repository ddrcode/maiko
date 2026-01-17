use std::{sync::Arc, time::Duration};

use tokio::{
    sync::{Mutex, mpsc::Sender},
    time::sleep,
};

use crate::{
    Envelope, Event, Topic,
    test_harness::{EventEntry, TestEvent},
};

pub struct TestHarness<E: Event, T: Topic<E>> {
    test_sender: Sender<TestEvent<E, T>>,
    actor_sender: Sender<Arc<Envelope<E>>>,
    entries: Arc<Mutex<Vec<EventEntry<E, T>>>>,
}

impl<E: Event, T: Topic<E>> TestHarness<E, T> {
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
        sleep(Duration::from_millis(500)).await
    }

    pub async fn send_as<'a, N>(&self, actor_name: N, event: E) -> crate::Result<EventSpy<E, T>>
    where
        N: Into<&'a str>,
    {
        let envelope = Envelope::new(event, actor_name.into());
        let id = envelope.id();
        self.actor_sender.send(Arc::new(envelope)).await?;
        self.settle().await;

        Ok(self.spy_event(&id).await)
    }

    pub async fn spy_event(&self, id: &u128) -> EventSpy<E, T> {
        let entries = self.entries.lock().await;
        EventSpy::new(&entries, id)
    }
}

pub struct EventSpy<E: Event, T: Topic<E>> {
    data: Vec<EventEntry<E, T>>,
}

impl<E: Event, T: Topic<E>> EventSpy<E, T> {
    pub(crate) fn new(entries: &[EventEntry<E, T>], id: &u128) -> Self {
        let data = entries
            .iter()
            .filter(|e| *id == e.event.id())
            .cloned()
            .collect();
        Self { data }
    }

    pub fn was_delivered(&self) -> bool {
        !self.data.is_empty()
    }

    pub fn was_delivered_to(&self, actor_name: &str) -> bool {
        println!("Spy entries for event {}", self.data.len());
        self.data
            .iter()
            .any(|e| e.actor_name.as_ref() == actor_name)
    }

    pub fn receivers_count(&self) -> usize {
        self.data.len()
    }

    pub fn receivers(&self) -> Vec<&str> {
        self.data.iter().map(|e| e.actor_name.as_ref()).collect()
    }
}
