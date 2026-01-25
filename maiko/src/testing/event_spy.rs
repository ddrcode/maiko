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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testing::EventEntry;
    use crate::{DefaultTopic, Envelope, Event};

    #[derive(Clone, Debug)]
    struct TestEvent(i32);
    impl Event for TestEvent {}

    #[test]
    fn was_delivered_returns_true_when_event_exists() {
        let envelope = Arc::new(Envelope::new(TestEvent(1), "alice"));
        let id = envelope.id();
        let entry = EventEntry::new(envelope, DefaultTopic, Arc::from("bob"));
        let records = Arc::new(vec![entry]);

        let spy = EventSpy::new(records, id);
        assert!(spy.was_delivered());
    }

    #[test]
    fn was_delivered_returns_false_when_event_not_found() {
        let envelope = Arc::new(Envelope::new(TestEvent(1), "alice"));
        let entry = EventEntry::new(envelope, DefaultTopic, Arc::from("bob"));
        let records = Arc::new(vec![entry]);

        let spy = EventSpy::new(records, 99999u128);
        assert!(!spy.was_delivered());
    }

    #[test]
    fn was_delivered_to_returns_true_for_matching_receiver() {
        let envelope = Arc::new(Envelope::new(TestEvent(1), "alice"));
        let id = envelope.id();
        let entry = EventEntry::new(envelope, DefaultTopic, Arc::from("bob"));
        let records = Arc::new(vec![entry]);

        let bob = ActorHandle::new(Arc::from("bob"));
        let spy = EventSpy::new(records, id);
        assert!(spy.was_delivered_to(&bob));
    }

    #[test]
    fn was_delivered_to_returns_false_for_non_matching_receiver() {
        let envelope = Arc::new(Envelope::new(TestEvent(1), "alice"));
        let id = envelope.id();
        let entry = EventEntry::new(envelope, DefaultTopic, Arc::from("bob"));
        let records = Arc::new(vec![entry]);

        let charlie = ActorHandle::new(Arc::from("charlie"));
        let spy = EventSpy::new(records, id);
        assert!(!spy.was_delivered_to(&charlie));
    }

    #[test]
    fn sender_returns_sender_name() {
        let envelope = Arc::new(Envelope::new(TestEvent(1), "alice"));
        let id = envelope.id();
        let entry = EventEntry::new(envelope, DefaultTopic, Arc::from("bob"));
        let records = Arc::new(vec![entry]);

        let spy = EventSpy::new(records, id);
        assert_eq!(&*spy.sender(), "alice");
    }

    #[test]
    fn receivers_returns_all_unique_receivers() {
        let envelope = Arc::new(Envelope::new(TestEvent(1), "alice"));
        let id = envelope.id();
        // Same event delivered to multiple actors
        let entry1 = EventEntry::new(envelope.clone(), DefaultTopic, Arc::from("bob"));
        let entry2 = EventEntry::new(envelope.clone(), DefaultTopic, Arc::from("charlie"));
        let entry3 = EventEntry::new(envelope, DefaultTopic, Arc::from("bob")); // duplicate
        let records = Arc::new(vec![entry1, entry2, entry3]);

        let spy = EventSpy::new(records, id);
        let receivers = spy.receivers();
        assert_eq!(receivers.len(), 2);
        assert!(receivers.iter().any(|r| &**r == "bob"));
        assert!(receivers.iter().any(|r| &**r == "charlie"));
    }

    #[test]
    fn receivers_count_returns_unique_receiver_count() {
        let envelope = Arc::new(Envelope::new(TestEvent(1), "alice"));
        let id = envelope.id();
        let entry1 = EventEntry::new(envelope.clone(), DefaultTopic, Arc::from("bob"));
        let entry2 = EventEntry::new(envelope, DefaultTopic, Arc::from("charlie"));
        let records = Arc::new(vec![entry1, entry2]);

        let spy = EventSpy::new(records, id);
        assert_eq!(spy.receivers_count(), 2);
    }

    #[test]
    fn children_returns_correlated_events() {
        // Parent event
        let parent = Arc::new(Envelope::new(TestEvent(1), "alice"));
        let parent_id = parent.id();
        let parent_entry = EventEntry::new(parent, DefaultTopic, Arc::from("bob"));

        // Child event correlated to parent
        let child = Arc::new(Envelope::with_correlation(TestEvent(2), "bob", parent_id));
        let child_entry = EventEntry::new(child, DefaultTopic, Arc::from("alice"));

        // Unrelated event
        let unrelated = Arc::new(Envelope::new(TestEvent(3), "charlie"));
        let unrelated_entry = EventEntry::new(unrelated, DefaultTopic, Arc::from("alice"));

        let records = Arc::new(vec![parent_entry, child_entry, unrelated_entry]);

        let spy = EventSpy::new(records, parent_id);
        let children = spy.children();
        assert_eq!(children.count(), 1);
        assert_eq!(children.first().unwrap().payload().0, 2);
    }
}
