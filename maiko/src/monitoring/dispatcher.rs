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
    monitoring::{Monitor, MonitorCommand, MonitorId, MonitoringEvent},
};

struct MonitorEntry<E: Event, T: Topic<E>> {
    monitor: Box<dyn Monitor<E, T>>,
    paused: bool,
}

impl<E: Event, T: Topic<E>> MonitorEntry<E, T> {
    fn new(monitor: Box<dyn Monitor<E, T>>) -> Self {
        Self {
            monitor,
            paused: false,
        }
    }
}

pub(crate) struct MonitorDispatcher<E: Event, T: Topic<E>> {
    receiver: Receiver<MonitorCommand<E, T>>,
    monitors: HashMap<MonitorId, MonitorEntry<E, T>>,
    last_id: MonitorId,
    cancel_token: Arc<CancellationToken>,
    ids_to_remove: Vec<MonitorId>,
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
            ids_to_remove: Vec::with_capacity(8),
            is_active,
        }
    }

    fn update_is_active(&mut self) {
        let active = self.monitors.values().any(|m| !m.paused);
        self.is_active.store(active, Ordering::Relaxed);
    }

    fn remove_monitor(&mut self, id: MonitorId) {
        self.monitors.remove(&id);
        self.update_is_active();
    }

    fn set_monitor_paused(&mut self, id: MonitorId, paused: bool) {
        if let Some(entry) = self.monitors.get_mut(&id) {
            entry.paused = paused;
            self.update_is_active();
        }
    }

    fn set_monitors_paused_to_all(&mut self, paused: bool) {
        for entry in self.monitors.values_mut() {
            entry.paused = paused;
        }
        self.update_is_active();
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
                self.monitors.insert(id, MonitorEntry::new(monitor));
                self.last_id += 1;
                self.update_is_active();
                let _ = resp.send(id);
            }
            RemoveMonitor(id) => {
                self.remove_monitor(id);
            }
            PauseAll => {
                self.set_monitors_paused_to_all(true);
            }
            ResumeAll => {
                self.set_monitors_paused_to_all(false);
            }
            PauseOne(id) => {
                self.set_monitor_paused(id, true);
            }
            ResumeOne(id) => {
                self.set_monitor_paused(id, false);
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
        for (id, entry) in &self.monitors {
            if entry.paused {
                continue;
            }

            let result = catch_unwind(AssertUnwindSafe(|| f(entry.monitor.as_ref())));
            if result.is_err() {
                tracing::error!(monitor_id = %id, "Monitor panicked, removing");
                self.ids_to_remove.push(*id);
            }
        }

        while let Some(id) = self.ids_to_remove.pop() {
            self.remove_monitor(id);
        }
    }
}
