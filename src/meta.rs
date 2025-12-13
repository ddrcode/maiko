use std::{sync::Arc, time::SystemTime};

use uuid::Uuid;

/// Metadata attached to every event envelope.
///
/// - `id`: unique identifier for the envelope.
/// - `timestamp`: creation time in nanoseconds since Unix epoch (truncated to `u64`).
/// - `sender`: actor name emitting the event.
#[derive(Debug, Clone)]
pub struct Meta {
    id: u128,
    timestamp: u64,
    sender: Arc<str>,
    correlation_id: Option<u128>,
}

impl Meta {
    /// Construct metadata for a given sender.
    pub fn new(sender: &str) -> Self {
        Self {
            id: Uuid::new_v4().as_u128(),
            timestamp: SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .expect("SystemTime before Unix epoch")
                .as_nanos() as u64,
            sender: Arc::from(sender),
            correlation_id: None,
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

    /// Name of the event emitter.
    pub fn sender(&self) -> &str {
        &self.sender
    }

    pub fn correlation_id(&self) -> Option<u128> {
        self.correlation_id
    }

    pub fn set_correlation_id(&mut self, correlation_id: u128) {
        self.correlation_id = Some(correlation_id);
    }
}
