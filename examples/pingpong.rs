use std::pin::Pin;
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

impl Actor for PingPong {
    type Event = PingPongEvent;
    type HandleFuture<'a> = Pin<Box<dyn std::future::Future<Output = Result<()>> + Send + 'a>>;
    type TickFuture<'a> = Pin<Box<dyn std::future::Future<Output = Result<()>> + Send + 'a>>;
    type StartFuture<'a> = Pin<Box<dyn std::future::Future<Output = Result<()>> + Send + 'a>>;
    type ShutdownFuture<'a> = Pin<Box<dyn std::future::Future<Output = Result<()>> + Send + 'a>>;

    fn on_start<'a>(&'a mut self) -> Self::StartFuture<'a> {
        Box::pin(async move {
            if self.ctx.name() == "pong-side" {
                self.ctx.send(PingPongEvent::Ping).await?;
            }
            Ok(())
        })
    }

    fn handle<'a>(&'a mut self, event: &'a Self::Event, _meta: &'a Meta) -> Self::HandleFuture<'a> {
        Box::pin(async move {
            println!("Event: {event:?}");
            match event {
                PingPongEvent::Ping => self.ctx.send(PingPongEvent::Pong).await?,
                PingPongEvent::Pong => self.ctx.send(PingPongEvent::Ping).await?,
            }
            Ok(())
        })
    }

    fn tick<'a>(&'a mut self) -> Self::TickFuture<'a> {
        Box::pin(async { Ok(()) })
    }

    fn on_shutdown<'a>(&'a mut self) -> Self::ShutdownFuture<'a> {
        Box::pin(async { Ok(()) })
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
