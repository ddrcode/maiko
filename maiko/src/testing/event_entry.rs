use std::sync::Arc;

use crate::{ActorHandle, Envelope, Event, EventId, Meta, Topic};

/// A record of an event delivery from one actor to another.
///
/// Each `EventEntry` represents a single delivery: the same event sent to
/// multiple actors produces multiple entries (one per recipient).
///
/// # Fields
///
/// - `event`: The envelope containing the event payload and metadata
/// - `topic`: The topic under which this event was routed
/// - `actor_name`: The name of the receiving actor
#[derive(Debug, Clone)]
pub struct EventEntry<E: Event, T: Topic<E>> {
    pub(crate) event: Arc<Envelope<E>>,
    pub(crate) topic: T,
    pub(crate) actor_name: Arc<str>,
}

impl<E: Event, T: Topic<E>> EventEntry<E, T> {
    pub(crate) fn new(event: Arc<Envelope<E>>, topic: T, actor_name: Arc<str>) -> Self {
        Self {
            event,
            topic,
            actor_name,
        }
    }

    /// Returns the unique ID of this event.
    #[inline]
    pub fn id(&self) -> EventId {
        self.event.id()
    }

    /// Returns a reference to the event payload.
    #[inline]
    pub fn payload(&self) -> &E {
        self.event.event()
    }

    /// Returns the event metadata (sender, timestamp, correlation).
    #[inline]
    pub fn meta(&self) -> &Meta {
        self.event.meta()
    }

    /// Returns the topic this event was routed under.
    #[inline]
    pub fn topic(&self) -> &T {
        &self.topic
    }

    /// Returns the name of the actor that sent this event.
    #[inline]
    pub fn sender(&self) -> &str {
        self.meta().actor_name()
    }

    /// Returns the name of the actor that received this event.
    #[inline]
    pub fn receiver(&self) -> &str {
        &self.actor_name
    }

    /// Returns true if this event was received by the specified actor.
    #[inline]
    pub(crate) fn receiver_actor_eq(&self, actor_handle: &ActorHandle) -> bool {
        self.actor_name == actor_handle.name
    }

    /// Returns true if this event was sent by the specified actor.
    #[inline]
    pub(crate) fn sender_actor_eq(&self, actor_handle: &ActorHandle) -> bool {
        self.meta().actor_name == actor_handle.name
    }
}
