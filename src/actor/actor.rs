use super::ActorContext;
use crate::Result;
use crate::event::Event;
use std::cell::RefCell;
use tokio::task_local;

task_local! {
    static ACTOR_CONTEXT: RefCell<Option<dyn ActorContext<E: Event> + 'static>>;
}

/// Core actor behavior
pub trait Actor<E: Event>: Send {
    /// Handle an incoming event
    async fn handle(&mut self, event: E) -> Result<()>;
}

/// Extension methods available to all actors
pub trait ActorExt<E: Event>: Actor<E> {
    /// Get the current actor's context
    fn context(&self) -> &impl ActorContext<E> {
        ACTOR_CONTEXT.with(|ctx| {
            ctx.borrow()
                .as_ref()
                .expect("Actor accessed outside of context")
        })
    }

    /// Send an event through the actor's broker
    async fn send(&self, event: E) -> Result<()> {
        self.context().send(event).await
    }

    /// Get the actor's name
    fn name(&self) -> &str {
        self.context().name()
    }
}

impl<T, E> ActorExt<E> for T
where
    T: Actor<E>,
    E: Event,
{
}
