use crate::{Event, Topic, testing::EventEntry};

pub enum TestEvent<E: Event, T: Topic<E>> {
    Event(EventEntry<E, T>),
    Exit,
    Reset,
}
