use std::sync::Arc;

use tokio::{
    select,
    sync::mpsc::{Receiver, error::TryRecvError},
};
use tokio_util::sync::CancellationToken;

use crate::{Actor, Config, Context, Envelope, Event, Result};

pub struct Runtime<'a, E: Event> {
    pub ctx: &'a Context<E>,

    pub(crate) receiver: &'a mut Receiver<Arc<Envelope<E>>>,

    pub cancel_token: Arc<CancellationToken>,

    pub config: Arc<Config>,

    pub(crate) watchdog_tx: tokio::sync::mpsc::Sender<()>,
}

impl<'a, E: Event> Runtime<'a, E> {
    pub async fn recv(&mut self) -> Option<Arc<Envelope<E>>> {
        self.receiver.recv().await
    }

    pub async fn try_recv(&mut self) -> std::result::Result<Arc<Envelope<E>>, TryRecvError> {
        self.receiver.try_recv()
    }

    pub async fn default_run<'b, A: Actor<Event = E> + ?Sized>(
        &'b mut self,
        actor: &'b mut A,
    ) -> Result<()> {
        while self.ctx.is_alive() {
            self.heartbeat();

            select! {
                biased;

                Some(ref envelope) = self.receiver.recv() => {
                    self.default_event_handler(actor, envelope).await?;
                }

                tick = actor.tick() => {
                    Self::handle_error(actor, tick)?;
                    tokio::task::yield_now().await; // FIXME
                }

            }
        }
        Ok(())
    }

    pub async fn default_tick<'b, A: Actor<Event = E> + ?Sized>(
        &'b mut self,
        actor: &'b mut A,
    ) -> Result<()> {
        select! {
            biased;

            Some(ref envelope) = self.receiver.recv() => {
                self.default_event_handler(actor, envelope).await?;
            }
        }
        Ok(())
    }

    pub async fn default_event_handler<'b, A: Actor<Event = E> + ?Sized>(
        &'b mut self,
        actor: &'b mut A,
        envelope: &Arc<Envelope<E>>,
    ) -> Result<()> {
        let res = actor.handle_envelope(envelope).await;
        Self::handle_error(actor, res)?;

        let mut cnt = 1;
        while let Ok(envelope) = self.receiver.try_recv() {
            let res = actor.handle_envelope(&envelope).await;
            Self::handle_error(actor, res)?;
            cnt += 1;
            if cnt == self.config.max_events_per_tick {
                break;
            }
        }
        if cnt > 0 {
            tokio::task::yield_now().await;
        }
        Ok(())
    }

    #[inline]
    pub fn handle_error<A: Actor<Event = E> + ?Sized>(actor: &A, result: Result<()>) -> Result<()> {
        if let Err(e) = result {
            actor.on_error(e)?;
        }
        Ok(())
    }

    #[inline]
    pub fn heartbeat(&self) {
        let _ = self.watchdog_tx.try_send(());
    }
}
