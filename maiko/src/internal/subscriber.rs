use std::{collections::HashSet, sync::Arc};

use tokio::sync::mpsc::Sender;

use crate::{Envelope, Event, Topic};

#[derive(Debug)]
pub(crate) struct Subscriber<E: Event, T: Topic<E>> {
    pub name: Arc<str>,
    pub topics: HashSet<T>,
    pub sender: Sender<Arc<Envelope<E>>>,
}

impl<E: Event, T: Topic<E>> Subscriber<E, T> {
    pub fn new(
        name: Arc<str>,
        topics: HashSet<T>,
        sender: Sender<Arc<Envelope<E>>>,
    ) -> Subscriber<E, T> {
        Subscriber {
            name,
            topics,
            sender,
        }
    }
}
