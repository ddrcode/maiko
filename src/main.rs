use maiko::prelude::*;

#[derive(Debug, Clone)]
enum Event {
    Ping,
    Pong,
}

struct PingPongActor {}

impl Actor<Event> for PingPongActor {
    async fn handle(&mut self, event: Event) -> Result<()> {
        println!("Got {event:?}");
        match event {
            Event::Ping => self.send(Event::Pong).await?,
            Event::Pong => self.send(Event::Ping).await?,
        }
        Ok(())
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let sup = Supervisor::new()
        .spawn("pp1", PingPongActor{})
        .spawn("pp2", PingPongActor{})
        .run()
        .await;

    sup.send_to("pp1", Event::Ping).await?;
    
    tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
    sup.stop().await?;
    
    Ok(())
}