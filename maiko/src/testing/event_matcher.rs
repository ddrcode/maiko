//! Event matching for chain queries.

use std::borrow::Cow;
use std::rc::Rc;

use crate::{Event, EventId, Label, Topic};

use super::EventEntry;

type MatchFn<E, T> = Rc<dyn Fn(&EventEntry<E, T>) -> bool>;

/// A matcher for filtering events in chain queries.
///
/// `EventMatcher` can match events by:
/// - Event ID (exact match)
/// - Label (using the `Label` trait)
/// - Custom predicate
///
/// # Example
///
/// ```ignore
/// use maiko::testing::EventMatcher;
///
/// // Match by label (requires Event: Label)
/// let matcher = EventMatcher::label("KeyPress");
///
/// // Match by ID
/// let matcher = EventMatcher::id(event_id);
///
/// // Match by predicate
/// let matcher = EventMatcher::matching(|e| matches!(e.payload(), MyEvent::KeyPress(_)));
/// ```
pub struct EventMatcher<E: Event, T: Topic<E>> {
    matcher: MatchFn<E, T>,
}

impl<E: Event, T: Topic<E>> EventMatcher<E, T> {
    /// Match events by their unique ID.
    pub fn id(id: EventId) -> Self {
        Self {
            matcher: Rc::new(move |entry| entry.id() == id),
        }
    }

    /// Match events using a custom predicate on the event entry.
    pub fn matching<F>(predicate: F) -> Self
    where
        F: Fn(&EventEntry<E, T>) -> bool + 'static,
    {
        Self {
            matcher: Rc::new(predicate),
        }
    }

    /// Match events using a custom predicate on the event payload.
    pub fn matching_event<F>(predicate: F) -> Self
    where
        F: Fn(&E) -> bool + 'static,
    {
        Self {
            matcher: Rc::new(move |entry| predicate(entry.payload())),
        }
    }

    /// Returns true if the given entry matches this matcher.
    pub(crate) fn matches(&self, entry: &EventEntry<E, T>) -> bool {
        (self.matcher)(entry)
    }
}

impl<E: Event + Label, T: Topic<E>> EventMatcher<E, T> {
    /// Match events by their label (variant name for enums).
    ///
    /// Requires the event type to implement `Label`.
    pub fn label(name: impl Into<Cow<'static, str>>) -> Self {
        let name: Cow<'static, str> = name.into();
        Self {
            matcher: Rc::new(move |entry| entry.payload().label() == name),
        }
    }
}

// Allow &str to be used directly as a label matcher
impl<E: Event + Label, T: Topic<E>> From<&'static str> for EventMatcher<E, T> {
    fn from(label: &'static str) -> Self {
        EventMatcher::label(label)
    }
}

// Allow String to be used as a label matcher
impl<E: Event + Label, T: Topic<E>> From<String> for EventMatcher<E, T> {
    fn from(label: String) -> Self {
        EventMatcher::label(label)
    }
}

// Allow EventId to be used directly as an id matcher
impl<E: Event, T: Topic<E>> From<EventId> for EventMatcher<E, T> {
    fn from(id: EventId) -> Self {
        EventMatcher::id(id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{ActorId, DefaultTopic, Envelope};
    use std::sync::Arc;

    #[derive(Clone, Debug)]
    #[allow(dead_code)]
    enum TestEvent {
        Ping,
        Pong,
    }

    impl Event for TestEvent {}

    impl Label for TestEvent {
        fn label(&self) -> Cow<'static, str> {
            Cow::Borrowed(match self {
                TestEvent::Ping => "Ping",
                TestEvent::Pong => "Pong",
            })
        }
    }

    fn make_entry(event: TestEvent) -> EventEntry<TestEvent, DefaultTopic> {
        let sender = ActorId::new(Arc::from("sender"));
        let receiver = ActorId::new(Arc::from("receiver"));
        let envelope = Arc::new(Envelope::new(event, sender));
        EventEntry::new(envelope, Arc::new(DefaultTopic), receiver)
    }

    #[test]
    fn label_matcher_matches_by_name() {
        let entry = make_entry(TestEvent::Ping);
        let matcher: EventMatcher<TestEvent, DefaultTopic> = EventMatcher::label("Ping");
        assert!(matcher.matches(&entry));

        let matcher: EventMatcher<TestEvent, DefaultTopic> = EventMatcher::label("Pong");
        assert!(!matcher.matches(&entry));
    }

    #[test]
    fn id_matcher_matches_by_id() {
        let entry = make_entry(TestEvent::Ping);
        let id = entry.id();

        let matcher = EventMatcher::id(id);
        assert!(matcher.matches(&entry));

        let matcher = EventMatcher::id(999999);
        assert!(!matcher.matches(&entry));
    }

    #[test]
    fn matching_event_uses_predicate() {
        let entry = make_entry(TestEvent::Ping);

        let matcher = EventMatcher::matching_event(|e| matches!(e, TestEvent::Ping));
        assert!(matcher.matches(&entry));

        let matcher = EventMatcher::matching_event(|e| matches!(e, TestEvent::Pong));
        assert!(!matcher.matches(&entry));
    }

    #[test]
    fn from_str_creates_label_matcher() {
        let entry = make_entry(TestEvent::Ping);
        let matcher: EventMatcher<TestEvent, DefaultTopic> = "Ping".into();
        assert!(matcher.matches(&entry));
    }
}
