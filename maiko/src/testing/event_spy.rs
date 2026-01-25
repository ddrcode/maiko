use std::sync::Arc;

use crate::{
    ActorHandle, Event, EventId, Topic,
    testing::{EventQuery, EventRecords},
};

/// A spy for observing the delivery and effects of a specific event.
///
/// Provides methods to inspect:
/// - Whether and where the event was delivered
/// - Child events correlated to this event
pub struct EventSpy<E: Event, T: Topic<E>> {
    id: EventId,
    records: EventRecords<E, T>,
    query: EventQuery<E, T>,
}

impl<E: Event, T: Topic<E>> EventSpy<E, T> {
    pub(crate) fn new(records: EventRecords<E, T>, id: impl Into<EventId>) -> Self {
        let id = id.into();
        let query = EventQuery::new(records.clone()).with_id(id);
        Self { id, records, query }
    }

    /// Returns true if the event was delivered to at least one actor.
    pub fn was_delivered(&self) -> bool {
        !self.query.is_empty()
    }

    /// Returns true if the event was delivered to the specified actor.
    pub fn was_delivered_to(&self, actor: &ActorHandle) -> bool {
        self.query.clone().received_by(actor).count() > 0
    }

    /// Returns the name of the actor that sent this event.
    pub fn sender(&self) -> Arc<str> {
        self.query
            .first()
            .map(|e| e.meta().actor_name.clone())
            .expect("EventSpy must have at least one delivery record")
    }

    /// Returns the number of actors that received this event.
    pub fn receivers_count(&self) -> usize {
        self.receivers().len()
    }

    /// Returns the names of actors that received this event.
    pub fn receivers(&self) -> Vec<Arc<str>> {
        use std::collections::HashSet;
        self.query
            .iter()
            .map(|e| e.actor_name.clone())
            .collect::<HashSet<_>>()
            .into_iter()
            .collect()
    }

    /// Returns a query for child events (events correlated to this one).
    ///
    /// Child events are those whose `correlation_id` matches this event's `id`.
    pub fn children(&self) -> EventQuery<E, T> {
        EventQuery::new(self.records.clone()).correlated_with(self.id)
    }
}
