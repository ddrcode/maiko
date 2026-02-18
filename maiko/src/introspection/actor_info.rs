use std::time::SystemTime;

use crate::ActorId;

/// Runtime status of an actor.
///
/// Tracks the lifecycle state as observed by the introspection system.
/// Status transitions follow this order:
///
/// ```text
/// Registered → Idle → Processing → Idle → ... → Stopped
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum ActorStatus {
    /// Actor has been added to the supervisor but the supervisor has not started yet.
    Registered,
    /// Actor is idle, waiting for events.
    Idle,
    /// Actor is currently processing an event inside `handle_event()`.
    Processing,
    /// Actor has exited (either normally or due to an error).
    Stopped,
}

impl std::fmt::Display for ActorStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ActorStatus::Registered => write!(f, "registered"),
            ActorStatus::Idle => write!(f, "idle"),
            ActorStatus::Processing => write!(f, "processing"),
            ActorStatus::Stopped => write!(f, "stopped"),
        }
    }
}

/// Runtime information about a single actor.
///
/// Obtained from [`Introspector::list_actors`](super::Introspector::list_actors)
/// or [`Introspector::actor_info`](super::Introspector::actor_info).
///
/// # Queue Depth
///
/// `mailbox_depth` reflects the number of pending events in the actor's
/// mailbox at the instant the snapshot was taken. Because actors run
/// concurrently, this value may have changed by the time you read it.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ActorInfo {
    /// The actor's unique identifier.
    pub actor_id: ActorId,
    /// Current lifecycle status.
    pub status: ActorStatus,
    /// Number of events currently pending in the actor's mailbox.
    ///
    /// Computed as `max_capacity - capacity` on the underlying channel sender.
    /// This is a point-in-time snapshot and may be stale by the time you read it.
    pub mailbox_depth: usize,
    /// Total capacity of the actor's mailbox channel.
    pub mailbox_capacity: usize,
    /// Total number of events this actor has processed (via `handle_event`).
    pub events_handled: u64,
    /// Total number of errors encountered by this actor.
    pub error_count: u64,
}

/// Point-in-time snapshot of the entire actor system.
///
/// Contains a timestamp and information about every registered actor.
///
/// # Example
///
/// ```ignore
/// let snapshot = supervisor.introspection().snapshot();
/// for actor in &snapshot.actors {
///     println!("{}: {} (queue: {}/{})",
///         actor.actor_id.name(),
///         actor.status,
///         actor.mailbox_depth,
///         actor.mailbox_capacity,
///     );
/// }
/// ```
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct SystemSnapshot {
    /// Timestamp in nanoseconds since Unix epoch when the snapshot was taken.
    pub timestamp: u64,
    /// Information about all registered actors at the time of the snapshot.
    pub actors: Vec<ActorInfo>,
}

impl SystemSnapshot {
    /// Create a new snapshot with the current timestamp.
    pub(crate) fn new(actors: Vec<ActorInfo>) -> Self {
        let timestamp = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .expect("SystemTime before Unix epoch")
            .as_nanos() as u64;
        Self { timestamp, actors }
    }
}
