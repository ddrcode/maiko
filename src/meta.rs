use std::{sync::Arc, time::SystemTime};

use uuid::Uuid;

/// Metadata attached to every event envelope.
///
/// - `id`: unique identifier for the envelope.
/// - `timestamp`: creation time in nanoseconds since Unix epoch (truncated to `u64`).
/// - `sender_name`: actor name emitting the event.
/// - `correlation_id`: optional id to link related events together.  Useful for
///   tracing and debugging event flows.
///
/// There is no logic at Maiko built aroung the `correlation_id`, so the value doesn't
/// have any special meaning to the runtime.  It's up to the user to set and interpret it.
/// For example, an actor may choose to set the `correlation_id` of child events, but
/// it may also have another meaning in a different context.
#[derive(Debug, Clone)]
pub struct Meta {
    id: u128,
    timestamp: u64,
    sender_name: Arc<str>,
    correlation_id: Option<u128>,
}

impl Meta {
    /// Construct metadata for a given sender (actor) and optional correlation id.
    pub fn new(sender_name: Arc<str>, correlation_id: Option<u128>) -> Self {
        Self {
            id: Uuid::new_v4().as_u128(),
            timestamp: SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .expect("SystemTime before Unix epoch")
                .as_nanos() as u64,
            sender_name,
            correlation_id,
        }
    }

    /// Unique identifier for this envelope.
    pub fn id(&self) -> u128 {
        self.id
    }

    /// Timestamp in nanoseconds since Unix epoch (u64 truncation).
    pub fn timestamp(&self) -> u64 {
        self.timestamp
    }

    /// Name of actor that sent the event.
    pub fn sender_name(&self) -> &str {
        self.sender_name.as_ref()
    }

    /// Optional value of correlation data.
    /// It might by a parent event id, but it's up to the user to define its meaning.
    pub fn correlation_id(&self) -> Option<u128> {
        self.correlation_id
    }
}
