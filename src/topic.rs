use crate::event::Event;

pub trait Topic<E: Event> {
    fn from_event(event: &E) -> Self;
}

#[derive(Debug)]
pub struct DefaultTopic;

impl<E: Event> Topic<E> for DefaultTopic {
    fn from_event(_event: &E) -> DefaultTopic {
        DefaultTopic
    }
}

impl std::fmt::Display for DefaultTopic {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "default")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::event::Event;

    #[derive(Debug)]
    struct TestEvent;

    impl Event for TestEvent {}

    #[test]
    fn test_default_topic() {
        let event = TestEvent;
        let result = DefaultTopic::from_event(&event);
        assert_eq!(result.to_string(), "default");
    }

    #[test]
    fn test_topic_as_enum() {
        #[allow(unused)]
        enum TestEvent {
            Temperature(f64),
            Humidity(f64),
            Exit,
            Sleep(u64),
        }
        impl Event for TestEvent {}

        #[derive(Debug, PartialEq)]
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
        #[allow(unused)]
        enum TestEvent {
            Temperature(f64),
            Humidity(f64),
            Exit,
            Sleep(u64),
        }
        impl Event for TestEvent {}

        #[derive(Debug, PartialEq)]
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
