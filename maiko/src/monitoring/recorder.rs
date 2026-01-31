use std::fs::File;
use std::io::BufWriter;
use std::sync::{Arc, Mutex};
use std::path::Path;
use serde::Serialize;
use crate::{ActorId, Envelope, Event, Topic, monitoring::Monitor, DefaultTopic};

/// A monitor that records events to a file in JSON format.
///
/// It records each dispatched event as a JSON object on a new line.
pub struct Recorder {
    writer: Arc<Mutex<BufWriter<File>>>,
}

impl Recorder {
    /// Create a new recorder that writes to the specified path.
    pub fn new<P: AsRef<Path>>(path: P) -> std::io::Result<Self> {
        let file = File::create(path)?;
        Ok(Self {
            writer: Arc::new(Mutex::new(BufWriter::new(file))),
        })
    }
}

impl<E, T> Monitor<E, T> for Recorder
where
    E: Event + Serialize,
    T: Topic<E>,
{
    fn on_event_dispatched(&self, envelope: &Envelope<E>, _topic: &T, receiver: &ActorId) {
        // We define a helper struct to serialize the envelope + receiver info.
        // Flattening the envelope includes its meta and event fields directly.
        #[derive(Serialize)]
        struct RecordEntry<'a, E> {
            #[serde(flatten)]
            envelope: &'a Envelope<E>,
            receiver: &'a ActorId,
        }

        let entry = RecordEntry {
            envelope,
            receiver,
        };

        if let Ok(mut writer) = self.writer.lock() {
            if let Err(e) = serde_json::to_writer(&mut *writer, &entry) {
                eprintln!("Recorder failed to serialize event: {}", e);
            }
            // Add newline for easy reading/parsing (JSON Lines)
            // Add newline for easy reading/parsing (JSON Lines)
            let _ = std::io::Write::write_all(&mut *writer, b"\n");
            let _ = std::io::Write::flush(&mut *writer);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Read;
    use std::sync::Arc;
    use serde::Serialize;

    #[derive(Clone, Debug, Serialize)]
    struct TestEvent(String);
    impl Event for TestEvent {}

    #[test]
    fn test_recorder_writes_json() {
        let path = "test_log.jsonl";
        let recorder = Recorder::new(path).expect("Failed to create recorder");
        
        let event = TestEvent("hello".to_string());
        let sender_id = ActorId::new(Arc::from("sender"));
        let envelope = Envelope::new(event.clone(), sender_id);
        let receiver_id = ActorId::new(Arc::from("receiver"));

        recorder.on_event_dispatched(&envelope, &DefaultTopic, &receiver_id);

        // Verify content
        let mut file = File::open(path).expect("Failed to open log file");
        let mut content = String::new();
        file.read_to_string(&mut content).expect("Failed to read log file");

        // Simple string checks
        assert!(content.contains("hello"));
        assert!(content.contains("sender"));
        assert!(content.contains("receiver"));

        // Cleanup
        let _ = std::fs::remove_file(path);
    }
}
