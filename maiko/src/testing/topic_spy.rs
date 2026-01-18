use std::collections::HashSet;

use crate::{Event, Topic, testing::EventEntry};

pub struct TopicSpy<E: Event, T: Topic<E>> {
    data: Vec<EventEntry<E, T>>,
}

impl<E: Event, T: Topic<E>> TopicSpy<E, T> {
    pub(crate) fn new(entries: &[EventEntry<E, T>], topic: &T) -> Self {
        let data = entries
            .iter()
            .filter(|e| &e.topic == topic)
            .cloned()
            .collect();
        Self { data }
    }
    pub fn was_published(&self) -> bool {
        !self.data.is_empty()
    }
    pub fn publish_count(&self) -> usize {
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
