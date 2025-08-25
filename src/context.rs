use tokio::sync::mpsc::Sender;

use crate::{Envelope, Event};

pub struct Context<E: Event> {
    pub(super) sender: Sender<Envelope<E>>,
}

impl<E: Event> Context<E> {}
