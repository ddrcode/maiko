use tokio::sync::mpsc::Sender;

use crate::{Event, Result};
use std::sync::Arc;

/// Represents the execution context of an actor
pub trait ActorContext<E: Event>: Send + Sync {
    /// Get the actor's name
    fn name(&self) -> &str;

    fn sender(&self) -> &Sender<E>;

    /// Send an event through the broker
    async fn send(&self, event: E) -> Result<()> {
        self.sender().send(event).await.map_err(|_| crate::Error)
    }
}

/// Default implementation of ActorContext
#[derive(Clone)]
pub struct DefaultActorContext<E: Event> {
    name: String,
    sender: Sender<E>,
}

impl<E: Event> DefaultActorContext<E> {
    pub fn new(name: impl Into<String>, sender: Sender<E>) -> Self {
        Self {
            name: name.into(),
            sender,
        }
    }
}

impl<E: Event> ActorContext<E> for DefaultActorContext<E> {
    fn name(&self) -> &str {
        &self.name
    }
    fn sender(&self) -> &Sender<E> {
        &self.sender
    }
}
