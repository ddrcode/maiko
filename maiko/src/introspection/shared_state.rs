use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use tokio::sync::mpsc::Sender;

use crate::{ActorId, Envelope, Event};

use super::actor_info::{ActorInfo, ActorStatus};

/// Per-actor state entry stored in the shared introspection state.
pub(crate) struct ActorEntry<E: Event> {
    pub status: ActorStatus,
    pub events_handled: u64,
    pub error_count: u64,
    /// Cloned sender pointing to the actor's mailbox channel.
    /// Used to query `capacity()` / `max_capacity()` for queue depth.
    pub sender: Sender<Arc<Envelope<E>>>,
}

/// Internal shared state for the introspection system.
///
/// Written to by both:
/// - `Supervisor::register_actor` — stores the mailbox `Sender` for queue depth queries
/// - `StateTracker` monitor — updates status, event count, and error count via observer callbacks
///
/// Read by `Introspector` to produce `ActorInfo` and `SystemSnapshot`.
pub(crate) struct SharedState<E: Event> {
    actors: Mutex<HashMap<ActorId, ActorEntry<E>>>,
}

impl<E: Event> SharedState<E> {
    pub fn new() -> Self {
        Self {
            actors: Mutex::new(HashMap::new()),
        }
    }

    /// Register a new actor with its mailbox sender.
    ///
    /// Called by `Supervisor::register_actor` when the `introspection` feature is enabled.
    pub fn register_actor(&self, actor_id: ActorId, sender: Sender<Arc<Envelope<E>>>) {
        let mut actors = self.actors.lock().expect("SharedState lock poisoned");
        actors.insert(
            actor_id,
            ActorEntry {
                status: ActorStatus::Registered,
                events_handled: 0,
                error_count: 0,
                sender,
            },
        );
    }

    /// Update the status of an actor.
    pub fn set_status(&self, actor_id: &ActorId, status: ActorStatus) {
        let mut actors = self.actors.lock().expect("SharedState lock poisoned");
        if let Some(entry) = actors.get_mut(actor_id) {
            entry.status = status;
        }
    }

    /// Increment the events_handled counter for an actor.
    pub fn increment_handled(&self, actor_id: &ActorId) {
        let mut actors = self.actors.lock().expect("SharedState lock poisoned");
        if let Some(entry) = actors.get_mut(actor_id) {
            entry.events_handled += 1;
        }
    }

    /// Increment the error_count counter for an actor.
    pub fn increment_errors(&self, actor_id: &ActorId) {
        let mut actors = self.actors.lock().expect("SharedState lock poisoned");
        if let Some(entry) = actors.get_mut(actor_id) {
            entry.error_count += 1;
        }
    }

    /// Get info for all registered actors with live queue depth.
    pub fn list_actors(&self) -> Vec<ActorInfo> {
        let actors = self.actors.lock().expect("SharedState lock poisoned");
        actors
            .iter()
            .map(|(id, entry)| Self::build_actor_info(id, entry))
            .collect()
    }

    /// Get info for a specific actor.
    pub fn actor_info(&self, actor_id: &ActorId) -> Option<ActorInfo> {
        let actors = self.actors.lock().expect("SharedState lock poisoned");
        actors
            .get(actor_id)
            .map(|entry| Self::build_actor_info(actor_id, entry))
    }

    /// Build an `ActorInfo` from an entry, querying live queue depth.
    fn build_actor_info(actor_id: &ActorId, entry: &ActorEntry<E>) -> ActorInfo {
        let max_capacity = entry.sender.max_capacity();
        let remaining_capacity = entry.sender.capacity();
        ActorInfo {
            actor_id: actor_id.clone(),
            status: entry.status.clone(),
            mailbox_depth: max_capacity - remaining_capacity,
            mailbox_capacity: max_capacity,
            events_handled: entry.events_handled,
            error_count: entry.error_count,
        }
    }
}
