use crate::{Event, Meta};

/// Event plus metadata used by the broker for routing and observability.
#[derive(Debug, Clone)]
pub struct Envelope<E: Event> {
    pub meta: Meta,
    pub event: E,
}

impl<E: Event> Envelope<E> {
    /// Create a new envelope tagging the event with the given sender name.
    pub fn new(event: E, sender: &str) -> Self {
        Self {
            meta: Meta::new(sender),
            event,
        }
    }
}
