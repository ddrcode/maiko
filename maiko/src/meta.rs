use std::time::SystemTime;

use uuid::Uuid;

use crate::{ActorId, EventId};

/// Metadata attached to every event envelope.
///
/// - `id`: unique identifier for the envelope.
/// - `timestamp`: creation time in nanoseconds since Unix epoch (truncated to `u64`).
/// - `actor_name`: actor name emitting the event.
/// - `correlation_id`: optional id to link related events together.  Useful for
///   tracing and debugging event flows.
///
/// There is no logic at Maiko built around the `correlation_id`, so the value doesn't
/// have any special meaning to the runtime.  It's up to the user to set and interpret it.
/// For example, an actor may choose to set the `correlation_id` of child events, but
/// it may also have another meaning in a different context.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Meta {
    id: EventId,
    timestamp: u64,
    pub(crate) actor_id: ActorId,
    correlation_id: Option<EventId>,
}

impl Meta {
    /// Construct metadata for a given actor name and optional correlation id.
    ///
    /// # Panics
    ///
    /// Panics if the system clock is set before the Unix epoch.
    pub fn new(actor_id: ActorId, correlation_id: Option<EventId>) -> Self {
        Self {
            id: Uuid::new_v4().as_u128(),
            timestamp: SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .expect("SystemTime before Unix epoch")
                .as_nanos() as u64,
            actor_id,
            correlation_id,
        }
    }

    /// Unique identifier for this envelope.
    pub fn id(&self) -> EventId {
        self.id
    }

    /// Timestamp in nanoseconds since Unix epoch (u64 truncation).
    pub fn timestamp(&self) -> u64 {
        self.timestamp
    }

    /// Name of actor that sent the event.
    pub fn actor_name(&self) -> &str {
        self.actor_id.name()
    }

    pub fn actor_id(&self) -> &ActorId {
        &self.actor_id
    }

    /// Optional value of correlation data.
    /// It might by a parent event id, but it's up to the user to define its meaning.
    pub fn correlation_id(&self) -> Option<EventId> {
        self.correlation_id
    }
}
