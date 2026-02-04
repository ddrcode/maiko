use crate::{ActorId, Envelope, Event, Topic, monitoring::Monitor};
use serde::Serialize;
use std::cell::RefCell;
use std::fs::File;
use std::io::BufWriter;
use std::path::Path;

/// A monitor that records events to a file in JSON format.
///
/// It records each dispatched event as a JSON object on a new line.
pub struct Recorder {
    writer: RefCell<BufWriter<File>>,
}

// Monitor is Send, and we need to be Send to be passed to dispatcher.
// Since Recorder is only used in a single-threaded context within the dispatcher
// (as per reviewer's comment), we can implement Send.
// RefCell is Send if T is Send, and File/BufWriter are Send.
// So we don't need unsafe impl Send, struct is naturally Send.
// Wait, RefCell<T> is Send if T is Send. So it is fine.
// But RefCell is !Sync. Monitor requires Send, not Sync?
// Let's check Monitor trait definition in src/monitoring/monitor.rs
// pub trait Monitor<...>: Send { ... }
// It only requires Send. So RefCell is fine.

impl Recorder {
    /// Create a new recorder that writes to the specified path.
    pub fn new<P: AsRef<Path>>(path: P) -> std::io::Result<Self> {
        let file = File::create(path)?;
        Ok(Self {
            writer: RefCell::new(BufWriter::new(file)),
        })
    }
}

impl<E, T> Monitor<E, T> for Recorder
where
    E: Event + Serialize,
    T: Topic<E>,
{
    fn on_event_dispatched(&self, envelope: &Envelope<E>, _topic: &T, _receiver: &ActorId) {
        // Just serialize the envelope directly as requested.
        if let Ok(mut writer) = self.writer.try_borrow_mut() {
            if let Err(e) = serde_json::to_writer(&mut *writer, envelope) {
                eprintln!("Recorder failed to serialize event: {}", e);
            }
            // Add newline for easy reading/parsing (JSON Lines)
            let _ = std::io::Write::write_all(&mut *writer, b"\n");
            let _ = std::io::Write::flush(&mut *writer);
        } else {
            eprintln!("Recorder failed to borrow writer");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::DefaultTopic;
    use serde::Serialize;
    use std::io::Read;
    use std::sync::Arc;

    #[derive(Clone, Debug, Serialize)]
    struct TestEvent(String);
    impl Event for TestEvent {}

    #[test]
    fn test_recorder_writes_json() {
        let path = "test_log_refcell.jsonl";
        let recorder = Recorder::new(path).expect("Failed to create recorder");

        let event = TestEvent("hello".to_string());
        let sender_id = ActorId::new(Arc::from("sender"));
        let envelope = Envelope::new(event.clone(), sender_id);
        let receiver_id = ActorId::new(Arc::from("receiver"));

        recorder.on_event_dispatched(&envelope, &DefaultTopic, &receiver_id);

        // Verify content
        let mut file = File::open(path).expect("Failed to open log file");
        let mut content = String::new();
        file.read_to_string(&mut content)
            .expect("Failed to read log file");

        // Simple string checks
        assert!(content.contains("hello"));
        assert!(content.contains("sender"));
        // Receiver is NOT recorded anymore per requirements

        // Cleanup
        let _ = std::fs::remove_file(path);
    }
}
