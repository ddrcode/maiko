use std::sync::Arc;

use crate::{
    Event, Topic,
    testing::{EventEntry, EventQuery, EventRecords, spy_utils},
};

pub struct TopicSpy<E: Event, T: Topic<E>> {
    query: EventQuery<E, T>,
    topic: T,
}

impl<E: Event, T: Topic<E>> TopicSpy<E, T> {
    pub(crate) fn new(records: EventRecords<E, T>, topic: T) -> Self {
        Self {
            query: EventQuery::new(records).with_topic(topic.clone()),
            topic,
        }
    }

    pub fn was_published(&self) -> bool {
        !self.query.is_empty()
    }

    pub fn event_count(&self) -> usize {
        self.query.count()
    }

    pub fn subscribers_count(&self) -> usize {
        todo!()
    }

    pub fn receivers(&self) -> Vec<Arc<str>> {
        spy_utils::distinct(self.query.iter(), |e: &EventEntry<E, T>| {
            e.actor_name.clone()
        })
    }

    pub fn events(&self) -> EventQuery<E, T> {
        EventQuery::new(self.query.records()).with_topic(self.topic.clone())
    }
}
