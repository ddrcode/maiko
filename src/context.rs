use tokio::sync::mpsc::Sender;

use crate::Event;

pub struct Context<E: Event> {
    pub(super) sender: Sender<E>,
}
