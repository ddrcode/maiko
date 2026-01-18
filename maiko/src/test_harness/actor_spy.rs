use crate::{Event, Topic, test_harness::EventEntry};

pub struct ActorSpy<E: Event, T: Topic<E>> {
    received_data: Vec<EventEntry<E, T>>,
    sent_data: Vec<EventEntry<E, T>>,
}

impl<E: Event, T: Topic<E>> ActorSpy<E, T> {
    pub(crate) fn new<'a, N>(entries: &[EventEntry<E, T>], actor_name: N) -> Self
    where
        N: Into<&'a str>,
    {
        let actor_name = actor_name.into();
        let received_data: Vec<EventEntry<E, T>> = entries
            .iter()
            .filter(|e| e.actor_name.as_ref() == actor_name)
            .cloned()
            .collect();
        let sent_data: Vec<EventEntry<E, T>> = entries
            .iter()
            .filter(|e| e.event.meta().actor_name() == actor_name)
            .cloned()
            .collect();

        Self {
            received_data,
            sent_data,
        }
    }
    pub fn received_events_count(&self) -> usize {
        self.received_data.len()
    }
    pub fn sent_events_count(&self) -> usize {
        self.sent_data.len()
    }
}
