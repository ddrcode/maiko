use std::sync::Arc;

use tokio::sync::mpsc::{Receiver, Sender};
use tokio_util::sync::CancellationToken;

use crate::{Envelope, Event, Result};

pub struct Context<E: Event> {
    pub(crate) sender: Sender<Envelope<E>>,
    pub(crate) receiver: Option<Receiver<Envelope<E>>>,
    pub(crate) cancel_token: Arc<CancellationToken>,
}

impl<E: Event> Context<E> {
    pub(crate) async fn send(&self, event: E, name: &str) -> Result<()> {
        self.sender.send(Envelope::new(event, name)).await?;
        Ok(())
    }
}

impl<E: Event> Default for Context<E> {
    // FIXME: ugly default
    fn default() -> Self {
        let (tx, rx) = tokio::sync::mpsc::channel::<Envelope<E>>(1);
        Context {
            sender: tx,
            receiver: Some(rx),
            cancel_token: Arc::new(CancellationToken::new()),
        }
    }
}
