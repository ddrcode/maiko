use std::collections::HashSet;

use crate::{Event, Topic, testing::EventEntry};

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

    pub fn was_delivered_to<'a, N: Into<&'a str>>(&self, actor_name: N) -> bool {
        let actor_name = actor_name.into();
        self.data
            .iter()
            .any(|e| e.actor_name.as_ref() == actor_name)
    }

    pub fn receivers_count(&self) -> usize {
        self.data.len()
    }

    pub fn receivers(&self) -> Vec<&str> {
        self.data
            .iter()
            .map(|e| e.actor_name.as_ref())
            .collect::<HashSet<_>>()
            .into_iter()
            .collect()
    }
}
