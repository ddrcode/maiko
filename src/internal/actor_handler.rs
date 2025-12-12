use std::sync::atomic::Ordering;

use tokio::sync::mpsc::Receiver;

use crate::{Actor, Context, Envelope, Result};

pub(crate) struct ActorHandler<A: Actor> {
    pub(crate) actor: A,
    pub(crate) receiver: Receiver<Envelope<A::Event>>,
    pub(crate) ctx: Context<A::Event>,
}

impl<A: Actor> ActorHandler<A> {
    pub async fn run(&mut self) -> Result<()> {
        self.actor.on_start().await?;
        let token = self.ctx.cancel_token.clone();
        while self.ctx.alive.load(Ordering::Acquire) {
            if let Ok(event) = self.receiver.try_recv() {
                if let Err(e) = self.actor.handle(&event.event, &event.meta).await
                    && self.actor.on_error(&e)
                {
                    return Err(e);
                }
            }

            if let Err(e) = self.actor.tick().await
                && self.actor.on_error(&e)
            {
                return Err(e);
            }

            if token.is_cancelled() {
                self.ctx.stop();
                break;
            }

            tokio::task::yield_now().await;
        }
        self.actor.on_shutdown().await
    }
}
