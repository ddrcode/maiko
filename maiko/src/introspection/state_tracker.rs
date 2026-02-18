use std::sync::Arc;

use crate::{ActorId, Envelope, Event, Topic};
use crate::monitoring::Monitor;

use super::actor_info::ActorStatus;
use super::shared_state::SharedState;

/// A [`Monitor`] implementation that tracks actor state for introspection.
///
/// Observes lifecycle callbacks and updates the shared introspection state:
/// - `on_event_delivered` → status becomes `Processing`
/// - `on_event_handled` → status becomes `Idle`, events_handled incremented
/// - `on_error` → error_count incremented
/// - `on_actor_stop` → status becomes `Stopped`
pub(crate) struct StateTracker<E: Event> {
    state: Arc<SharedState<E>>,
}

impl<E: Event> StateTracker<E> {
    pub fn new(state: Arc<SharedState<E>>) -> Self {
        Self { state }
    }
}

impl<E: Event, T: Topic<E>> Monitor<E, T> for StateTracker<E> {
    fn on_event_delivered(&self, _envelope: &Envelope<E>, _topic: &T, receiver: &ActorId) {
        self.state.set_status(receiver, ActorStatus::Processing);
    }

    fn on_event_handled(&self, _envelope: &Envelope<E>, _topic: &T, receiver: &ActorId) {
        self.state.set_status(receiver, ActorStatus::Idle);
        self.state.increment_handled(receiver);
    }

    fn on_error(&self, _err: &str, actor_id: &ActorId) {
        self.state.increment_errors(actor_id);
    }

    fn on_actor_stop(&self, actor_id: &ActorId) {
        self.state.set_status(actor_id, ActorStatus::Stopped);
    }
}
