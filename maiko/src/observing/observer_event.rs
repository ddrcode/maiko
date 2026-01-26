use std::sync::Arc;

use tokio::sync::oneshot;

use crate::{ActorId, Envelope, Event, Topic, observing::Observer};

pub(crate) enum ObserverEvent<E: Event, T: Topic<E>> {
    AddObserver(Box<dyn Observer<E, T>>, oneshot::Sender<u8>),
    RemoveObserver(u8),
    EventSent(Arc<Envelope<E>>, T, ActorId),
    EventReceived(Arc<Envelope<E>>, ActorId),
    EventProcessed(Arc<Envelope<E>>, ActorId),
    Pause,
    Resume,
}
