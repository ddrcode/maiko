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

#[derive(Debug, Hash, Eq, PartialEq, Clone)]
enum GuesserTopic {
    Game,
    Output,
}

impl Topic<GuesserEvent> for GuesserTopic {
    fn from_event(event: &GuesserEvent) -> Self {
        use GuesserEvent::*;
        use GuesserTopic::*;

        match event {
            Message(..) => Output,
            Result(..) => Output,
            Guess(..) => Game,
        }
    }
}

struct Guesser {
    ctx: Context<GuesserEvent>,
    cycle_time: Duration,
}

impl Guesser {
    fn new(ctx: Context<GuesserEvent>, time: u64) -> Self {
        Self {
            ctx,
            cycle_time: Duration::from_millis(time),
        }
    }
}

#[async_trait]
impl Actor for Guesser {
    type Event = GuesserEvent;

    async fn tick(&mut self) -> maiko::Result<()> {
        sleep(self.cycle_time).await;
        let guess = rand::random::<u8>() % 10;
        self.ctx.send(GuesserEvent::Guess(guess)).await
    }
}

struct Game {
    ctx: Context<GuesserEvent>,
    number1: Option<u8>,
    number2: Option<u8>,
    count: u64,
}

impl Game {
    fn new(ctx: Context<GuesserEvent>) -> Self {
        Self {
            ctx,
            number1: None,
            number2: None,
            count: 0,
        }
    }
}

#[async_trait]
impl Actor for Game {
    type Event = GuesserEvent;

    async fn on_start(&mut self) -> maiko::Result<()> {
        self.ctx
            .send(GuesserEvent::Message(
                "Welcome to the Guessing Game!\n(the game will stop after 10 attempts)".to_string(),
            ))
            .await
    }

    async fn handle(&mut self, event: &Self::Event, meta: &Meta) -> maiko::Result<()> {
        if let GuesserEvent::Guess(guess) = event {
            if meta.actor_name() == "Player1" {
                self.number1 = Some(*guess);
            } else if meta.actor_name() == "Player2" {
                self.number2 = Some(*guess);
            }

            if let (Some(n1), Some(n2)) = (self.number1, self.number2) {
                self.count += 1;
                self.number1 = None;
                self.number2 = None;
                self.ctx.send(GuesserEvent::Result(n1, n2)).await?;
            }
        }

        Ok(())
    }

    async fn tick(&mut self) -> maiko::Result<()> {
        if self.count >= 10 {
            self.ctx.stop();
        }

        Ok(())
    }
}

struct Printer;

#[async_trait]
impl Actor for Printer {
    type Event = GuesserEvent;

    async fn handle(&mut self, event: &Self::Event, _meta: &Meta) -> maiko::Result<()> {
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

        Ok(())
    }
}

#[tokio::main]

async fn main() -> Result<(), MaikoError> {
    let mut supervisor = Supervisor::<GuesserEvent, GuesserTopic>::default();

    supervisor.add_actor("Player1", |ctx| Guesser::new(ctx, 500), &[])?;
    supervisor.add_actor("Player2", |ctx| Guesser::new(ctx, 350), &[])?;
    supervisor.add_actor("Game", Game::new, &[GuesserTopic::Game])?;
    supervisor.add_actor("Printer", |_| Printer, &[GuesserTopic::Output])?;

    supervisor.run().await?;
    println!("Game over!");
    Ok(())
}
