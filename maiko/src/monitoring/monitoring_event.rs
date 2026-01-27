use std::sync::Arc;

use crate::{ActorId, Envelope, Error};

pub(crate) enum MonitoringEvent<E: crate::Event, T: crate::Topic<E>> {
    EventDispatched(Arc<Envelope<E>>, Arc<T>, ActorId),
    EventDelivered(Arc<Envelope<E>>, Arc<T>, ActorId),
    EventHandled(Arc<Envelope<E>>, Arc<T>, ActorId),
    ActorStopped(ActorId),
    Error(Error, ActorId),
}
