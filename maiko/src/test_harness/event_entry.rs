use std::sync::Arc;

use crate::{Envelope, Event, Topic};

pub struct EventEntry<E: Event, T: Topic<E>> {
    event: Arc<Envelope<E>>,
    topic: T,
}

impl<E: Event, T: Topic<E>> EventEntry<E, T> {
    pub fn new(event: Arc<Envelope<E>>, topic: T) -> Self {
        Self { event, topic }
    }
}
