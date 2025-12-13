use std::sync::Arc;

use crate::{Event, Meta};

/// Event plus metadata used by the broker for routing and observability.
#[derive(Debug, Clone)]
pub struct Envelope<E: Event> {
    pub meta: Meta,
    pub event: E,
}

impl<E: Event> Envelope<E> {
    /// Create a new envelope tagging the event with the given sender name.
    pub fn new<N>(event: E, sender_name: N) -> Self
    where
        N: Into<Arc<str>>,
    {
        Self {
            meta: Meta::new(sender_name.into(), None),
            event,
        }
    }

    pub fn with_correlation_id<N>(event: E, sender_name: N, correlation_id: u128) -> Self
    where
        N: Into<Arc<str>>,
    {
        Self {
            meta: Meta::new(sender_name.into(), Some(correlation_id)),
            event,
        }
    }
}
