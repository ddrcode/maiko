use std::hash::Hash;

use crate::event::Event;

/// Maps events to routing topics.
///
/// Implement this for your own topic type (usually an enum) to classify
/// events for the broker. Actors subscribe to one or more topics, and the
/// broker delivers events to matching subscribers.
///
/// Common patterns:
/// - Enum topics for simple classification.
/// - Struct topics when you need richer metadata (e.g., names or IDs).
///
/// Trait bounds: refer to the event trait as [`crate::Event`] in generic
/// signatures to avoid confusion with the `Event` derive macro.
pub trait Topic<E: Event>: Hash + PartialEq + Eq + Clone {
    fn from_event(event: &E) -> Self
    where
        Self: Sized;
}

/// Default topic that broadcasts events to all subscribed actors.
///
/// Use `Broadcast` when you don't need topic-based filtering and want
/// all actors to receive all events. This is the simplest routing strategy.
///
/// # Examples
///
/// ```rust, ignore
/// use maiko::{Supervisor, Broadcast};
/// let mut sup = Supervisor::<MyEvent>::default();
/// sup.add_actor("actor", |ctx| MyActor { ctx }, &[Broadcast])?;
/// ```
#[derive(Debug, Hash, Eq, PartialEq, Clone)]
pub struct Broadcast;

impl<E: Event> Topic<E> for Broadcast {
    fn from_event(_event: &E) -> Broadcast {
        Broadcast
    }
}

impl std::fmt::Display for Broadcast {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "broadcast")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::event::Event;

    #[derive(Debug, Clone)]
    struct TestEvent;

    impl Event for TestEvent {}

    #[test]
    fn test_default_topic() {
        let event = TestEvent;
        let result = Broadcast::from_event(&event);
        assert_eq!(result.to_string(), "broadcast");
    }

    #[test]
    fn test_topic_as_enum() {
        #[allow(dead_code)]
        #[derive(Clone)]
        enum TestEvent {
            Temperature(f64),
            Humidity(f64),
            Exit,
            Sleep(u64),
        }
        impl Event for TestEvent {}

        #[derive(Debug, PartialEq, Eq, Hash, Clone)]
        enum TestTopic {
            IoT,
            System,
        }
        impl Topic<TestEvent> for TestTopic {
            fn from_event(event: &TestEvent) -> Self {
                match event {
                    TestEvent::Temperature(_) | TestEvent::Humidity(_) => TestTopic::IoT,
                    TestEvent::Exit | TestEvent::Sleep(_) => TestTopic::System,
                }
            }
        }
        assert_eq!(
            TestTopic::from_event(&TestEvent::Temperature(22.5)),
            TestTopic::IoT
        );
        assert_eq!(TestTopic::from_event(&TestEvent::Exit), TestTopic::System);
        assert_eq!(
            TestTopic::from_event(&TestEvent::Sleep(1000)),
            TestTopic::System
        );
        assert_eq!(
            TestTopic::from_event(&TestEvent::Humidity(45.0)),
            TestTopic::IoT
        );
    }

    #[test]
    fn test_topic_as_struct() {
        #[allow(dead_code)]
        #[derive(Clone)]
        enum TestEvent {
            Temperature(f64),
            Humidity(f64),
            Exit,
            Sleep(u64),
        }
        impl Event for TestEvent {}

        #[derive(Debug, PartialEq, Eq, Hash, Clone)]
        struct TestTopic {
            name: String,
        }
        impl Topic<TestEvent> for TestTopic {
            fn from_event(event: &TestEvent) -> Self {
                match event {
                    TestEvent::Temperature(_) | TestEvent::Humidity(_) => TestTopic {
                        name: "IoT".to_string(),
                    },
                    TestEvent::Exit | TestEvent::Sleep(_) => TestTopic {
                        name: "System".to_string(),
                    },
                }
            }
        }

        assert_eq!(
            TestTopic::from_event(&TestEvent::Temperature(22.5)),
            TestTopic {
                name: "IoT".to_string()
            }
        );
        assert_eq!(
            TestTopic::from_event(&TestEvent::Exit),
            TestTopic {
                name: "System".to_string()
            }
        );
    }
}
