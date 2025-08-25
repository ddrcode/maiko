use crate::{Envelope, Event, Topic};
use tokio::sync::mpsc::{Receiver, Sender, channel};

pub struct Broker<E: Event> {
    tx: Sender<Envelope<E>>,
    rx: Receiver<Envelope<E>>,
}

impl<E: Event> Broker<E> {
    pub fn new(size: usize) -> Broker<E> {
        let (tx, rx) = channel::<Envelope<E>>(size);
        Broker { tx, rx }
    }

    async fn send<T: Topic<E>>(&self, event: E) {
        let topic = T::from_event(&event);
        let event = Envelope::new(event);
    }
}
