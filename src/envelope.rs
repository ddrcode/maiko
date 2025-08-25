use crate::{Event, Meta};

pub struct Envelope<E: Event> {
    meta: Meta,
    event: E,
}

impl<E: Event> Envelope<E> {
    pub fn new(event: E) -> Self {
        Self {
            meta: Meta::new(),
            event,
        }
    }
}
