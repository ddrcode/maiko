use std::{sync::Arc, time::Duration};

use tokio::sync::mpsc::{Sender, UnboundedReceiver, unbounded_channel};

use crate::{
    ActorId, Envelope, Event, EventId, Supervisor, Topic,
    monitoring::MonitorHandle,
    testing::{ActorSpy, EventCollector, EventEntry, EventQuery, EventRecords, EventSpy, TopicSpy},
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
/// let mut test = supervisor.init_test_harness().await;
/// supervisor.start().await?;
///
/// test.start_recording().await;
/// let id = test.send_as(&producer, MyEvent::Data(42)).await?;
/// test.stop_recording().await;
///
/// assert!(test.event(id).was_delivered_to(&consumer));
/// assert_eq!(1, test.actor(&consumer).inbound_count());
/// ```
pub struct Harness<E: Event, T: Topic<E>> {
    snapshot: EventRecords<E, T>,
    monitor_handle: MonitorHandle<E, T>,
    receiver: UnboundedReceiver<EventEntry<E, T>>,
    actor_sender: Sender<Arc<Envelope<E>>>,
}

impl<E: Event, T: Topic<E>> Harness<E, T> {
    pub async fn new(supervior: &mut Supervisor<E, T>) -> Self {
        let (tx, rx) = unbounded_channel();
        let monitor = EventCollector::new(tx);
        let monitor_handle = supervior.monitors().add(monitor).await;
        // monitor_handle.pause().await;
        Self {
            snapshot: Vec::new(),
            monitor_handle,
            receiver: rx,
            actor_sender: supervior.sender.clone(),
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
    }

    /// Clear all recorded events and reset the snapshot.
    pub fn reset(&mut self) {
        self.snapshot.clear();
        while let Ok(_entry) = self.receiver.try_recv() {}
    }

    /// Wait for events to propagate through the system.
    ///
    /// This sends a flush command through the collector's queue and waits for
    /// acknowledgment, ensuring all prior events have been processed by the
    /// collector. A small delay is added to allow actors to process events
    /// and emit responses, which then need to flow through the broker and
    /// collector.
    ///
    /// For chatty actors that continuously produce events, use
    /// [`settle_with_timeout`](Self::settle_with_timeout) instead.
    pub async fn settle(&mut self) {
        tokio::task::yield_now().await;
        for _ in 0..3 {
            // tokio::time::sleep(Duration::from_millis(1)).await;
            tokio::task::yield_now().await;
        }
        self.monitor_handle.flush().await;
        while let Ok(entry) = self.receiver.try_recv() {
            self.snapshot.push(entry);
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
        EventQuery::new(self.snapshot.clone())
    }

    /// Returns a spy for observing a specific event by ID.
    ///
    /// Use this to inspect delivery and child events.
    pub fn event(&self, id: EventId) -> EventSpy<E, T> {
        EventSpy::new(self.snapshot.clone(), id)
    }

    /// Returns a spy for observing events from a specific actor's perspective.
    ///
    /// Use this to inspect what an actor sent and received.
    pub fn actor(&self, actor: &ActorId) -> ActorSpy<E, T> {
        ActorSpy::new(self.snapshot.clone(), actor.clone())
    }

    /// Returns a spy for observing events on a specific topic.
    ///
    /// Use this to inspect event flow through a topic.
    pub fn topic(&self, topic: T) -> TopicSpy<E, T> {
        TopicSpy::new(self.snapshot.clone(), topic)
    }

    // ==================== Debugging ====================

    /// Print all recorded events to stdout for debugging.
    ///
    /// Shows sender, receiver, and event ID for each recorded delivery.
    pub fn dump(&self) {
        if self.snapshot.is_empty() {
            println!("(no events recorded)");
            return;
        }
        println!("Recorded events ({} deliveries):", self.snapshot.len());
        for (i, entry) in self.snapshot.iter().enumerate() {
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
        self.snapshot.len()
    }
}
