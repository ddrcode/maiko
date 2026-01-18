use std::sync::Arc;

use crate::{Envelope, Event, Topic};

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
}
