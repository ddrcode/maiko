use std::sync::Arc;

use tokio::sync::mpsc::Sender;

use crate::{ActorId, Envelope, Event, Topic, internal::Subscription};

#[derive(Debug)]
pub(crate) struct Subscriber<E: Event, T: Topic<E>> {
    pub actor_id: ActorId,
    pub topics: Subscription<T>,
    pub sender: Sender<Arc<Envelope<E>>>,
}

impl<E: Event, T: Topic<E>> Subscriber<E, T> {
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
}

impl<E: Event, T: Topic<E>> PartialEq for Subscriber<E, T> {
    fn eq(&self, other: &Self) -> bool {
        self.actor_id == other.actor_id
    }
}
