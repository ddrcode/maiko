use std::sync::{Arc, atomic::Ordering};

use tokio::{select, sync::mpsc::Receiver};
use tokio_util::sync::CancellationToken;

use crate::{Actor, Context, Envelope, Result};

pub(crate) struct ActorHandler<A: Actor> {
    pub(crate) actor: A,
    pub(crate) receiver: Receiver<Envelope<A::Event>>,
    pub(crate) cancel_token: Arc<CancellationToken>,
    pub(crate) ctx: Context<A::Event>,
}

impl<A: Actor> ActorHandler<A> {
    pub async fn run(&mut self) -> Result<()> {
        self.actor.on_start().await?;
        let token = self.cancel_token.clone();
        while self.ctx.alive.load(Ordering::Relaxed) {
            select! {
                biased;
                Some(event) = self.receiver.recv() => {
                    self.actor.handle(&event.event, &event.meta).await?;
                },
                r = self.actor.tick() => r?,
                _ = token.cancelled() => {
                    self.ctx.stop();
                    break;
                }
            }
        }
        self.actor.on_shutdown().await
    }
}
