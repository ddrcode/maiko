use std::{sync::Arc, time::Duration};

use tokio::{
    sync::{Mutex, mpsc::Sender},
    time::sleep,
};

use crate::{
    ActorHandle, Envelope, Event, EventId, Topic,
    testing::{ActorSpy, EventEntry, EventQuery, EventRecords, EventSpy, TestEvent, TopicSpy},
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
    pub(crate) test_sender: Sender<TestEvent<E, T>>,
    actor_sender: Sender<Arc<Envelope<E>>>,
    entries: Arc<Mutex<Vec<EventEntry<E, T>>>>,
    snapshot: EventRecords<E, T>,
}

impl<E: Event, T: Topic<E>> Harness<E, T> {
    pub fn new(
        test_sender: Sender<TestEvent<E, T>>,
        actor_sender: Sender<Arc<Envelope<E>>>,
        entries: Arc<Mutex<Vec<EventEntry<E, T>>>>,
    ) -> Self {
        Self {
            test_sender,
            actor_sender,
            entries,
            snapshot: Arc::new(Vec::new()),
        }
    }

    // ==================== Recording Control ====================

    /// Start recording events. Call before sending test events.
    pub async fn start_recording(&mut self) {
        let _ = self.test_sender.send(TestEvent::StartRecording).await;
    }

    /// Stop recording and capture a snapshot for querying.
    ///
    /// After calling this, spy methods will query the captured snapshot.
    pub async fn stop_recording(&mut self) {
        self.settle().await;
        self.snapshot = self.take_snapshot().await;
        let _ = self.test_sender.send(TestEvent::StopRecording).await;
    }

    /// Clear all recorded events and reset the snapshot.
    pub async fn reset(&mut self) {
        let _ = self.test_sender.send(TestEvent::Reset).await;
        self.snapshot = Arc::new(Vec::new());
    }

    /// Signal the test harness to exit.
    pub async fn exit(&self) {
        let _ = self.test_sender.send(TestEvent::Exit).await;
    }

    /// Wait for events to propagate through the system.
    ///
    /// This is a naive implementation using a fixed delay. For more reliable
    /// testing, prefer explicit assertions on expected state.
    pub async fn settle(&self) {
        // FIXME: Replace with proper synchronization
        sleep(Duration::from_millis(20)).await
    }

    async fn take_snapshot(&self) -> EventRecords<E, T> {
        let entries = self.entries.lock().await;
        Arc::new(entries.clone())
    }

    // ==================== Event Injection ====================

    /// Send an event as if it came from the specified actor.
    ///
    /// Returns the event ID which can be used with [`event`](Self::event) to
    /// inspect delivery.
    pub async fn send_as(&self, actor: &ActorHandle, event: E) -> crate::Result<EventId> {
        let envelope = Envelope::new(event, actor.name());
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
    pub fn actor(&self, actor: &ActorHandle) -> ActorSpy<E, T> {
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
