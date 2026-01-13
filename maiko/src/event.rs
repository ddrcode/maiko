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
#[cfg(not(feature = "serde"))]
pub trait Event: Send + Sync + Clone + 'static {}
#[cfg(feature = "serde")]
pub trait Event:
    Send + Sync + Clone + serde::Serialize + for<'a> serde::Deserialize<'a> + 'static
{
}
