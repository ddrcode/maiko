use std::sync::{Arc, mpsc::Receiver};

use tokio::select;
use tokio_util::sync::CancellationToken;

use crate::{Actor, Envelope, Event, Result};

pub struct ActorHandler<A: Actor> {
    actor: A,
    receiver: Receiver<Envelope<A::Event>>,
    cancel_token: Arc<CancellationToken>,
}

impl<A: Actor> ActorHandler<A> {
    async fn run(&mut self) -> Result<()> {
        self.actor.start().await?;
        let mut rx = self.receiver;
        let token = self.cancel_token.clone();
        loop {
            select! {
                _ = token.cancelled() => break,
                Some(event) = rx.recv() => {
            //         if let Some(out) = self.handle(&event.event, &event.meta).await? {
            // self.actor.send(out).await?;
            //         }
                },
                r = self.actor.tick() => r?
            }
        }
        self.actor.shutdown().await
    }
}
