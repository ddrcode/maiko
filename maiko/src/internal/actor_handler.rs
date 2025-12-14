use std::sync::{Arc, atomic::Ordering};

use tokio::sync::mpsc::Receiver;

use crate::{Actor, Context, Envelope, Result};

pub(crate) struct ActorHandler<A: Actor> {
    pub(crate) actor: A,
    pub(crate) receiver: Receiver<Arc<Envelope<A::Event>>>,
    pub(crate) ctx: Context<A::Event>,
    pub(crate) max_events_per_tick: usize,
}

impl<A: Actor> ActorHandler<A> {
    pub async fn run(&mut self) -> Result<()> {
        self.actor.on_start().await?;
        let token = self.ctx.cancel_token.clone();
        while self.ctx.alive.load(Ordering::Acquire) {
            let mut cnt = 0;
            while let Ok(event) = self.receiver.try_recv() {
                if let Err(e) = self.actor.handle(&event.event, &event.meta).await {
                    self.actor.on_error(e)?;
                }
                cnt += 1;
                if cnt == self.max_events_per_tick {
                    break;
                }
            }
            if cnt > 0 {
                tokio::task::yield_now().await;
            }

            if let Err(e) = self.actor.tick().await {
                self.actor.on_error(e)?;
            }

            if token.is_cancelled() {
                self.ctx.stop();
                break;
            }
        }

        self.actor.on_shutdown().await
    }
}
