use std::{
    collections::HashMap,
    panic::{AssertUnwindSafe, catch_unwind},
    sync::Arc,
};

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
    ids_to_remove: Vec<u8>,
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
            ids_to_remove: Vec::with_capacity(8),
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
            PauseOne(_id) => {
                todo!("PauseOne handle not implemented");
            }
            ResumeOne(_id) => {
                todo!("ResumeOne handle not implemented");
            }
            EventDispatched(envelope, topic, actor_id) if !self.paused => {
                self.notify(|m| m.on_event_dispatched(&envelope, &topic, &actor_id));
            }
            EventDelivered(envelope, actor_id) if !self.paused => {
                self.notify(|m| m.on_event_delivered(&envelope, &actor_id));
            }
            EventHandled(envelope, actor_id) if !self.paused => {
                self.notify(|m| m.on_event_handled(&envelope, &actor_id));
            }
            Error(error, actor_id) if !self.paused => {
                self.notify(|m| m.on_error(&error, &actor_id));
            }
            ActorStopped(actor_id) if !self.paused => {
                self.notify(|m| m.on_actor_stop(&actor_id));
            }

            _ => {}
        }
    }
}

impl<E: Event, T: Topic<E>> MonitorDispatcher<E, T> {
    fn notify(&mut self, f: impl Fn(&dyn Monitor<E, T>)) {
        for (id, monitor) in &self.monitors {
            let result = catch_unwind(AssertUnwindSafe(|| f(monitor.as_ref())));
            if result.is_err() {
                tracing::error!(monitor_id = %id, "Monitor panicked, removing");
                self.ids_to_remove.push(*id);
            }
        }

        self.ids_to_remove.drain(..).for_each(|id| {
            self.monitors.remove(&id);
        });
    }
}
