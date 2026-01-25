//! Test harness for observing and asserting on event flow.
//!
//! Enable with the `test-harness` feature:
//!
//! ```toml
//! [dev-dependencies]
//! maiko = { version = "0.2", features = ["test-harness"] }
//! ```
//!
//! # Example
//!
//! ```ignore
//! let mut test = supervisor.init_test_harness().await;
//! supervisor.start().await?;
//!
//! test.start_recording().await;
//! let id = test.send_as(&producer, MyEvent::Data(42)).await?;
//! test.stop_recording().await;
//!
//! // Query using spies
//! assert!(test.event(id).was_delivered_to(&consumer));
//! assert_eq!(1, test.actor(&consumer).inbound_count());
//!
//! // Or use EventQuery for complex queries
//! let orders = test.events()
//!     .sent_by(&trader)
//!     .matching_event(|e| matches!(e, MyEvent::Order(_)))
//!     .count();
//! ```

mod actor_spy;
mod event_collector;
mod event_entry;
mod event_query;
mod event_spy;
mod harness;
mod test_event;
mod topic_spy;

use std::sync::{Arc, atomic::AtomicBool};

pub use actor_spy::ActorSpy;
pub(crate) use event_collector::EventCollector;
pub use event_entry::EventEntry;
pub use event_query::EventQuery;
pub use event_spy::EventSpy;
pub use harness::Harness;
pub(crate) use test_event::TestEvent;
use tokio::sync::{Mutex, mpsc::Sender};
pub use topic_spy::TopicSpy;

use crate::{Envelope, Event, Topic};

pub(crate) type EventRecords<E, T> = Arc<Vec<EventEntry<E, T>>>;

/// Shared flag for broker to check if recording is active.
/// This avoids the overhead of sending events when not recording.
pub(crate) type RecordingFlag = Arc<AtomicBool>;

pub(crate) fn init_harness<E: Event, T: Topic<E>>(
    actor_sender: Sender<Arc<Envelope<E>>>,
) -> (Harness<E, T>, EventCollector<E, T>, RecordingFlag) {
    let (tx, rx) = tokio::sync::mpsc::channel(1024);
    let events = Arc::new(Mutex::new(Vec::with_capacity(1024)));
    let recording = Arc::new(AtomicBool::new(false));
    let collector = EventCollector::new(rx, events.clone());
    (
        Harness::new(tx, actor_sender, events, recording.clone()),
        collector,
        recording,
    )
}
