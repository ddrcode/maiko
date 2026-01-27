use tokio::sync::oneshot;

use crate::{
    Event, Topic,
    monitoring::{Monitor, MonitorId, MonitoringEvent},
};

pub(crate) enum MonitorCommand<E: Event, T: Topic<E>> {
    AddMonitor(Box<dyn Monitor<E, T>>, oneshot::Sender<MonitorId>),
    RemoveMonitor(MonitorId),
    Pause,
    Resume,
    PauseOne(MonitorId),
    ResumeOne(MonitorId),
    DispatchEvent(MonitoringEvent<E, T>),
}
