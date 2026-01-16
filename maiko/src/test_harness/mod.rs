mod event_collector;
mod event_entry;
mod test_event;
mod test_harness;

use std::sync::Arc;

pub(crate) use event_collector::EventCollector;
pub(crate) use event_entry::EventEntry;
pub(crate) use test_event::TestEvent;
pub use test_harness::TestHarness;
use tokio::sync::{Mutex, mpsc::Sender};

use crate::{Envelope, Event, Topic};

pub(crate) fn init_harness<E: Event, T: Topic<E>>(
    actor_sender: Sender<Arc<Envelope<E>>>,
) -> (TestHarness<E, T>, EventCollector<E, T>) {
    let (tx, rx) = tokio::sync::mpsc::channel(1024);
    let events = Arc::new(Mutex::new(Vec::with_capacity(1024)));
    let collector = EventCollector::new(rx, events.clone());
    (TestHarness::new(tx, actor_sender, events), collector)
}
