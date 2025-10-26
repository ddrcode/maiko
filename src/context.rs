use tokio::sync::mpsc::{Receiver, Sender};

use crate::{Envelope, Event};

pub struct Context<E: Event> {
    pub(crate) sender: Sender<Envelope<E>>,
    pub(crate) receiver: Receiver<Envelope<E>>,
}

impl<E: Event> Context<E> {}

impl<E: Event> Default for Context<E> {
    fn default() -> Self {
        let (tx, rx) = tokio::sync::mpsc::channel::<Envelope<E>>(1);
        Context {
            sender: tx,
            receiver: rx,
        }
    }
}
