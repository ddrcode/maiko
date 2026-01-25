use std::{collections::HashSet, sync::Arc};

use tokio::sync::mpsc::Sender;

use crate::{ActorHandle, Envelope, Event, Topic};

#[derive(Debug)]
pub(crate) struct Subscriber<E: Event, T: Topic<E>> {
    pub handle: ActorHandle,
    pub topics: HashSet<T>,
    pub sender: Sender<Arc<Envelope<E>>>,
}

impl<E: Event, T: Topic<E>> Subscriber<E, T> {
    pub fn new(
        handle: ActorHandle,
        topics: HashSet<T>,
        sender: Sender<Arc<Envelope<E>>>,
    ) -> Subscriber<E, T> {
        Subscriber {
            handle,
            topics,
            sender,
        }
    }
}

impl<E: Event, T: Topic<E>> PartialEq for Subscriber<E, T> {
    fn eq(&self, other: &Self) -> bool {
        self.handle == other.handle
    }
}
