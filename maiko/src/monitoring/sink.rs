use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};

use tokio::sync::mpsc::Sender;

use crate::{
    Event, Topic,
    monitoring::{MonitorCommand, MonitoringEvent},
};

pub(crate) struct MonitoringSink<E: Event, T: Topic<E>> {
    is_active: Arc<AtomicBool>,
    sender: Sender<MonitorCommand<E, T>>,
}

impl<E: Event, T: Topic<E>> MonitoringSink<E, T> {
    pub fn new(sender: Sender<MonitorCommand<E, T>>, is_active: Arc<AtomicBool>) -> Self {
        Self { sender, is_active }
    }

    #[inline]
    pub fn is_active(&self) -> bool {
        self.is_active.load(Ordering::Relaxed)
    }

    #[inline]
    pub fn send(&self, event: MonitoringEvent<E, T>) {
        let msg = MonitorCommand::DispatchEvent(event);
        let _ = self.sender.try_send(msg);
    }
}
