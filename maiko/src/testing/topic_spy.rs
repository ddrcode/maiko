use std::sync::Arc;

use crate::{
    Event, Topic,
    testing::{EventQuery, EventRecords},
};

/// A spy for observing events on a specific topic.
///
/// Provides methods to inspect:
/// - Whether events were published to this topic
/// - Which actors received events on this topic
pub struct TopicSpy<E: Event, T: Topic<E>> {
    query: EventQuery<E, T>,
}

impl<E: Event, T: Topic<E>> TopicSpy<E, T> {
    pub(crate) fn new(records: EventRecords<E, T>, topic: T) -> Self {
        Self {
            query: EventQuery::new(records).with_topic(topic),
        }
    }

    /// Returns true if any events were published to this topic.
    pub fn was_published(&self) -> bool {
        !self.query.is_empty()
    }

    /// Returns the number of event deliveries on this topic.
    ///
    /// Note: A single event delivered to multiple actors counts multiple times.
    pub fn event_count(&self) -> usize {
        self.query.count()
    }

    /// Returns the names of actors that received events on this topic.
    pub fn receivers(&self) -> Vec<Arc<str>> {
        use std::collections::HashSet;
        self.query
            .iter()
            .map(|e| e.actor_name.clone())
            .collect::<HashSet<_>>()
            .into_iter()
            .collect()
    }

    /// Returns the number of distinct actors that received events on this topic.
    pub fn receivers_count(&self) -> usize {
        self.receivers().len()
    }

    /// Returns a query for further filtering events on this topic.
    pub fn events(&self) -> EventQuery<E, T> {
        self.query.clone()
    }
}
