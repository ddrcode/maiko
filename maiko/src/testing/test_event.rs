use tokio::sync::oneshot;

use crate::{Event, Topic, testing::EventEntry};

pub enum TestEvent<E: Event, T: Topic<E>> {
    /// An event delivery record from the broker.
    Event(EventEntry<E, T>),
    /// Request to flush - collector responds when all prior events are processed.
    Flush(oneshot::Sender<()>),
    /// Signal to exit the collector loop.
    Exit,
    /// Clear all recorded events.
    Reset,
    /// Start recording events.
    StartRecording,
    /// Stop recording events.
    StopRecording,
    /// Notify all event queues are empty (up to this point).
    Idle,
}

impl<E: Event, T: Topic<E>> std::fmt::Debug for TestEvent<E, T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TestEvent::Event(e) => write!(f, "Event({})", e.event.id()),
            TestEvent::Flush(_) => write!(f, "Flush"),
            TestEvent::Exit => write!(f, "Exit"),
            TestEvent::Reset => write!(f, "Reset"),
            TestEvent::StartRecording => write!(f, "StartRecording"),
            TestEvent::StopRecording => write!(f, "StopRecording"),
            TestEvent::Idle => write!(f, "Idle"),
        }
    }
}
