use std::collections::HashSet;

use crate::{Actor, Context, Error, Event, Result, Supervisor, Topic};

/// Builder for configuring and registering actors with the supervisor.
///
/// Created by calling [`Supervisor::build_actor()`] and finalized with [`.build()`](Self::build).
///
/// # Example
///
/// ```ignore
/// supervisor.build_actor("worker")
///     .actor(|ctx| MyActor::new(ctx))
///     .topics(&[MyTopic::Data])
///     .build()?;
/// ```
///
/// See also: [`Supervisor::add_actor()`] for a simpler API.
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

    /// Set the actor using a factory function that receives the context.
    ///
    /// # Example
    ///
    /// ```ignore
    /// builder.actor(|ctx| MyActor::new(ctx))
    /// ```
    pub fn actor<F>(mut self, actor_factory: F) -> Self
    where
        F: FnOnce(Context<E>) -> A,
    {
        let actor = actor_factory(self.context.clone());
        self.actor = Some(actor);
        self
    }

    /// Set the topics this actor subscribes to.
    ///
    /// Replaces any previously set topics. Use [`add_topic()`](Self::add_topic)
    /// to add topics incrementally.
    ///
    /// # Example
    ///
    /// ```ignore
    /// builder.topics(&[MyTopic::Data, MyTopic::Control])
    /// ```
    pub fn topics(mut self, topics: &[T]) -> Self {
        self.topics = HashSet::from_iter(topics.iter().cloned());
        self
    }

    /// Add a single topic to this actor's subscriptions.
    ///
    /// Can be called multiple times to incrementally build the topic set.
    ///
    /// # Example
    ///
    /// ```ignore
    /// builder
    ///     .add_topic(MyTopic::Data)
    ///     .add_topic(MyTopic::Control)
    /// ```
    pub fn add_topic(mut self, topic: T) -> Self {
        self.topics.insert(topic);
        self
    }

    /// Finalize the builder and register the actor with the supervisor.
    ///
    /// Returns an error if the actor was not provided via [`actor()`](Self::actor).
    ///
    /// # Errors
    ///
    /// - [`Error::ActorBuilderError`] if actor was not provided
    /// - [`Error::BrokerAlreadyStarted`] if supervisor was already started
    /// - [`Error::SubscriberAlreadyExists`] if an actor with the same name exists
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
