use std::sync::{Arc, atomic::Ordering};

use tokio::{select, sync::mpsc::Receiver};
use tokio_util::sync::CancellationToken;

use crate::{Actor, Context, Envelope, Result};

pub(crate) struct ActorHandler<A: Actor> {
    pub(crate) actor: A,
    pub(crate) receiver: Receiver<Arc<Envelope<A::Event>>>,
    pub(crate) ctx: Context<A::Event>,
    pub(crate) max_events_per_tick: usize,
    pub(crate) cancel_token: Arc<CancellationToken>,
}

impl<A: Actor> ActorHandler<A> {
    pub async fn run(&mut self) -> Result<()> {
        self.actor.on_start().await?;
        let token = self.cancel_token.clone();
        while self.ctx.alive {
            select! {
                biased;

                _ = token.cancelled() => {
                    self.ctx.stop();
                    break;
                },

                Some(event) = self.receiver.recv() => {
                    let res = self.actor.handle(&event.event, &event.meta).await;
                    self.handle_error(res)?;

                    let mut cnt = 1;
                    while let Ok(event) = self.receiver.try_recv() {
                        let res = self.actor.handle(&event.event, &event.meta).await;
                        self.handle_error(res)?;
                        cnt += 1;
                        if cnt == self.max_events_per_tick {
                            break;
                        }
                    }
                    if cnt > 0 {
                        tokio::task::yield_now().await;
                    }
                }

                tick = self.actor.tick() => {
                    self.handle_error(tick)?;
                    tokio::task::yield_now().await;
                }

            }
        }

        self.actor.on_shutdown().await
    }

    #[inline]
    fn handle_error(&self, result: Result<()>) -> Result<()> {
        if let Err(e) = result {
            self.actor.on_error(e)?;
        }
        Ok(())
    }
}
