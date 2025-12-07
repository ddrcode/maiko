use std::time::Duration;

use async_trait::async_trait;
use maiko::prelude::*;
use tokio::time::sleep;

#[derive(Event, Clone, Debug)]
enum GuesserEvent {
    Guess(u8),
    Result(u8, u8),
    Message(String),
}

#[derive(Debug, Hash, Eq, PartialEq)]
enum GuesserTopic {
    Game,
    Output,
}

impl Topic<GuesserEvent> for GuesserTopic {
    fn from_event(event: &GuesserEvent) -> Self {
        use GuesserEvent::*;
        use GuesserTopic::*;
        match event {
            Message(_) => Output,
            Result(..) => Output,
            Guess(_) => Game,
        }
    }
}

struct Guesser {
    name: String,
    cycle_time: Duration,
    ctx: Context<GuesserEvent>,
}

impl Guesser {
    fn new(name: &str, time: u64) -> Self {
        Self {
            name: name.to_string(),
            ctx: Default::default(),
            cycle_time: Duration::from_millis(time),
        }
    }
}

#[async_trait]
impl Actor for Guesser {
    type Event = GuesserEvent;
    fn ctx(&self) -> &Context<Self::Event> {
        &self.ctx
    }
    fn ctx_mut(&mut self) -> &mut Context<Self::Event> {
        &mut self.ctx
    }
    fn set_ctx(&mut self, ctx: Context<Self::Event>) -> maiko::Result<()> {
        self.ctx = ctx;
        Ok(())
    }
    fn name(&self) -> &str {
        &self.name
    }
    async fn tick(&mut self) -> maiko::Result<()> {
        sleep(self.cycle_time).await;
        let guess = rand::random::<u8>() % 10;
        self.send(GuesserEvent::Guess(guess)).await
    }
}

#[derive(Default)]
struct Game {
    ctx: Context<GuesserEvent>,
    number1: Option<u8>,
    number2: Option<u8>,
    count: u64,
}

#[async_trait]
impl Actor for Game {
    type Event = GuesserEvent;
    fn ctx(&self) -> &Context<Self::Event> {
        &self.ctx
    }
    fn ctx_mut(&mut self) -> &mut Context<Self::Event> {
        &mut self.ctx
    }
    fn set_ctx(&mut self, ctx: Context<Self::Event>) -> maiko::Result<()> {
        self.ctx = ctx;
        Ok(())
    }
    fn name(&self) -> &str {
        "game"
    }
    async fn start(&mut self) -> maiko::Result<()> {
        self.send(GuesserEvent::Message(
            "Welcome to the Guessing Game!".to_string(),
        ))
        .await
    }
    async fn handle(
        &mut self,
        event: &Self::Event,
        meta: &Meta,
    ) -> maiko::Result<Option<Self::Event>> {
        if self.count >= 10 {
            self.exit();
        }
        match event {
            GuesserEvent::Guess(guess) => {
                if meta.sender() == "Player1" {
                    self.number1 = Some(*guess);
                } else if meta.sender() == "Player2" {
                    self.number2 = Some(*guess);
                }
                if let (Some(n1), Some(n2)) = (self.number1, self.number2) {
                    self.count += 1;
                    self.number1 = None;
                    self.number2 = None;
                    Ok(Some(GuesserEvent::Result(n1, n2)))
                } else {
                    Ok(None)
                }
            }
            _ => Ok(None),
        }
    }
}

#[derive(Default)]
struct Printer {
    ctx: Context<GuesserEvent>,
}

#[async_trait]
impl Actor for Printer {
    type Event = GuesserEvent;
    fn ctx(&self) -> &Context<Self::Event> {
        &self.ctx
    }
    fn ctx_mut(&mut self) -> &mut Context<Self::Event> {
        &mut self.ctx
    }
    fn set_ctx(&mut self, ctx: Context<Self::Event>) -> maiko::Result<()> {
        self.ctx = ctx;
        Ok(())
    }
    fn name(&self) -> &str {
        "printer"
    }
    async fn handle(
        &mut self,
        event: &Self::Event,
        _meta: &Meta,
    ) -> maiko::Result<Option<Self::Event>> {
        match event {
            GuesserEvent::Message(msg) => {
                println!("{}", msg);
            }
            GuesserEvent::Result(guess, number) if guess == number => {
                println!("Correct guess! The number was {}", number);
            }
            GuesserEvent::Result(guess, number) if guess != number => {
                println!(
                    "Wrong! The number was {} while the guess was {}",
                    number, guess
                );
            }
            _ => {}
        }
        Ok(None)
    }
}

#[tokio::main]
async fn main() -> Result<(), MaikoError> {
    let mut supervisor = Supervisor::<GuesserEvent, GuesserTopic>::default();

    supervisor.add_actor(Guesser::new("Player1", 2000), vec![])?;
    supervisor.add_actor(Guesser::new("Player2", 750), vec![])?;
    supervisor.add_actor(Game::default(), vec![GuesserTopic::Game])?;
    supervisor.add_actor(Printer::default(), vec![GuesserTopic::Output])?;

    supervisor.start().await?;

    Ok(())
}
