use std::sync::{Arc, atomic::AtomicBool};

use tokio::sync::oneshot;
use tokio_util::sync::CancellationToken;

use crate::{
    Config, Event, Topic,
    monitoring::{
        Monitor, MonitorCommand, MonitorDispatcher, MonitorHandle, MonitorId, MonitoringSink,
    },
};

pub struct MonitorRegistry<E: Event, T: Topic<E>> {
    cancel_token: Arc<CancellationToken>,
    dispatcher: Option<MonitorDispatcher<E, T>>,
    dispatcher_handle: Option<tokio::task::JoinHandle<()>>,
    pub(crate) sender: tokio::sync::mpsc::Sender<MonitorCommand<E, T>>,
    pub(crate) is_active: Arc<AtomicBool>,
}

impl<E: Event, T: Topic<E>> MonitorRegistry<E, T> {
    pub(crate) fn new(config: &Config) -> Self {
        let cancel_token = Arc::new(CancellationToken::new());
        let (tx, rx) = tokio::sync::mpsc::channel(config.monitoring_channel_size);
        let is_active = Arc::new(AtomicBool::new(false));
        let dispatcher = MonitorDispatcher::new(rx, cancel_token.clone(), is_active.clone());
        Self {
            cancel_token,
            sender: tx,
            dispatcher: Some(dispatcher),
            dispatcher_handle: None,
            is_active,
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
        self.cancel();
        if let Some(handle) = self.dispatcher_handle.take() {
            let _ = handle.await;
        }
    }

    pub(crate) fn cancel(&self) {
        self.cancel_token.cancel();
    }

    pub(crate) fn sink(&self) -> MonitoringSink<E, T> {
        MonitoringSink::new(self.sender.clone(), self.is_active.clone())
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
        let _ = self.sender.send(MonitorCommand::PauseAll).await;
    }

    pub async fn resume(&self) {
        let _ = self.sender.send(MonitorCommand::ResumeAll).await;
    }
}
