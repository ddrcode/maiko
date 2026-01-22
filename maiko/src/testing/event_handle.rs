use crate::{Event, EventId, Meta, Topic, testing::EventEntry};

pub struct EventHandle<E: Event, T: Topic<E>> {
    entry: EventEntry<E, T>,
}

impl<E: Event, T: Topic<E>> EventHandle<E, T> {
    pub fn new(entry: EventEntry<E, T>) -> Self {
        Self { entry }
    }

    #[inline]
    pub fn id(&self) -> EventId {
        self.entry.event.id()
    }

    pub fn payload(&self) -> &E {
        self.entry.event.event()
    }

    pub fn meta(&self) -> &Meta {
        self.entry.event.meta()
    }
}

impl<E: Event, T: Topic<E>> From<EventHandle<E, T>> for EventId {
    fn from(handle: EventHandle<E, T>) -> EventId {
        handle.id()
    }
}

impl<E: Event, T: Topic<E>> From<EventEntry<E, T>> for EventHandle<E, T> {
    fn from(entry: EventEntry<E, T>) -> Self {
        EventHandle::new(entry)
    }
}

impl<E: Event, T: Topic<E>> From<&EventEntry<E, T>> for EventHandle<E, T> {
    fn from(entry: &EventEntry<E, T>) -> Self {
        EventHandle::new(entry.clone())
    }
}
