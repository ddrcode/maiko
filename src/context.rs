use std::sync::Arc;

use tokio::sync::mpsc::{Receiver, Sender};
use tokio_util::sync::CancellationToken;

use crate::{Envelope, Event};

pub struct Context<E: Event> {
    pub(crate) sender: Sender<Envelope<E>>,
    pub(crate) receiver: Receiver<Envelope<E>>,
    pub(crate) cancel_token: Arc<CancellationToken>,
}

impl<E: Event> Context<E> {}

impl<E: Event> Default for Context<E> {
    // FIXME: ugly default
    fn default() -> Self {
        let (tx, rx) = tokio::sync::mpsc::channel::<Envelope<E>>(1);
        Context {
            sender: tx,
            receiver: rx,
            cancel_token: Arc::new(CancellationToken::new()),
        }
    }
}
