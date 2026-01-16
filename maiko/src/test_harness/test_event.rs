use crate::{Event, Topic, test_harness::EventEntry};

pub enum TestEvent<E: Event, T: Topic<E>> {
    Event(EventEntry<E, T>),
    Exit,
    Reset,
}
