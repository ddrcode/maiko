use std::sync::Arc;
use crate::{Event, EventBroker, Result};

/// Represents the execution context of an actor
pub trait ActorContext<E: Event>: Send + Sync {
    /// Get the actor's name
    fn name(&self) -> &str;
    
    /// Get a reference to the event broker
    fn broker(&self) -> &Arc<EventBroker<E>>;
    
    /// Send an event through the broker
    async fn send(&self, event: E) -> Result<()> {
        self.broker().publish(event).await
    }
}

/// Default implementation of ActorContext
#[derive(Clone)]
pub struct DefaultActorContext<E: Event> {
    name: String,
    broker: Arc<EventBroker<E>>,
}

impl<E: Event> DefaultActorContext<E> {
    pub fn new(name: impl Into<String>, broker: Arc<EventBroker<E>>) -> Self {
        Self {
            name: name.into(),
            broker,
        }
    }
}

impl<E: Event> ActorContext<E> for DefaultActorContext<E> {
    fn name(&self) -> &str {
        &self.name
    }
    
    fn broker(&self) -> &Arc<EventBroker<E>> {
        &self.broker
    }
}