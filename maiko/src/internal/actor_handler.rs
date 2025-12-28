use std::sync::Arc;

use tokio::{select, sync::mpsc::Receiver};
use tokio_util::sync::CancellationToken;

use crate::{Actor, Config, Context, Envelope, Result, Runtime};

pub(crate) struct ActorHandler<A: Actor> {
    pub(crate) actor: A,
    pub(crate) receiver: Receiver<Arc<Envelope<A::Event>>>,
    pub(crate) ctx: Context<A::Event>,
    pub(crate) cancel_token: Arc<CancellationToken>,
    pub(crate) config: Arc<Config>,
}

impl<A: Actor> ActorHandler<A> {
    pub async fn run(&mut self) -> Result<()> {
        self.actor.on_start().await?;

        {
            let mut runtime = Runtime {
                ctx: &self.ctx,
                receiver: &mut self.receiver,
                cancel_token: self.cancel_token.clone(),
                config: self.config.clone(),
            };
            let actor_fut = self.actor.run(&mut runtime);
            tokio::pin!(actor_fut);

            while self.ctx.is_alive() {
                select! {
                    result = &mut actor_fut => {
                        result?;
                        break;
                    }

                    _ = tokio::time::sleep(self.config.watchdog_interval) => {
                        // Check if heartbeat received
                        // match self.watchdog_rx.try_recv() {
                        //     Ok(_) => continue,  // Heartbeat received, reset timer
                        //     Err(_) => {
                        //         // No heartbeat - actor might be stuck
                        //         tracing::error!(
                        //             "Actor {} watchdog timeout - no heartbeat for {:?}",
                        //             self.runtime.ctx.name(),
                        //             timeout
                        //         );
                        //         return Err(Error::WatchdogTimeout);
                        //     }
                        // }
                    }
                }
            }
        }

        self.actor.on_shutdown().await
    }
}
