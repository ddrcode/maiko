use std::sync::Arc;

use tokio::sync::{Mutex, mpsc::Sender};

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

    pub async fn send_as(&self, actor_name: &str, event: E) -> crate::Result {
        let envelope = Envelope::new(event, actor_name);
        self.actor_sender.send(Arc::new(envelope)).await?;

        Ok(())
    }
}


pub struct EventSpy<E: Event, T: Topic<E>> {
    fn new(entries: &Vec<EventEntry<E,T>>, id: &u128) -> Self {
        entries.iter().filter(|e| *e.id() == *id);
        Self{}
    }
}
