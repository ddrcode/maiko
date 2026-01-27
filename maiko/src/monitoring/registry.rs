use std::sync::Arc;

use tokio::sync::oneshot;
use tokio_util::sync::CancellationToken;

use crate::{
    Event, Topic,
    monitoring::{Monitor, MonitorCommand, MonitorDispatcher, MonitorHandle, MonitorId},
};

pub struct MonitorRegistry<E: Event, T: Topic<E>> {
    cancel_token: Arc<CancellationToken>,
    dispatcher: Option<MonitorDispatcher<E, T>>,
    dispatcher_handle: Option<tokio::task::JoinHandle<()>>,
    pub(crate) sender: tokio::sync::mpsc::Sender<MonitorCommand<E, T>>,
}

impl<E: Event, T: Topic<E>> MonitorRegistry<E, T> {
    pub(crate) fn new() -> Self {
        let cancel_token = Arc::new(CancellationToken::new());
        let (tx, rx) = tokio::sync::mpsc::channel(1024);
        let dispatcher = MonitorDispatcher::new(rx, cancel_token.clone());
        Self {
            cancel_token,
            sender: tx,
            dispatcher: Some(dispatcher),
            dispatcher_handle: None,
        }
    }

    pub(crate) fn start(&mut self) {
        let mut dispatcher = self
            .dispatcher
            .take()
            .expect("Dispatcher must exist on start");
        let handle = tokio::spawn(async move {
            dispatcher.run().await;
        });
        self.dispatcher_handle = Some(handle);
    }

    pub(crate) async fn stop(&mut self) {
        self.cancel_token.cancel();
        if let Some(handle) = self.dispatcher_handle.take() {
            let _ = handle.await;
        }
    }

    pub async fn add<M: Monitor<E, T> + 'static>(&self, monitor: M) -> MonitorHandle<E, T> {
        let (tx, rx) = oneshot::channel();
        let _ = self
            .sender
            .send(MonitorCommand::AddMonitor(Box::new(monitor), tx))
            .await;
        let id = rx
            .await
            .expect("Monitor Id should be retrieved successfully");
        MonitorHandle::new(id, self.sender.clone())
    }

    pub async fn remove(&self, id: MonitorId) {
        let _ = self.sender.send(MonitorCommand::RemoveMonitor(id)).await;
    }

    pub async fn pause(&self) {
        let _ = self.sender.send(MonitorCommand::Pause).await;
    }

    pub async fn resume(&self) {
        let _ = self.sender.send(MonitorCommand::Resume).await;
    }
}
