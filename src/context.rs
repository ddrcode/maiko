use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};

use tokio::sync::mpsc::Sender;
use tokio_util::sync::CancellationToken;

use crate::{Envelope, Event, Result};

#[derive(Clone)]
pub struct Context<E: Event> {
    pub(crate) name: Arc<str>,
    pub(crate) sender: Sender<Envelope<E>>,
    pub(crate) alive: Arc<AtomicBool>,
    pub(crate) cancel_token: Arc<CancellationToken>,
}

impl<E: Event> Context<E> {
    pub async fn send(&self, event: E) -> Result<()> {
        self.sender
            .send(Envelope::new(event, self.name.as_ref()))
            .await?;
        // .try_send(Envelope::new(event, self.name.as_ref()))?;
        Ok(())
    }

    pub fn stop(&self) {
        self.alive.store(false, Ordering::Release);
        self.cancel_token.cancel();
    }

    #[inline]
    pub fn name(&self) -> &str {
        &self.name
    }

    #[inline]
    pub fn is_alive(&self) -> bool {
        self.alive.load(Ordering::Relaxed)
    }
}
