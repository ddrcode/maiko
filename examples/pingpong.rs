use async_trait::async_trait;
use maiko::*;
use tokio;

#[derive(Clone)]
enum PingPongEvent {
    Ping,
    Pong,
}

impl Event for PingPongEvent {}

struct PingPong {
    name: String,
    ctx: Context<PingPongEvent>,
}

impl PingPong {
    fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            ctx: Context::default(),
        }
    }
}

#[async_trait]
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

    async fn start(&mut self) -> Result<()> {
        if self.name == "ping-side" {
            println!("Starting");
            self.send(PingPongEvent::Pong).await?;
        }
        Ok(())
    }

    async fn handle(&mut self, event: &Self::Event) -> Result<Option<Self::Event>> {
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

    fn name(&self) -> &str {
        self.name.as_str()
    }
}

#[tokio::main]
pub async fn main() -> Result<()> {
    let mut sup = Supervisor::<PingPongEvent, DefaultTopic>::new(128);
    sup.add_actor(PingPong::new("ping-side"), vec![DefaultTopic])?;
    sup.add_actor(PingPong::new("pong-side"), vec![DefaultTopic])?;
    sup.start().await?;
    Ok(())
}
