use crate::{Event, Meta};

#[derive(Debug, Clone)]
pub struct Envelope<E: Event> {
    pub meta: Meta,
    pub event: E,
}

impl<E: Event> Envelope<E> {
    pub fn new(event: E, sender: &str) -> Self {
        Self {
            meta: Meta::new(sender),
            event,
        }
    }
}
