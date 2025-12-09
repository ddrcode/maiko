use std::time::Duration;

use async_trait::async_trait;
use maiko::prelude::*;
use tokio::time::sleep;

#[derive(Clone, Debug)]
enum GuesserEvent {
    Guess(u8),
    Result(u8, u8),
    Message(String),
}
impl Event for GuesserEvent {}

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
    cycle_time: Duration,
}

impl Guesser {
    fn new(time: u64) -> Self {
        Self {
            cycle_time: Duration::from_millis(time),
        }
    }
}

#[async_trait]
impl Actor for Guesser {
    type Event = GuesserEvent;
    async fn tick(&mut self, ctx: &Context<Self::Event>) -> maiko::Result<()> {
        sleep(self.cycle_time).await;
        let guess = rand::random::<u8>() % 10;
        ctx.send(GuesserEvent::Guess(guess)).await
    }
}

#[derive(Default)]
struct Game {
    number1: Option<u8>,
    number2: Option<u8>,
    count: u64,
}

#[async_trait]
impl Actor for Game {
    type Event = GuesserEvent;
    async fn on_start(&mut self, ctx: &Context<Self::Event>) -> maiko::Result<()> {
        ctx.send(GuesserEvent::Message(
            "Welcome to the Guessing Game!".to_string(),
        ))
        .await
    }
    async fn handle(
        &mut self,
        event: &Self::Event,
        meta: &Meta,
    ) -> maiko::Result<Option<Self::Event>> {
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
    async fn tick(&mut self, ctx: &Context<Self::Event>) -> maiko::Result<()> {
        if self.count >= 10 {
            ctx.stop();
        }
        Ok(())
    }
}

struct Printer;

#[async_trait]
impl Actor for Printer {
    type Event = GuesserEvent;
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

    supervisor.add_actor("Player1", Guesser::new(2000), vec![])?;
    supervisor.add_actor("Player2", Guesser::new(750), vec![])?;
    supervisor.add_actor("Game", Game::default(), vec![GuesserTopic::Game])?;
    supervisor.add_actor("Printer", Printer, vec![GuesserTopic::Output])?;

    supervisor.start().await?;

    Ok(())
}
