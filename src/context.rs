use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};

use tokio::sync::mpsc::Sender;

use crate::{Envelope, Event, Result};

#[derive(Clone)]
pub struct Context<E: Event> {
    pub(crate) name: Arc<str>,
    pub(crate) sender: Sender<Envelope<E>>,
    pub(crate) alive: Arc<AtomicBool>,
}

impl<E: Event> Context<E> {
    pub async fn send(&self, event: E) -> Result<()> {
        self.sender
            .send(Envelope::new(event, self.name.as_ref()))
            .await?;
        Ok(())
    }

    pub fn stop(&self) {
        self.alive.store(false, Ordering::Relaxed);
    }

    pub fn name(&self) -> &str {
        &self.name
    }
}
