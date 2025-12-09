use async_trait::async_trait;
use maiko::*;
use tokio::{self, select};

#[derive(Clone, Debug)]
enum PingPongEvent {
    Ping,
    Pong,
}
impl Event for PingPongEvent {}

struct PingPong;

#[async_trait]
impl Actor for PingPong {
    type Event = PingPongEvent;

    async fn on_start(&mut self, ctx: &Context<Self::Event>) -> Result<()> {
        if ctx.name() == "ping-side" {
            ctx.send(PingPongEvent::Pong).await?;
        }
        Ok(())
    }

    async fn handle(&mut self, event: &Self::Event, _meta: &Meta) -> Result<Option<Self::Event>> {
        println!("Event: {event:?}");
        match event {
            PingPongEvent::Ping => Ok(Some(PingPongEvent::Pong)),
            PingPongEvent::Pong => Ok(Some(PingPongEvent::Ping)),
        }
    }
}

#[tokio::main]
pub async fn main() -> Result<()> {
    let mut sup = Supervisor::<PingPongEvent, DefaultTopic>::default();
    sup.add_actor("ping-side", PingPong, vec![DefaultTopic])?;
    sup.add_actor("pong-side", PingPong, vec![DefaultTopic])?;

    let timeout = tokio::time::sleep(tokio::time::Duration::from_millis(1));
    select! {
        _ = sup.start() => {},
        _ = timeout => {
            sup.stop().await?;
        }
    }

    Ok(())
}
