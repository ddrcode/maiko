use crate::{
    Actor, ActorConfig, ActorId, Context, Event, Result, Subscribe, Supervisor, Topic,
    internal::Subscription,
};

pub struct ActorBuilder<'a, E: Event, T: Topic<E>, A: Actor<Event = E>> {
    supervisor: &'a mut Supervisor<E, T>,
    actor: A,
    ctx: Context<A::Event>,
    config: ActorConfig,
    topics: Subscription<T>,
}

impl<'a, E: Event, T: Topic<E>, A: Actor<Event = E>> ActorBuilder<'a, E, T, A> {
    pub(crate) fn new(
        supervisor: &'a mut Supervisor<E, T>,
        actor: A,
        ctx: Context<A::Event>,
    ) -> Self {
        Self {
            supervisor,
            ctx,
            actor,
            config: ActorConfig::default(),
            topics: Subscription::None,
        }
    }

    pub fn with_topics<S>(mut self, topics: T) -> Self
    where
        S: Into<Subscribe<E, T>>,
    {
        self.topics = Into::<Subscribe<E, T>>::into(topics).0;
        self
    }

    pub fn with_config(mut self, config: ActorConfig) -> Self {
        self.config = config;
        self
    }

    pub fn build(self) -> Result<ActorId> {
        self.supervisor
            .register_actor(self.ctx, self.actor, self.topics, self.config)
    }
}
