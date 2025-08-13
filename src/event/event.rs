use super::Topic;

/// An event that can be sent between actors
pub trait Event: Clone + Send + Sync + 'static {
    /// The topic type this event uses for routing
    type Topic: Topic;

    /// Returns the topic this event should be routed to
    fn topic(&self) -> Self::Topic;
}