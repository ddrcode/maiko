use std::collections::HashSet;

use crate::{Actor, Context, Event, Topic, internal::ActorHandler};

pub struct ActorBuilder<E: Event, T: Topic<E>, A: Actor<Event = E>> {
    pub(crate) topics: HashSet<T>,
    pub(crate) actor: Option<A>,
}

impl<E: Event, T: Topic<E>, A: Actor<Event = E>> ActorBuilder<E, T, A> {
    pub fn new() -> Self {
        Self {
            topics: HashSet::new(),
            actor: None,
        }
    }
    pub fn actor(mut self, actor: A) -> Self {
        self.actor = Some(actor);
        self
    }

    pub fn with_config(mut self) -> Self {
        self
    }
    pub fn with_topics(mut self, topics: &[T]) -> Self {
        self.topics = HashSet::from_iter(topics.iter().cloned());
        self
    }
    pub fn add_topic(mut self, topic: T) -> Self {
        self.topics.insert(topic);
        self
    }
}
