use std::sync::Arc;

use tokio::sync::{Mutex, mpsc::Receiver};

use crate::{
    Event, Topic,
    testing::{EventEntry, TestEvent},
};

pub struct EventCollector<E: Event, T: Topic<E>> {
    events: Arc<Mutex<Vec<EventEntry<E, T>>>>,
    receiver: Receiver<TestEvent<E, T>>,
}

impl<E: Event, T: Topic<E>> EventCollector<E, T> {
    pub fn new(
        receiver: Receiver<TestEvent<E, T>>,
        events: Arc<Mutex<Vec<EventEntry<E, T>>>>,
    ) -> Self {
        Self { receiver, events }
    }

    pub async fn run(&mut self) -> crate::Result {
        let mut is_alive = true;
        let mut recording = false;
        while is_alive {
            if let Some(mut event) = self.receiver.recv().await {
                let records = &mut self.events.lock().await;
                let mut should_stop = false;
                while is_alive {
                    match event {
                        TestEvent::Event(entry) if recording => records.push(entry),
                        TestEvent::Flush(responder) => {
                            // All prior events have been processed (we're here),
                            // signal the harness to continue
                            let _ = responder.send(());
                        }
                        TestEvent::Exit => is_alive = false,
                        TestEvent::Reset => records.clear(),
                        TestEvent::StartRecording => recording = true,
                        TestEvent::StopRecording => should_stop = true,
                        _ => {}
                    }
                    if let Ok(next_event) = self.receiver.try_recv() {
                        event = next_event;
                    } else {
                        break;
                    }
                }
                if should_stop && recording {
                    tokio::task::yield_now().await;
                    recording = false;
                }
            }
        }
        Ok(())
    }
}
