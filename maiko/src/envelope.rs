use std::sync::Arc;

use crate::{Event, Meta};

/// Event plus metadata used by the broker for routing and observability.
///
/// - `event`: the user-defined payload implementing `Event`.
/// - `meta`: `Meta` describing who emitted the event and when.
///   Includes `actor_name` and optional `correlation_id` for linking related events.
#[derive(Debug, Clone)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(bound = "")
)]
pub struct Envelope<E: Event> {
    meta: Meta,
    event: E,
}

impl<E: Event> Envelope<E> {
    /// Create a new envelope tagging the event with the given actor name.
    pub fn new<N>(event: E, actor_name: N) -> Self
    where
        N: Into<Arc<str>>,
    {
        Self {
            meta: Meta::new(actor_name.into(), None),
            event,
        }
    }

    /// Create a new envelope with an explicit correlation id.
    ///
    /// Use this to link child events to a parent or to group related flows.
    pub fn with_correlation<N>(event: E, actor_name: N, correlation_id: u128) -> Self
    where
        N: Into<Arc<str>>,
    {
        Self {
            meta: Meta::new(actor_name.into(), Some(correlation_id)),
            event,
        }
    }

    /// Returns a reference to the event payload.
    ///
    /// This is a convenience method for pattern matching. For method calls,
    /// you can also use `Deref` (e.g., `envelope.some_event_method()`).
    ///
    /// # Example
    ///
    /// ```ignore
    /// match envelope.event() {
    ///     MyEvent::Foo(x) => handle_foo(x),
    ///     MyEvent::Bar => handle_bar(),
    /// }
    /// ```
    #[inline]
    pub fn event(&self) -> &E {
        &self.event
    }

    #[inline]
    pub fn meta(&self) -> &Meta {
        &self.meta
    }
}

impl<E: Event> From<(&E, &Meta)> for Envelope<E> {
    fn from((event, meta): (&E, &Meta)) -> Self {
        Envelope::<E> {
            meta: meta.clone(),
            event: event.clone(),
        }
    }
}

impl<E: Event> std::ops::Deref for Envelope<E> {
    type Target = E;
    fn deref(&self) -> &E {
        &self.event
    }
}
