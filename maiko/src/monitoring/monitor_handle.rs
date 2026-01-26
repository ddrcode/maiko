use tokio::sync::mpsc::Sender;

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
}
