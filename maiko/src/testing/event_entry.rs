use std::sync::Arc;

use crate::{ActorHandle, Envelope, Event, EventId, Meta, Topic};

/// A record of an event delivery from one actor to another.
///
/// Each `EventEntry` represents a single delivery: the same event sent to
/// multiple actors produces multiple entries (one per recipient).
///
/// # Fields
///
/// - `event`: The envelope containing the event payload and metadata
/// - `topic`: The topic under which this event was routed
/// - `actor_name`: The name of the receiving actor
#[derive(Debug, Clone)]
pub struct EventEntry<E: Event, T: Topic<E>> {
    pub(crate) event: Arc<Envelope<E>>,
    pub(crate) topic: T,
    pub(crate) actor_name: Arc<str>,
}

impl<E: Event, T: Topic<E>> EventEntry<E, T> {
    pub(crate) fn new(event: Arc<Envelope<E>>, topic: T, actor_name: Arc<str>) -> Self {
        Self {
            event,
            topic,
            actor_name,
        }
    }

    /// Returns the unique ID of this event.
    #[inline]
    pub fn id(&self) -> EventId {
        self.event.id()
    }

    /// Returns a reference to the event payload.
    #[inline]
    pub fn payload(&self) -> &E {
        self.event.event()
    }

    /// Returns the event metadata (sender, timestamp, correlation).
    #[inline]
    pub fn meta(&self) -> &Meta {
        self.event.meta()
    }

    /// Returns the topic this event was routed under.
    #[inline]
    pub fn topic(&self) -> &T {
        &self.topic
    }

    /// Returns the name of the actor that sent this event.
    #[inline]
    pub fn sender(&self) -> &str {
        self.meta().actor_name()
    }

    /// Returns the name of the actor that received this event.
    #[inline]
    pub fn receiver(&self) -> &str {
        &self.actor_name
    }

    /// Returns true if this event was received by the specified actor.
    #[inline]
    pub(crate) fn receiver_actor_eq(&self, actor_handle: &ActorHandle) -> bool {
        self.actor_name == actor_handle.name
    }

    /// Returns true if this event was sent by the specified actor.
    #[inline]
    pub(crate) fn sender_actor_eq(&self, actor_handle: &ActorHandle) -> bool {
        self.meta().actor_name == actor_handle.name
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::DefaultTopic;

    #[derive(Clone, Debug)]
    struct TestEvent(i32);
    impl Event for TestEvent {}

    fn make_entry() -> EventEntry<TestEvent, DefaultTopic> {
        let envelope = Arc::new(Envelope::new(TestEvent(42), "sender-actor"));
        EventEntry::new(envelope, DefaultTopic, Arc::from("receiver-actor"))
    }

    #[test]
    fn id_returns_envelope_id() {
        let entry = make_entry();
        // ID should be non-zero (generated)
        assert_ne!(entry.id(), 0);
    }

    #[test]
    fn payload_returns_event() {
        let entry = make_entry();
        assert_eq!(entry.payload().0, 42);
    }

    #[test]
    fn meta_returns_envelope_meta() {
        let entry = make_entry();
        assert_eq!(entry.meta().actor_name(), "sender-actor");
    }

    #[test]
    fn topic_returns_routing_topic() {
        let entry = make_entry();
        assert_eq!(*entry.topic(), DefaultTopic);
    }

    #[test]
    fn sender_returns_sender_name() {
        let entry = make_entry();
        assert_eq!(entry.sender(), "sender-actor");
    }

    #[test]
    fn receiver_returns_receiver_name() {
        let entry = make_entry();
        assert_eq!(entry.receiver(), "receiver-actor");
    }

    #[test]
    fn receiver_actor_eq_matches_correctly() {
        let entry = make_entry();
        let matching = ActorHandle::new(Arc::from("receiver-actor"));
        let not_matching = ActorHandle::new(Arc::from("other-actor"));

        assert!(entry.receiver_actor_eq(&matching));
        assert!(!entry.receiver_actor_eq(&not_matching));
    }

    #[test]
    fn sender_actor_eq_matches_correctly() {
        let entry = make_entry();
        let matching = ActorHandle::new(Arc::from("sender-actor"));
        let not_matching = ActorHandle::new(Arc::from("other-actor"));

        assert!(entry.sender_actor_eq(&matching));
        assert!(!entry.sender_actor_eq(&not_matching));
    }
}
