use std::sync::Arc;

use tokio::{select, sync::mpsc::Receiver};
use tokio_util::sync::CancellationToken;

use crate::{Actor, Config, Context, Envelope, Error, Result, Runtime};

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
            let (watchdog_tx, mut watchdog_rx) =
                tokio::sync::mpsc::channel::<()>(self.config.watchdog_channel_size);

            let mut runtime = Runtime {
                ctx: &self.ctx,
                receiver: &mut self.receiver,
                config: self.config.clone(),
                watchdog_tx,
            };

            let actor_fut = self.actor.run(&mut runtime);
            tokio::pin!(actor_fut);

            while self.ctx.is_alive() {
                select! {
                    biased;

                    _ = self.cancel_token.cancelled() => {
                        self.ctx.stop();
                        break;
                    },

                    result = &mut actor_fut => {
                        result?;
                        break;
                    }

                    _ = tokio::time::sleep(self.config.watchdog_interval) => {
                        // Drain all heartbeats - we just need to know at least one arrived
                        let mut got_heartbeat = false;
                        while watchdog_rx.try_recv().is_ok() {
                            got_heartbeat = true;
                        }

                        if !got_heartbeat {
                            eprintln!("Watchdog timeout for actor: {}", self.ctx.name());
                            return Err(Error::WatchdogTimeout);
                        }
                    }
                }
            }
        }

        self.actor.on_shutdown().await
    }
}
