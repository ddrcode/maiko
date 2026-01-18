mod actor_spy;
mod event_collector;
mod event_entry;
mod event_spy;
mod test_event;
mod test_harness;
mod topic_spy;

use std::sync::Arc;

pub use actor_spy::ActorSpy;
pub(crate) use event_collector::EventCollector;
pub(crate) use event_entry::EventEntry;
pub use event_spy::EventSpy;
pub(crate) use test_event::TestEvent;
pub use test_harness::TestHarness;
use tokio::sync::{Mutex, mpsc::Sender};
pub use topic_spy::TopicSpy;

use crate::{Envelope, Event, Topic};

pub(crate) fn init_harness<E: Event, T: Topic<E>>(
    actor_sender: Sender<Arc<Envelope<E>>>,
) -> (TestHarness<E, T>, EventCollector<E, T>) {
    let (tx, rx) = tokio::sync::mpsc::channel(1024);
    let events = Arc::new(Mutex::new(Vec::with_capacity(1024)));
    let collector = EventCollector::new(rx, events.clone());
    (TestHarness::new(tx, actor_sender, events), collector)
}
