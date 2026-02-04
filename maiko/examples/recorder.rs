use maiko::{monitors::Recorder, *};
use serde::{Deserialize, Serialize};

#[allow(unused)]
#[derive(Event, Clone, Debug, Serialize, Deserialize)]
enum MyEvent {
    Hello(String),
    Goodbye(u32),
}

struct Greeter;

impl Actor for Greeter {
    type Event = MyEvent;

    async fn handle_event(&mut self, envelope: &Envelope<Self::Event>) -> Result<()> {
        println!("Actor received: {:?}", envelope.event());
        Ok(())
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let mut sup = Supervisor::<MyEvent>::default();

    // Create a recorder that logs to "event_log.jsonl"
    let recorder = Recorder::new("event_log.jsonl")?;

    // Register the recorder monitor and get the handle
    let _recorder_handle = sup.monitors().add(recorder).await;

    // Add an actor
    sup.add_actor("greeter", |_ctx| Greeter, &[DefaultTopic])?;

    // Start the supervisor
    sup.start().await?;

    // Send some events
    sup.send(MyEvent::Hello("World".into())).await?;
    sup.send(MyEvent::Goodbye(42)).await?;

    // Wait a bit for processing
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

    // sup.stop() gracefully shuts down and waits for queue to drain.
    // It is sufficient for flushing in this simple case as consumers finish processing.
    sup.stop().await?;

    println!("Events recorded to event_log.jsonl");
    Ok(())
}
