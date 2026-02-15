use std::sync::Arc;

use crate::{ActorId, Envelope, OverflowPolicy};

pub(crate) enum MonitoringEvent<E: crate::Event, T: crate::Topic<E>> {
    EventDispatched(Arc<Envelope<E>>, Arc<T>, ActorId),
    EventDelivered(Arc<Envelope<E>>, Arc<T>, ActorId),
    EventHandled(Arc<Envelope<E>>, Arc<T>, ActorId),
    Overflow(Arc<Envelope<E>>, Arc<T>, ActorId, OverflowPolicy),
    ActorStopped(ActorId),
    Error(Arc<str>, ActorId),
}
