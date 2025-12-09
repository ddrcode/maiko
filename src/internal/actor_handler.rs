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
        self.actor.on_start(&self.ctx).await?;
        let token = self.cancel_token.clone();
        while self.ctx.alive.load(Ordering::Relaxed) {
            select! {
                _ = token.cancelled() => break,
                Some(event) = self.receiver.recv() => {
                    if let Some(out) = self.actor.handle(&event.event, &event.meta).await? {
                        self.ctx.send(out).await?;
                    }
                },
                r = self.actor.tick(&self.ctx) => r?
            }
        }
        self.cancel_token.cancel();
        self.actor.on_shutdown().await
    }
}
