/// Marker trait for events processed by Maiko.
///
/// Implement this for your event type (often an enum). Events must be
/// `Send + Sync + Clone + 'static` because they:
/// - Are wrapped in `Arc<Envelope<E>>` and shared across threads (Sync)
/// - Are sent through channels to spawned tasks (Send + 'static)
/// - Are routed to multiple subscribers (Clone)
///
/// The broker wraps events in an `Envelope` carrying metadata
/// such as sender name and timestamp.
pub trait Event: Send + Sync + Clone + 'static {}
