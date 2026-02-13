use std::time::Duration;

use maiko::{Actor, Context, StepAction, Supervisor, monitors::Tracer};

#[derive(maiko::Event, maiko::Label, Debug, Clone)]
enum Event {
    Start(usize),
    Data(Box<[u8; 1024]>),
    Done,
}

#[derive(Clone, PartialEq, Eq, Hash, Debug, maiko::Label)]
enum Topic {
    Data,
    Command,
}

impl maiko::Topic<Event> for Topic {
    fn from_event(event: &Event) -> Self
    where
        Self: Sized,
    {
        match event {
            Event::Start(_) => Topic::Command,
            Event::Data(_) => Topic::Data,
            Event::Done => Topic::Command,
        }
    }
}

struct Producer {
    ctx: Context<Event>,
    cnt: usize,
    checksum: u64,
}

impl Actor for Producer {
    type Event = Event;

    async fn handle_event(&mut self, envelope: &maiko::Envelope<Self::Event>) -> maiko::Result<()> {
        if let Event::Start(cnt) = envelope.event() {
            self.cnt = *cnt;
            self.checksum = 0;
        }
        Ok(())
    }

    async fn step(&mut self) -> maiko::Result<StepAction> {
        if self.cnt == 0 {
            return Ok(StepAction::AwaitEvent);
        }
        let mut buf: [u8; 1024] = [0; 1024];
        getrandom::fill(&mut buf).map_err(|e| maiko::Error::External(e.to_string().into()))?;
        let data = Box::new(buf);
        self.checksum = self
            .checksum
            .wrapping_add(data.iter().map(|b| *b as u64).sum::<u64>());
        self.ctx.send(Event::Data(data)).await?;
        self.cnt -= 1;
        if self.cnt == 0 {
            println!("Producer checksum: {}", self.checksum);
            self.ctx.send(Event::Done).await?;
        }
        Ok(StepAction::Continue)
    }

    fn on_error(&self, error: maiko::Error) -> maiko::Result<()> {
        eprintln!("Producer error: {}", error);
        Ok(())
    }
}

struct Consumer {
    ctx: Context<Event>,
    checksum: u64,
}

impl Actor for Consumer {
    type Event = Event;
    async fn handle_event(&mut self, envelope: &maiko::Envelope<Self::Event>) -> maiko::Result<()> {
        match envelope.event() {
            Event::Done => {
                println!("Consumer checksum: {}", self.checksum);
                self.ctx.stop();
            }
            Event::Data(data) => {
                self.checksum = self
                    .checksum
                    .wrapping_add(data.iter().map(|b| *b as u64).sum::<u64>());
                // simulating slow consumer
                tokio::time::sleep(Duration::from_millis(1)).await;
            }
            _ => (),
        }
        Ok(())
    }

    fn on_error(&self, error: maiko::Error) -> maiko::Result<()> {
        eprintln!("Consumer error: {}", error);
        Ok(())
    }
}

#[tokio::main]
async fn main() -> maiko::Result {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    let mut sup = Supervisor::<Event, Topic>::default();
    sup.add_actor(
        "producer",
        |ctx| Producer {
            ctx,
            cnt: 0,
            checksum: 0,
        },
        &[Topic::Command],
    )?;
    sup.add_actor(
        "consumer",
        |ctx| Consumer { ctx, checksum: 0 },
        &[Topic::Data, Topic::Command],
    )?;
    sup.monitors().add(Tracer).await;

    sup.start().await?;
    sup.send(Event::Start(129)).await?;
    sup.join().await?;
    sup.stop().await?;
    Ok(())
}
