use std::collections::HashSet;

use crate::{Actor, Context, Error, Event, Result, Supervisor, Topic};

pub struct ActorBuilder<'a, E: Event, T: Topic<E>, A: Actor<Event = E>> {
    supervisor: &'a mut Supervisor<E, T>,
    context: Context<E>,
    topics: HashSet<T>,
    actor: Option<A>,
}

impl<'a, E: Event, T: Topic<E>, A: Actor<Event = E>> ActorBuilder<'a, E, T, A> {
    pub(crate) fn new(supervisor: &'a mut Supervisor<E, T>, context: Context<E>) -> Self {
        Self {
            supervisor,
            context,
            topics: HashSet::new(),
            actor: None,
        }
    }

    pub fn actor<F>(mut self, actor_factory: F) -> Self
    where
        F: FnOnce(Context<E>) -> A,
    {
        let actor = actor_factory(self.context.clone());
        self.actor = Some(actor);
        self
    }

    pub fn topics(mut self, topics: &[T]) -> Self {
        self.topics = HashSet::from_iter(topics.iter().cloned());
        self
    }

    pub fn add_topic(mut self, topic: T) -> Self {
        self.topics.insert(topic);
        self
    }

    pub fn build(mut self) -> Result<()> {
        let actor = self
            .actor
            .take()
            .ok_or_else(|| Error::ActorBuilderError("Actor not provided.".into()))?;
        let topics = std::mem::take(&mut self.topics);
        let ctx = self.context;
        self.supervisor.register_actor(ctx, actor, topics)
    }
}
