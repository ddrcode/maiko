use maiko::*;
use tokio;

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

    fn ctx(&self) -> &Context<Self::Event> {
        &self.ctx
    }

    fn ctx_mut(&mut self) -> &mut Context<Self::Event> {
        &mut self.ctx
    }

    fn set_ctx(&mut self, ctx: Context<Self::Event>) -> Result<()> {
        self.ctx = ctx;
        Ok(())
    }

    async fn handle(&mut self, event: Self::Event) -> Result<Option<Self::Event>> {
        match event {
            PingPongEvent::Ping => {
                println!("Ping");
                Ok(Some(PingPongEvent::Pong))
            }
            PingPongEvent::Pong => {
                println!("Pong");
                Ok(Some(PingPongEvent::Ping))
            }
        }
    }
}

#[tokio::main]
pub async fn main() -> Result<()> {
    let broker = Broker::<PingPongEvent>::new(128);
    Ok(())
}
