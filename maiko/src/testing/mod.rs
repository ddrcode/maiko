mod actor_spy;
mod event_collector;
mod event_entry;
mod event_query;
mod event_spy;
mod harness;
pub(crate) mod spy_utils;
mod test_event;
mod topic_spy;

use std::sync::Arc;

pub use actor_spy::ActorSpy;
pub(crate) use event_collector::EventCollector;
pub(crate) use event_entry::EventEntry;
pub use event_query::EventQuery;
pub use event_spy::EventSpy;
pub use harness::Harness;
pub(crate) use test_event::TestEvent;
use tokio::sync::{Mutex, mpsc::Sender};
pub use topic_spy::TopicSpy;

use crate::{Envelope, Event, Topic};

type EventRecords<E, T> = Arc<Vec<EventEntry<E, T>>>;

pub(crate) fn init_harness<E: Event, T: Topic<E>>(
    actor_sender: Sender<Arc<Envelope<E>>>,
) -> (Harness<E, T>, EventCollector<E, T>) {
    let (tx, rx) = tokio::sync::mpsc::channel(1024);
    let events = Arc::new(Mutex::new(Vec::with_capacity(1024)));
    let collector = EventCollector::new(rx, events.clone());
    (Harness::new(tx, actor_sender, events), collector)
}
