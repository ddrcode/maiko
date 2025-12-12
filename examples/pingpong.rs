use async_trait::async_trait;
use maiko::{Actor, Context, DefaultTopic, Event, Meta, Result, Supervisor};

#[derive(Clone, Debug)]
enum PingPongEvent {
    Ping,
    Pong,
}
impl Event for PingPongEvent {}

struct PingPong {
    ctx: Context<PingPongEvent>,
}

#[async_trait]
impl Actor for PingPong {
    type Event = PingPongEvent;

    async fn on_start(&mut self) -> Result<()> {
        if self.ctx.name() == "pong-side" {
            self.ctx.send(PingPongEvent::Ping).await?;
        }
        Ok(())
    }

    async fn handle(&mut self, event: &Self::Event, _meta: &Meta) -> Result<()> {
        println!("Event: {event:?}");
        match event {
            PingPongEvent::Ping => self.ctx.send(PingPongEvent::Pong).await?,
            PingPongEvent::Pong => self.ctx.send(PingPongEvent::Ping).await?,
        }
        Ok(())
    }
}

#[tokio::main]
pub async fn main() -> Result<()> {
    let mut sup = Supervisor::<PingPongEvent>::default();
    sup.add_actor("ping-side", |ctx| PingPong { ctx }, &[DefaultTopic])?;
    sup.add_actor("pong-side", |ctx| PingPong { ctx }, &[DefaultTopic])?;

    sup.start().await?;
    tokio::time::sleep(tokio::time::Duration::from_millis(1)).await;
    sup.stop().await?;
    println!("Done");
    Ok(())
}
