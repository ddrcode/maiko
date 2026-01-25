use crate::{Event, Topic, testing::EventEntry};

pub enum TestEvent<E: Event, T: Topic<E>> {
    Event(EventEntry<E, T>),
    Exit,
    Reset,
    StartRecording,
    StopRecording,
}

impl<E: Event, T: Topic<E>> std::fmt::Debug for TestEvent<E, T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TestEvent::Event(e) => write!(f, "Event({})", e.event.id()),
            TestEvent::Exit => write!(f, "Exit"),
            TestEvent::Reset => write!(f, "Reset"),
            TestEvent::StartRecording => write!(f, "StartRecording"),
            TestEvent::StopRecording => write!(f, "StopRecording"),
        }
    }
}
