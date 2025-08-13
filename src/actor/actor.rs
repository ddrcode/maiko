use std::cell::RefCell;
use tokio::task_local;
use crate::event::{Event};
use super::{ActorContext, DefaultActorContext};

task_local! {
    static ACTOR_CONTEXT: RefCell<Option<DefaultActorContext>>;
}

pub struct ActorBase<E: Event> {
    context: ActorContext<E>,
}

pub trait Actor<E: Event>: Send {
    fn with_context<F, R>(&self, f: F) -> R
    where
        F: FnOnce(&ActorContext<E>) -> R;

    async fn handle(&mut self, event: E) -> Result<()>;
}

impl<E: Event> ActorBase<E> {
    pub fn context(&self) -> &ActorContext<E> {
        &self.context
    }
}

