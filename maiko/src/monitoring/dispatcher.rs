use std::{
    collections::HashMap,
    panic::{AssertUnwindSafe, catch_unwind},
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
};

use tokio::{select, sync::mpsc::Receiver};
use tokio_util::sync::CancellationToken;

use crate::{
    Event, Topic,
    monitoring::{Monitor, MonitorCommand, MonitoringEvent},
};

pub(crate) struct MonitorDispatcher<E: Event, T: Topic<E>> {
    receiver: Receiver<MonitorCommand<E, T>>,
    monitors: HashMap<u8, Box<dyn Monitor<E, T>>>,
    last_id: u8,
    paused: bool,
    cancel_token: Arc<CancellationToken>,
    ids_to_remove: Vec<u8>,
    is_active: Arc<AtomicBool>,
}

impl<E: Event, T: Topic<E>> MonitorDispatcher<E, T> {
    pub fn new(
        receiver: Receiver<MonitorCommand<E, T>>,
        cancel_token: Arc<CancellationToken>,
        is_active: Arc<AtomicBool>,
    ) -> Self {
        Self {
            receiver,
            cancel_token,
            monitors: HashMap::new(),
            last_id: 0,
            paused: false,
            ids_to_remove: Vec::with_capacity(8),
            is_active,
        }
    }

    fn update_is_active(&mut self) {
        let active = !self.monitors.is_empty() && !self.paused;
        self.is_active.store(active, Ordering::Relaxed);
    }

    pub async fn run(&mut self) {
        loop {
            select! {
                _ = self.cancel_token.cancelled() => {
                    break;
                }
                Some(cmd) = self.receiver.recv() => {
                    self.handle_command(cmd)
                }
            }
        }
    }

    fn handle_command(&mut self, cmd: MonitorCommand<E, T>) {
        use MonitorCommand::*;
        match cmd {
            AddMonitor(monitor, resp) => {
                let id = self.last_id;
                self.monitors.insert(id, monitor);
                self.last_id += 1;
                self.update_is_active();
                let _ = resp.send(id);
            }
            RemoveMonitor(id) => {
                self.monitors.remove(&id);
                self.update_is_active();
            }
            Pause => {
                self.paused = true;
                self.update_is_active();
            }
            Resume => {
                self.paused = false;
                self.update_is_active();
            }
            PauseOne(_id) => {
                todo!("PauseOne handle not implemented");
            }
            ResumeOne(_id) => {
                todo!("ResumeOne handle not implemented");
            }
            DispatchEvent(event) if self.is_active.load(Ordering::Relaxed) => {
                self.handle_event(event);
            }
            _ => {}
        }
    }

    fn handle_event(&mut self, event: MonitoringEvent<E, T>) {
        use MonitoringEvent::*;
        match event {
            EventDispatched(envelope, topic, actor_id) => {
                self.notify(|m| m.on_event_dispatched(&envelope, &topic, &actor_id));
            }
            EventDelivered(envelope, actor_id) => {
                self.notify(|m| m.on_event_delivered(&envelope, &actor_id));
            }
            EventHandled(envelope, actor_id) => {
                self.notify(|m| m.on_event_handled(&envelope, &actor_id));
            }
            Error(error, actor_id) => {
                self.notify(|m| m.on_error(&error, &actor_id));
            }
            ActorStopped(actor_id) => {
                self.notify(|m| m.on_actor_stop(&actor_id));
            }
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
