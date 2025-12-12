# Maiko

A lightweight, topic-based actor library for Tokio. Unlike many actor libraries (like Actix or Ractor), that are inspired by
Erlang/Akka, Maiko focuses on maximum decoupling and is bringing distributed system patterns
(like event broker, topics, etc) to local actor model.

From code perspective it tries to be non-invasive and lets you implement the code your way (or, well - Tokio way).
What it brings is the runtime that orchestrates communication between actors and supervises the state of actors.
No new concepts to learn: just a few trait methods and standard Tokio channels.

```rust
#[derive(Clone, Debug)]
enum PingPongEvent { Ping, Pong }
impl Event for PingPongEvent {}

struct PingPong;

#[async_trait]
impl Actor for PingPong {
    type Event = PingPongEvent;
    async fn handle(&mut self, event: &Self::Event, _meta: &Meta) -> Result<()> {
        match event { PingPongEvent::Ping => println!("Ping"), PingPongEvent::Pong => println!("Pong") }
        Ok(())
    }
}

#[tokio::main]
pub async fn main() -> Result<()> {
    let mut sup = Supervisor::<PingPongEvent, DefaultTopic>::default();
    sup.add_actor(PingPong, vec![DefaultTopic])?;
    sup.add_actor(PingPong, vec![DefaultTopic])?;

    // Blocking: start + await shutdown
    sup.run().await?;
    Ok(())
}
```


## Core Ideas

- Events over messages
- Topics over addresses
- One-directional event flows
- Tokio-first, zero-magic API
- Supervisor orchestration with graceful error handling
- Designed for daemons, bots, UIs, embedded — wherever flows matter

## API Sketch

- `Event`: marker trait implemented for your event enum (derive macro available).
- `Topic<E>`: maps events to topics for routing.
- `Actor`: implement `handle`, optionally `tick`, `on_start`, `on_shutdown`.
- `Supervisor<E,T>`: register actors via `add_actor(actor, topics)`.
    - `start()`: non-blocking; spawns the broker.
    - `join()`: await broker and actors to finish.
    - `run()`: start + join (blocking until shutdown).
    - `stop()`: request graceful shutdown and await.

### Patterns

Blocking:
```rust
sup.run().await?;
```

Non-blocking service:
```rust
sup.start()?;
// do other work
sup.stop().await?;
```

With timeout:
```rust
use tokio::{select, time::{sleep, Duration}};
select! {
        _ = sup.run() => {},
        _ = sleep(Duration::from_secs(5)) => sup.stop().await?,
}
```

See `examples/` for runnable demos.

## Idea

The project is not an out-of-blue idea - it emerged from my own experience while
working on project [Charon](https://github.com/ddrcode/charon) where I designed a system like that.
