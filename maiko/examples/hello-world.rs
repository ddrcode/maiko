use maiko::*;

// Define your events
#[derive(Event, Clone, Debug)]
enum MyEvent {
    Hello(String),
}

// Create an actor
struct Greeter;

impl Actor for Greeter {
    type Event = MyEvent;

    async fn handle(&mut self, event: &Self::Event, meta: &Meta) -> Result<()> {
        match event {
            MyEvent::Hello(name) => {
                println!("Hello, {}! (from {})", name, meta.actor_name());
            }
        }
        Ok(())
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let mut sup = Supervisor::<MyEvent>::default();

    // Add actor and subscribe it to all topics (Broadcast)
    sup.add_actor("greeter", |_ctx| Greeter, &[Broadcast])?;

    // Start the supervisor and send a message
    sup.start().await?;
    sup.send(MyEvent::Hello("World".into())).await?;

    // Graceful shutdown (it attempts to process all events already in the queue)
    sup.stop().await?;
    Ok(())
}
