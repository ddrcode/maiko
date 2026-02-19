use std::collections::{HashMap, HashSet};
use std::sync::{Arc, Mutex};

use crate::{ActorId, DefaultTopic, Envelope, Event, OverflowPolicy, Topic, monitoring::Monitor};

/// Monitor that tracks actor lifecycle and overflow status.
pub struct ActorMonitor {
    inner: Arc<Mutex<ActorMonitorInner>>,
}

struct ActorMonitorInner {
    active: HashSet<ActorId>,
    stopped: HashSet<ActorId>,
    overflow_counts: HashMap<ActorId, usize>,
}

/// Status returned by `actor_status()`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ActorStatus {
    Alive,
    Stopped,
    Overflowing(usize),
}

impl ActorMonitor {
    /// Create a new `ActorMonitor`.
    pub fn new() -> Self {
        Self {
            inner: Arc::new(Mutex::new(ActorMonitorInner {
                active: HashSet::new(),
                stopped: HashSet::new(),
                overflow_counts: HashMap::new(),
            })),
        }
    }

    /// Returns a snapshot of currently registered (alive) actors.
    pub fn actors(&self) -> Vec<ActorId> {
        let lock = self.inner.lock().unwrap();
        lock.active.iter().cloned().collect()
    }

    /// Returns a snapshot of actors that have stopped.
    pub fn stopped_actors(&self) -> Vec<ActorId> {
        let lock = self.inner.lock().unwrap();
        lock.stopped.iter().cloned().collect()
    }

    /// Returns the status of a specific actor.
    pub fn actor_status(&self, actor: &ActorId) -> ActorStatus {
        let lock = self.inner.lock().unwrap();
        if let Some(&count) = lock.overflow_counts.get(actor) {
            return ActorStatus::Overflowing(count);
        }
        if lock.active.contains(actor) {
            ActorStatus::Alive
        } else {
            ActorStatus::Stopped
        }
    }
}

impl<E, T> Monitor<E, T> for ActorMonitor
where
    E: Event,
    T: Topic<E> + Send,
{
    fn on_actor_registered(&self, actor_id: &ActorId) {
        let mut lock = self.inner.lock().unwrap();
        lock.active.insert(actor_id.clone());
        // ensure stopped set doesn't keep a stale entry
        lock.stopped.remove(actor_id);
    }

    fn on_actor_stop(&self, actor_id: &ActorId) {
        let mut lock = self.inner.lock().unwrap();
        lock.active.remove(actor_id);
        lock.stopped.insert(actor_id.clone());
    }

    fn on_overflow(
        &self,
        _envelope: &Envelope<E>,
        _topic: &T,
        receiver: &ActorId,
        _policy: OverflowPolicy,
    ) {
        let mut lock = self.inner.lock().unwrap();
        *lock.overflow_counts.entry(receiver.clone()).or_insert(0) += 1;
    }
}

impl Default for ActorMonitor {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;

    #[derive(Clone, Debug)]
    struct TestEvent(i32);
    impl Event for TestEvent {}

    fn make_id(name: &str) -> ActorId {
        ActorId::new(Arc::from(name))
    }

    #[test]
    fn default_is_empty() {
        let m = ActorMonitor::default();
        assert!(m.actors().is_empty());
        assert!(m.stopped_actors().is_empty());
    }

    #[test]
    fn actor_registration_and_status() {
        let monitor = ActorMonitor::new();
        let a = make_id("actor-1");
        let m: &dyn Monitor<TestEvent, DefaultTopic> = &monitor;
        m.on_actor_registered(&a);
        assert!(monitor.actors().iter().any(|id| id == &a));
        assert_eq!(monitor.actor_status(&a), ActorStatus::Alive);
    }

    #[test]
    fn actor_stop_and_stopped_list() {
        let monitor = ActorMonitor::new();
        let a = make_id("actor-2");
        let m: &dyn Monitor<TestEvent, DefaultTopic> = &monitor;
        m.on_actor_registered(&a);
        m.on_actor_stop(&a);
        assert!(monitor.stopped_actors().iter().any(|id| id == &a));
        assert_eq!(monitor.actor_status(&a), ActorStatus::Stopped);
    }

    #[test]
    fn overflow_counts_and_status() {
        let monitor = ActorMonitor::new();
        let a = make_id("actor-3");
        let env = Envelope::new(TestEvent(1), a.clone());
        let topic = DefaultTopic;
        monitor.on_overflow(&env, &topic, &a, OverflowPolicy::Fail);
        assert_eq!(monitor.actor_status(&a), ActorStatus::Overflowing(1));
        monitor.on_overflow(&env, &topic, &a, OverflowPolicy::Fail);
        assert_eq!(monitor.actor_status(&a), ActorStatus::Overflowing(2));
    }

    #[test]
    fn overflow_precedes_alive_or_stopped() {
        let monitor = ActorMonitor::new();
        let a = make_id("actor-4");
        let env = Envelope::new(TestEvent(1), a.clone());
        let topic = DefaultTopic;

        let m: &dyn Monitor<TestEvent, DefaultTopic> = &monitor;
        m.on_actor_registered(&a);
        m.on_overflow(&env, &topic, &a, OverflowPolicy::Fail);
        // overflow takes precedence in `actor_status()`
        assert_eq!(monitor.actor_status(&a), ActorStatus::Overflowing(1));

        m.on_actor_stop(&a);
        // still overflowing even after stop (overflow_counts checked first)
        assert_eq!(monitor.actor_status(&a), ActorStatus::Overflowing(1));
    }
}
