use std::{collections::HashMap, sync::Arc};

use tokio::{select, sync::mpsc::Receiver};
use tokio_util::sync::CancellationToken;

use crate::{
    Event, Topic,
    monitoring::{Monitor, MonitorCommand},
};

pub(crate) struct MonitorDispatcher<E: Event, T: Topic<E>> {
    receiver: Receiver<MonitorCommand<E, T>>,
    monitors: HashMap<u8, Box<dyn Monitor<E, T>>>,
    last_id: u8,
    paused: bool,
    cancel_token: Arc<CancellationToken>,
}

impl<E: Event, T: Topic<E>> MonitorDispatcher<E, T> {
    pub fn new(
        receiver: Receiver<MonitorCommand<E, T>>,
        cancel_token: Arc<CancellationToken>,
    ) -> Self {
        Self {
            receiver,
            cancel_token,
            monitors: HashMap::new(),
            last_id: 0,
            paused: false,
        }
    }

    pub async fn run(&mut self) {
        loop {
            select! {
                _ = self.cancel_token.cancelled() => {
                    break;
                }
                Some(cmd) = self.receiver.recv() => {
                    self.handle_command(cmd).await;
                }
            }
        }
    }

    async fn handle_command(&mut self, cmd: MonitorCommand<E, T>) {
        use MonitorCommand::*;
        match cmd {
            AddMonitor(monitor, resp) => {
                let id = self.last_id;
                self.monitors.insert(id, monitor);
                self.last_id += 1;
                let _ = resp.send(id);
            }
            RemoveMonitor(id) => {
                self.monitors.remove(&id);
            }
            Pause => {
                self.paused = true;
            }
            Resume => {
                self.paused = false;
            }
            PauseOne(_) => {
                todo!("PauseOne handle not implemented");
            }
            ResumeOne(_) => {
                todo!("ResumeOne handle not implemented");
            }
            EventDispatched(envelope, topic, actor_id) if !self.paused => {
                self.monitors
                    .values()
                    .for_each(|m| m.on_event_dispatched(&envelope, &topic, &actor_id));
            }

            _ => {}
        }
    }
}
