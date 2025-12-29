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
/// When the `serde` feature is enabled, events must also implement
/// `Serialize` and `Deserialize` for IPC and persistence.
#[cfg_attr(
    feature = "serde",
    doc = r"
When the `serde` feature is enabled, this trait is equivalent to:
```rust
# use serde::{Serialize, Deserialize};
pub trait Event:
    Send + Sync + Clone + 'static + Serialize + for<'a> Deserialize<'a>
{
}
```
"
)]
pub trait Event: Send + Sync + Clone + 'static + private::Sealed {}

mod private {
    use super::*;

    // This trait is sealed and cannot be implemented for types outside this crate.
    pub trait Sealed: Sized {}

    #[cfg(not(feature = "serde"))]
    impl<T> Sealed for T where T: Event + Send + Sync + Clone + 'static {}

    #[cfg(feature = "serde")]
    impl<T> Sealed for T where
        T: Event
            + Send
            + Sync
            + Clone
            + 'static
            + serde::Serialize
            + for<'a> serde::Deserialize<'a>
    {
    }
}
