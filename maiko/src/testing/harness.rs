use std::{
    sync::Arc,
    time::{Duration, Instant},
};

use tokio::sync::mpsc::{Sender, UnboundedReceiver, unbounded_channel};

use crate::{
    ActorId, Envelope, Event, EventId, Supervisor, Topic,
    monitoring::MonitorHandle,
    testing::{
        ActorSpy, EventChain, EventCollector, EventEntry, EventQuery, EventRecords, EventSpy,
        TopicSpy,
    },
};

/// Test harness for observing and asserting on event flow in a Maiko system.
///
/// The harness provides:
/// - Event injection via [`send_as`](Self::send_as)
/// - Recording control via [`start_recording`](Self::start_recording) / [`stop_recording`](Self::stop_recording)
/// - Query access via [`events`](Self::events), [`event`](Self::event), [`actor`](Self::actor), [`topic`](Self::topic)
///
/// # Example
///
/// ```ignore
/// // Create harness BEFORE starting supervisor
/// let mut test = Harness::new(&mut supervisor).await;
/// supervisor.start().await?;
///
/// test.start_recording().await;
/// let id = test.send_as(&producer, MyEvent::Data(42)).await?;
/// test.stop_recording().await;
///
/// assert!(test.event(id).was_delivered_to(&consumer));
/// assert_eq!(1, test.actor(&consumer).events_received());
/// ```
///
/// # Warning
///
/// **Do not use in production.** The test harness uses an unbounded channel
/// for event collection, which can lead to memory exhaustion under high load.
/// For production monitoring, use the [`monitoring`](crate::monitoring) API directly.
pub struct Harness<E: Event, T: Topic<E>> {
    snapshot: Vec<EventEntry<E, T>>,
    records: EventRecords<E, T>,
    monitor_handle: MonitorHandle<E, T>,
    receiver: UnboundedReceiver<EventEntry<E, T>>,
    actor_sender: Sender<Arc<Envelope<E>>>,
}

impl<E: Event, T: Topic<E>> Harness<E, T> {
    pub async fn new(supervisor: &mut Supervisor<E, T>) -> Self {
        let (tx, rx) = unbounded_channel();
        let monitor = EventCollector::new(tx);
        let monitor_handle = supervisor.monitors().add(monitor).await;
        // monitor_handle.pause().await;
        Self {
            snapshot: Vec::new(),
            records: Arc::new(Vec::new()),
            monitor_handle,
            receiver: rx,
            actor_sender: supervisor.sender.clone(),
        }
    }

    // ==================== Recording Control ====================

    /// Start recording events. Call before sending test events.
    pub async fn start_recording(&self) {
        self.monitor_handle.resume().await;
    }

    /// Stop recording and capture a snapshot for querying.
    ///
    /// After calling this, spy methods will query the captured snapshot.
    pub async fn stop_recording(&mut self) {
        self.settle().await;
        self.monitor_handle.pause().await;
        self.records = Arc::new(std::mem::take(&mut self.snapshot));
    }

    /// Clears all recorded events, resetting the harness for the next test phase.
    pub fn reset(&mut self) {
        self.snapshot.clear();
        self.records = Arc::new(Vec::new());
        while let Ok(_entry) = self.receiver.try_recv() {}
    }

    /// Default settle window: wait 1ms for quiet before considering settled.
    pub const DEFAULT_SETTLE_WINDOW: Duration = Duration::from_millis(1);

    /// Default max settle time: give up waiting after 10ms.
    pub const DEFAULT_MAX_SETTLE: Duration = Duration::from_millis(10);

    /// Wait for events to propagate through the system.
    ///
    /// Collects events until no new events arrive for 1ms (settle window),
    /// or until 10ms total have elapsed (max settle time).
    ///
    /// For chatty actors that continuously produce events, the max settle
    /// time prevents infinite waiting. Use [`settle_with_timeout`](Self::settle_with_timeout)
    /// for custom timing.
    pub async fn settle(&mut self) {
        self.settle_with_timeout(Self::DEFAULT_SETTLE_WINDOW, Self::DEFAULT_MAX_SETTLE)
            .await;
    }

    /// Wait for events with custom timing parameters.
    ///
    /// # Arguments
    ///
    /// * `settle_window` - Return early if no events arrive for this duration
    /// * `max_settle` - Maximum total time to wait, regardless of activity
    ///
    /// Useful for chatty actors where you want a shorter max settle time,
    /// or for slow systems where you need a longer settle window.
    pub async fn settle_with_timeout(&mut self, settle_window: Duration, max_settle: Duration) {
        let deadline = Instant::now() + max_settle;

        loop {
            let remaining = deadline.saturating_duration_since(Instant::now());
            if remaining.is_zero() {
                break;
            }

            let timeout = settle_window.min(remaining);
            match tokio::time::timeout(timeout, self.receiver.recv()).await {
                Ok(Some(entry)) => self.snapshot.push(entry),
                Ok(None) => break, // Channel closed
                Err(_) => break,   // Quiet for settle_window - system settled
            }
        }
    }

    // ==================== Event Injection ====================

    /// Send an event as if it came from the specified actor.
    ///
    /// Returns the event ID which can be used with [`event`](Self::event) to
    /// inspect delivery.
    pub async fn send_as(&self, actor: &ActorId, event: E) -> crate::Result<EventId> {
        let envelope = Envelope::new(event, actor.clone());
        let id = envelope.id();
        self.actor_sender.send(Arc::new(envelope)).await?;
        Ok(id)
    }

    // ==================== Query Access ====================

    /// Returns a query over all recorded events.
    ///
    /// This is the most flexible way to query events, allowing arbitrary
    /// filtering and inspection.
    ///
    /// # Example
    ///
    /// ```ignore
    /// let orders = test.events()
    ///     .sent_by(&trader)
    ///     .matching_event(|e| matches!(e, MarketEvent::Order(_)))
    ///     .count();
    /// ```
    pub fn events(&self) -> EventQuery<E, T> {
        EventQuery::new(self.records.clone())
    }

    /// Returns a spy for observing a specific event by ID.
    ///
    /// Use this to inspect delivery and child events.
    pub fn event(&self, id: EventId) -> EventSpy<E, T> {
        EventSpy::new(self.records.clone(), id)
    }

    /// Returns a spy for observing events from a specific actor's perspective.
    ///
    /// Use this to inspect what an actor sent and received.
    pub fn actor(&self, actor: &ActorId) -> ActorSpy<E, T> {
        ActorSpy::new(self.records.clone(), actor.clone())
    }

    /// Returns a spy for observing events on a specific topic.
    ///
    /// Use this to inspect event flow through a topic.
    pub fn topic(&self, topic: T) -> TopicSpy<E, T> {
        TopicSpy::new(self.records.clone(), topic)
    }

    /// Returns an event chain for tracing event propagation from a root event.
    ///
    /// The chain captures all events correlated to the root (children, grandchildren, etc.)
    /// and provides methods to verify actor flow and event sequences.
    ///
    /// # Example
    ///
    /// ```ignore
    /// let chain = test.chain(root_event_id);
    ///
    /// // Verify exact path from root to leaf
    /// assert!(chain.actors().exact(&[&scanner, &pipeline, &writer, &telemetry]));
    ///
    /// // Verify contiguous sub-path
    /// assert!(chain.actors().segment(&[&pipeline, &writer]));
    ///
    /// // Verify reachability (gaps allowed)
    /// assert!(chain.actors().passes_through(&[&scanner, &telemetry]));
    ///
    /// // Verify event sequence
    /// assert!(chain.events().segment(&["KeyPress", "HidReport"]));
    /// ```
    pub fn chain(&self, id: EventId) -> EventChain<E, T> {
        EventChain::new(self.records.clone(), id)
    }

    // ==================== Debugging ====================

    /// Print all recorded events to stdout for debugging.
    ///
    /// Shows sender, receiver, and event ID for each recorded delivery.
    pub fn dump(&self) {
        if self.records.is_empty() {
            println!("(no events recorded)");
            return;
        }
        println!("Recorded events ({} deliveries):", self.records.len());
        for (i, entry) in self.records.iter().enumerate() {
            println!(
                "  {}: [{}] --> [{}]  (id: {})",
                i,
                entry.sender(),
                entry.receiver(),
                entry.id(),
            );
        }
    }

    /// Returns the number of recorded events.
    pub fn event_count(&self) -> usize {
        self.records.len()
    }
}
