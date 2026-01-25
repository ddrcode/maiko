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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testing::EventEntry;
    use crate::{Envelope, Event};

    #[derive(Clone, Debug)]
    struct TestEvent(i32);
    impl Event for TestEvent {}

    #[derive(Clone, Copy, Debug, Hash, Eq, PartialEq)]
    enum TestTopic {
        Data,
        Control,
    }

    impl Topic<TestEvent> for TestTopic {
        fn from_event(event: &TestEvent) -> Self {
            if event.0 < 100 {
                TestTopic::Data
            } else {
                TestTopic::Control
            }
        }
    }

    fn make_entry(
        event: TestEvent,
        sender: &str,
        receiver: &str,
    ) -> EventEntry<TestEvent, TestTopic> {
        let topic = TestTopic::from_event(&event);
        let envelope = Arc::new(Envelope::new(event, sender));
        EventEntry::new(envelope, topic, Arc::from(receiver))
    }

    fn sample_records() -> EventRecords<TestEvent, TestTopic> {
        Arc::new(vec![
            make_entry(TestEvent(1), "alice", "bob"),     // Data
            make_entry(TestEvent(2), "alice", "charlie"), // Data
            make_entry(TestEvent(100), "bob", "alice"),   // Control
            make_entry(TestEvent(101), "bob", "charlie"), // Control
        ])
    }

    #[test]
    fn was_published_returns_true_when_events_exist() {
        let spy = TopicSpy::new(sample_records(), TestTopic::Data);
        assert!(spy.was_published());
    }

    #[test]
    fn was_published_returns_false_when_no_events() {
        let records: EventRecords<TestEvent, TestTopic> = Arc::new(vec![
            make_entry(TestEvent(100), "alice", "bob"), // Only Control
        ]);
        let spy = TopicSpy::new(records, TestTopic::Data);
        assert!(!spy.was_published());
    }

    #[test]
    fn event_count_returns_delivery_count() {
        let spy = TopicSpy::new(sample_records(), TestTopic::Data);
        assert_eq!(spy.event_count(), 2);
    }

    #[test]
    fn event_count_counts_multiple_deliveries() {
        // Same event delivered to multiple actors
        let envelope = Arc::new(Envelope::new(TestEvent(1), "alice"));
        let records = Arc::new(vec![
            EventEntry::new(envelope.clone(), TestTopic::Data, Arc::from("bob")),
            EventEntry::new(envelope, TestTopic::Data, Arc::from("charlie")),
        ]);

        let spy = TopicSpy::new(records, TestTopic::Data);
        assert_eq!(spy.event_count(), 2); // 2 deliveries
    }

    #[test]
    fn receivers_returns_unique_actors() {
        let spy = TopicSpy::new(sample_records(), TestTopic::Control);
        let receivers = spy.receivers();
        assert_eq!(receivers.len(), 2);
        assert!(receivers.iter().any(|r| &**r == "alice"));
        assert!(receivers.iter().any(|r| &**r == "charlie"));
    }

    #[test]
    fn receivers_count_returns_unique_receiver_count() {
        let spy = TopicSpy::new(sample_records(), TestTopic::Data);
        assert_eq!(spy.receivers_count(), 2); // bob and charlie
    }

    #[test]
    fn events_returns_filterable_query() {
        let spy = TopicSpy::new(sample_records(), TestTopic::Data);
        let alice_events = spy.events().matching(|e| e.sender() == "alice").count();
        assert_eq!(alice_events, 2);
    }

    #[test]
    fn empty_topic_has_zero_counts() {
        let records: EventRecords<TestEvent, TestTopic> = Arc::new(vec![]);
        let spy = TopicSpy::new(records, TestTopic::Data);

        assert!(!spy.was_published());
        assert_eq!(spy.event_count(), 0);
        assert_eq!(spy.receivers_count(), 0);
        assert!(spy.receivers().is_empty());
    }
}
