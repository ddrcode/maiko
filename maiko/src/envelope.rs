use crate::{ActorId, Event, EventId, Meta};

/// Event plus metadata used by the broker for routing and observability.
///
/// - `event`: the user-defined payload implementing `Event`.
/// - `meta`: `Meta` describing who emitted the event and when.
///   Includes `actor_name` and optional `correlation_id` for linking related events.
#[derive(Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(
    feature = "serde",
    serde(bound(
        serialize = "E: serde::Serialize",
        deserialize = "E: serde::de::DeserializeOwned"
    ))
)]
pub struct Envelope<E: Event> {
    meta: Meta,
    event: E,
}

impl<E: Event> Envelope<E> {
    /// Create a new envelope tagging the event with the given actor name.
    pub fn new(event: E, actor_id: ActorId) -> Self {
        Self {
            meta: Meta::new(actor_id, None),
            event,
        }
    }

    /// Create a new envelope with an explicit correlation id.
    ///
    /// Use this to link child events to a parent or to group related flows.
    pub fn with_correlation(event: E, actor_id: ActorId, correlation_id: EventId) -> Self {
        Self {
            meta: Meta::new(actor_id, Some(correlation_id)),
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

    #[inline]
    pub fn id(&self) -> EventId {
        self.meta.id()
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

// Debug is implemented only when E: Debug.
// This allows Envelope to be used with non-Debug events while still providing
// full debug output when the event type supports it.
impl<E: Event + std::fmt::Debug> std::fmt::Debug for Envelope<E> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Envelope")
            .field("id", &self.meta.id())
            .field("sender", &self.meta.actor_name())
            .field("event", &self.event)
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Clone, Debug)]
    #[allow(unused)]
    struct TestEvent(i32);
    impl Event for TestEvent {}

    #[test]
    fn envelope_debug() {
        let envelope = Envelope::new(TestEvent(42), "test-actor");
        let debug_str = format!("{:?}", envelope);

        assert!(debug_str.contains("TestEvent"));
        assert!(debug_str.contains("42"));
        assert!(debug_str.contains("test-actor"));
    }
}
