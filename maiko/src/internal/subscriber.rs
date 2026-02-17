use std::{hash, sync::Arc};

use tokio::sync::mpsc::Sender;

use crate::{ActorId, Envelope, Event, Topic, internal::Subscription};

#[derive(Debug)]
pub(crate) struct Subscriber<E, T: Eq + hash::Hash> {
    pub actor_id: ActorId,
    pub topics: Subscription<T>,
    pub sender: Sender<Arc<Envelope<E>>>,
}

impl<E, T: Eq + hash::Hash> Subscriber<E, T> {
    pub fn new(
        actor_id: ActorId,
        topics: Subscription<T>,
        sender: Sender<Arc<Envelope<E>>>,
    ) -> Subscriber<E, T> {
        Subscriber {
            actor_id,
            topics,
            sender,
        }
    }

    pub fn is_closed(&self) -> bool {
        self.sender.is_closed()
    }
}

impl<E, T: Eq + hash::Hash> PartialEq for Subscriber<E, T> {
    fn eq(&self, other: &Self) -> bool {
        self.actor_id == other.actor_id
    }
}

impl<E: Event, T: Topic<E>> Eq for Subscriber<E, T> {}
