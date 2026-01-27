use std::time::Duration;

use tokio::sync::{mpsc::Sender, oneshot};

use crate::{
    Event, Topic,
    monitoring::{MonitorCommand, MonitorId},
};

pub struct MonitorHandle<E: Event, T: Topic<E>> {
    id: MonitorId,
    sender: Sender<MonitorCommand<E, T>>,
}

impl<E: Event, T: Topic<E>> MonitorHandle<E, T> {
    pub(crate) fn new(id: MonitorId, sender: Sender<MonitorCommand<E, T>>) -> Self {
        Self { id, sender }
    }

    pub fn id(&self) -> MonitorId {
        self.id
    }

    pub async fn remove(self) {
        let _ = self
            .sender
            .send(MonitorCommand::RemoveMonitor(self.id))
            .await;
    }

    pub async fn pause(&self) {
        let _ = self.sender.send(MonitorCommand::PauseOne(self.id)).await;
    }

    pub async fn resume(&self) {
        let _ = self.sender.send(MonitorCommand::ResumeOne(self.id)).await;
    }

    /// Flush waits for the monitoring dispatcher queue to be empty and stay
    /// empty for the specified settle window before returning.
    ///
    /// This ensures all events queued before the flush call have been processed.
    pub async fn flush(&self, settle_window: Duration) {
        let (tx, rx) = oneshot::channel();
        let _ = self
            .sender
            .send(MonitorCommand::Flush {
                response: tx,
                settle_window,
            })
            .await;
        let _ = rx.await;
    }
}
