use std::sync::Arc;

use tokio::sync::oneshot;

use crate::{
    ActorId, Envelope, Event, Topic,
    monitoring::{Monitor, MonitorId},
};

pub(crate) enum MonitorCommand<E: Event, T: Topic<E>> {
    AddMonitor(Box<dyn Monitor<E, T>>, oneshot::Sender<MonitorId>),
    RemoveMonitor(MonitorId),
    EventDispatched(Arc<Envelope<E>>, T, ActorId),
    EventDelivered(Arc<Envelope<E>>, ActorId),
    EventHandled(Arc<Envelope<E>>, ActorId),
    Pause,
    Resume,
    PauseOne(MonitorId),
    ResumeOne(MonitorId),
}
