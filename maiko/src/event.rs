use std::borrow::Cow;

/// Marker trait for events processed by Maiko.
///
/// Implement this for your event type (often an enum). Events must be
/// `Send + Sync + Clone + 'static` because they:
/// - Are wrapped in `Arc<Envelope<E>>` and shared across threads (Sync)
/// - Cross task boundaries and live in spawned tasks (Send, 'static)
/// - Are routed to multiple subscribers (Clone)
///
/// The broker wraps events in an `Envelope` carrying metadata
/// such as sender name and timestamp.
///
/// # Event Names
///
/// The `name()` method returns a human-readable name for the event,
/// typically the enum variant name. This is used for logging, monitoring,
/// and diagram generation. The default implementation returns the full
/// type name via `std::any::type_name`.
///
/// When using `#[derive(Event)]` on an enum, `name()` is automatically
/// implemented to return the variant name (e.g., "SensorReading").
pub trait Event: Send + Sync + Clone + 'static {
    /// Returns a human-readable name for this event.
    ///
    /// For enum events, this is typically the variant name (e.g., "SensorReading").
    /// The default implementation returns the type name via `std::any::type_name`.
    fn name(&self) -> Cow<'static, str> {
        Cow::Borrowed(std::any::type_name::<Self>())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Note: #[derive(Event)] can't be tested here because the macro generates
    // `impl maiko::Event` which doesn't resolve within the maiko crate itself.
    // The derive macro is tested via examples (e.g., pingpong).

    #[derive(Clone)]
    struct ManualEvent;

    impl Event for ManualEvent {}

    #[test]
    fn test_manual_event_default_name() {
        // Default uses type_name, which includes module path
        assert!(ManualEvent.name().ends_with("ManualEvent"));
    }

    #[derive(Clone)]
    #[allow(dead_code)]
    enum EnumEvent {
        Foo,
        Bar(i32),
    }

    impl Event for EnumEvent {
        fn name(&self) -> Cow<'static, str> {
            Cow::Borrowed(match self {
                EnumEvent::Foo => "Foo",
                EnumEvent::Bar(_) => "Bar",
            })
        }
    }

    #[test]
    fn test_manual_enum_event_name() {
        assert_eq!(EnumEvent::Foo.name(), "Foo");
        assert_eq!(EnumEvent::Bar(42).name(), "Bar");
    }
}
