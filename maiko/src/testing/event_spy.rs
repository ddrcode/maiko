use std::sync::Arc;

use crate::{
    ActorHandle, Event, EventId, Topic,
    testing::{EventQuery, EventRecords, spy_utils},
};

pub struct EventSpy<E: Event, T: Topic<E>> {
    id: EventId,
    records: EventRecords<E, T>,
    query: EventQuery<E, T>,
}

impl<E: Event, T: Topic<E>> EventSpy<E, T> {
    pub(crate) fn new(records: EventRecords<E, T>, id: impl Into<EventId>) -> Self {
        let id = id.into();
        let query = EventQuery::new(records.clone()).with_event(id);
        Self { id, records, query }
    }

    pub fn was_delivered(&self) -> bool {
        !self.query.is_empty()
    }

    pub fn sender(&self) -> Arc<str> {
        self.query
            .iter()
            .find(|e| self.id == e.event.id())
            .map(|e| e.event.meta().actor_name.clone())
            .expect("sender must exist in EventSpy")
    }

    pub fn was_delivered_to(&self, actor: &ActorHandle) -> bool {
        self.query.any(|e| e.actor_name.as_ref() == actor.name())
    }

    pub fn receivers_count(&self) -> usize {
        self.receivers().len()
    }

    pub fn receivers(&self) -> Vec<Arc<str>> {
        spy_utils::distinct(self.query.iter(), |e| e.actor_name.clone())
    }

    pub fn children(&self) -> EventQuery<E, T> {
        EventQuery::new(self.records.clone()).correlated_with(self.id)
    }
}
