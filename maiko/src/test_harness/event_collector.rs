use std::sync::Arc;

use tokio::sync::{Mutex, mpsc::Receiver};

use crate::{
    Event, Topic,
    test_harness::{EventEntry, TestEvent},
};

pub struct EventCollector<E: Event, T: Topic<E>> {
    events: Arc<Mutex<Vec<EventEntry<E, T>>>>,
    receiver: Receiver<TestEvent<E, T>>,
}

impl<E: Event, T: Topic<E>> EventCollector<E, T> {
    pub fn new(
        receiver: Receiver<TestEvent<E, T>>,
        events: Arc<Mutex<Vec<EventEntry<E, T>>>>,
    ) -> Self {
        Self { receiver, events }
    }

    pub async fn run(&mut self) -> crate::Result {
        let mut is_alive = true;
        while is_alive {
            if let Some(event) = self.receiver.recv().await {
                let events = &mut self.events.lock().await;
                is_alive = Self::handle_event(events, event);
                while let Ok(event) = self.receiver.try_recv()
                    && is_alive
                {
                    is_alive = Self::handle_event(events, event);
                }
            }
        }
        Ok(())
    }

    fn handle_event(events: &mut Vec<EventEntry<E, T>>, event: TestEvent<E, T>) -> bool {
        match event {
            TestEvent::Event(entry) => events.push(entry),
            TestEvent::Exit => return false,
            TestEvent::Reset => events.clear(),
        }
        true
    }
}
