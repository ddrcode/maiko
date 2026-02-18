use std::sync::Arc;

use crate::{ActorId, Event};

use super::actor_info::{ActorInfo, SystemSnapshot};
use super::shared_state::SharedState;

/// Handle for querying runtime actor state.
///
/// Provides live introspection into the actor system, including:
/// - Actor listing and individual lookups
/// - Mailbox queue depths (queried live from channel senders)
/// - Event and error counters
/// - Point-in-time system snapshots
///
/// # Example
///
/// ```ignore
/// let introspector = supervisor.introspection();
///
/// // List all actors
/// for actor in introspector.list_actors() {
///     println!("{}: {} (queue: {}/{})",
///         actor.actor_id.name(),
///         actor.status,
///         actor.mailbox_depth,
///         actor.mailbox_capacity,
///     );
/// }
///
/// // Snapshot the entire system
/// let snapshot = introspector.snapshot();
/// println!("System has {} actors at t={}", snapshot.actors.len(), snapshot.timestamp);
/// ```
pub struct Introspector<E: Event> {
    pub(crate) state: Arc<SharedState<E>>,
}

impl<E: Event> Introspector<E> {
    pub(crate) fn new(state: Arc<SharedState<E>>) -> Self {
        Self { state }
    }

    /// List all registered actors with their current status and queue depths.
    ///
    /// Queue depths are queried live from the underlying channel senders
    /// and reflect the state at the instant of the call.
    pub fn list_actors(&self) -> Vec<ActorInfo> {
        self.state.list_actors()
    }

    /// Get runtime info for a specific actor.
    ///
    /// Returns `None` if the actor has not been registered.
    pub fn actor_info(&self, actor_id: &ActorId) -> Option<ActorInfo> {
        self.state.actor_info(actor_id)
    }

    /// Take a point-in-time snapshot of the entire actor system.
    ///
    /// The snapshot includes a timestamp and information about every
    /// registered actor, including live queue depths.
    pub fn snapshot(&self) -> SystemSnapshot {
        SystemSnapshot::new(self.state.list_actors())
    }
}
