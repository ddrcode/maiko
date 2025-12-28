use std::sync::Arc;

use tokio::{select, sync::mpsc::Receiver};
use tokio_util::sync::CancellationToken;

use crate::{Actor, Config, Context, Envelope, Event, Result};

pub struct Runtime<'a, E: Event> {
    // Embed context (by reference)
    pub ctx: &'a Context<E>,

    // Plus receiver (exclusive to run())
    pub(crate) receiver: &'a mut Receiver<Arc<Envelope<E>>>,

    pub cancel_token: Arc<CancellationToken>,

    pub config: Arc<Config>,

    pub(crate) watchdog_tx: tokio::sync::mpsc::Sender<()>,
}

impl<'a, E: Event> Runtime<'a, E> {
    pub fn default_run<'b, A: Actor<Event = E> + ?Sized>(
        &'b mut self,
        actor: &'b mut A,
    ) -> impl Future<Output = Result<()>> + Send + 'b {
        async move {
            while self.ctx.is_alive() {
                self.heartbeat();

                select! {
                    biased;

                    _ = self.cancel_token.cancelled() => {
                        self.ctx.stop();
                        break;
                    },

                    Some(event) = self.receiver.recv() => {
                        let res = actor.handle(&event.event, &event.meta).await;
                        // Self::handle_error(actor, res)?;

                        let mut cnt = 1;
                        while let Ok(event) = self.receiver.try_recv() {
                            let res = actor.handle(&event.event, &event.meta).await;
                            // Self::handle_error(actor, res)?;
                            cnt += 1;
                            if cnt == self.config.max_events_per_tick {
                                break;
                            }
                        }
                        if cnt > 0 {
                            tokio::task::yield_now().await;
                        }
                    }

                    tick = actor.tick() => {
                        // Self::handle_error(actor, tick)?;
                        tick?;
                        tokio::task::yield_now().await;
                    }

                }
            }
            Ok(())
        }
    }

    #[inline]
    fn handle_error<A: Actor<Event = E>>(actor: &A, result: Result<()>) -> Result<()> {
        if let Err(e) = result {
            actor.on_error(e)?;
        }
        Ok(())
    }

    #[inline]
    pub fn heartbeat(&self) {
        // Non-blocking send - if buffer full, supervisor is alive
        let _ = self.watchdog_tx.try_send(());
    }
}
