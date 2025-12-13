/// Marker trait for events processed by Maiko.
///
/// Implement this for your event type (often an enum). Events must be
/// `Send + Clone` because they cross task boundaries and are routed via
/// channels. The broker wraps events in an `Envelope` carrying metadata
/// such as sender name and timestamp.
pub trait Event: Send + Clone {}
