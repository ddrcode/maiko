use std::sync::Arc;

use crate::{ActorHandle, Envelope, Event, EventId, Meta, Topic};

#[derive(Debug, Clone)]
pub struct EventEntry<E: Event, T: Topic<E>> {
    pub(crate) event: Arc<Envelope<E>>,
    pub(crate) topic: T,
    pub(crate) actor_name: Arc<str>,
}

impl<E: Event, T: Topic<E>> EventEntry<E, T> {
    pub fn new(event: Arc<Envelope<E>>, topic: T, actor_name: Arc<str>) -> Self {
        Self {
            event,
            topic,
            actor_name,
        }
    }

    #[inline]
    pub fn id(&self) -> EventId {
        self.event.id()
    }

    pub fn payload(&self) -> &E {
        self.event.event()
    }

    pub fn meta(&self) -> &Meta {
        self.event.meta()
    }

    #[inline]
    pub fn receiver_actor_eq(&self, actor_handle: &ActorHandle) -> bool {
        self.actor_name == actor_handle.name
    }

    #[inline]
    pub fn sender_actor_eq(&self, actor_handle: &ActorHandle) -> bool {
        self.event.meta().actor_name == actor_handle.name
    }
}
