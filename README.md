# Maiko

A lightweight, topic-based actor library for Tokio. Unlike many actor libraries (like Actix or Ractor), that are inspired by
Erlang/Akka, Maiko focuses on maximum decoupling and is bringing distributed system patterns
(like event broker, topics, etc) to local actor model.

From code perspective it tries to be non-invasive and lets you implement the code your way (Tokio-first).
What it brings is the runtime that orchestrates communication between actors and supervises the state of actors.
No new concepts to learn: just a few trait methods via `async_trait` and standard Tokio channels.

```rust
#[derive(Clone, Debug)]
enum PingPongEvent { Ping, Pong }
impl Event for PingPongEvent {}

struct PingPong {
    ctx: Context<PingPongEvent>,
}

#[async_trait]
impl Actor for PingPong {
    type Event = PingPongEvent;
    async fn handle(&mut self, event: &Self::Event, _meta: &Meta) -> Result<()> {
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
    sup.add_actor("ping", |ctx| PingPong { ctx }, &[DefaultTopic])?;
    sup.add_actor("pong", |ctx| PingPong { ctx }, &[DefaultTopic])?;

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
- `Supervisor<E,T>`: register actors via `add_actor(name, |ctx| actor, topics)`.
    - `start()`: non-blocking; spawns the broker.
    - `join()`: await actor tasks to finish.
    - `run()`: start + join (blocking until shutdown).
    - `stop()`: request graceful shutdown and await.

### Patterns

Blocking:
```rust
sup.run().await?;
```

Non-blocking service:
```rust
sup.start().await?;
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

## Design Overview

- **Supervisor**: Orchestrates actors and the broker. Adds actors via a factory that receives a `Context<E>`. Provides lifecycle controls: `start()`, `join()`, `run()`, `stop()`.
- **Broker**: Receives `Envelope<E>` on a central channel and routes to subscribers based on `Topic<E>::from_event(&E)`.
- **Actor**: User-implemented type using `async_trait` with hooks: `on_start`, `handle`, `tick`, `on_shutdown`. Uses `Context<E>` to `send` events or `stop` itself.
- **Event**: Marker trait (`Send + Clone`). Your domain events implement this.
- **Topic<E>**: Maps events to routing topics. Use `DefaultTopic` for simple cases, or define your own enum/struct.
- **Envelope/Meta**: Broker wraps events in `Envelope` with `Meta` (id, timestamp, sender).

## Errors & Shutdown

- **Error handling**: If `handle` or `tick` returns an error, `on_error(&Error) -> bool` is called. Return `true` to propagate (terminate the actor task), or `false` to swallow and continue.
- **Graceful stop**: Call `Context::stop()` from inside an actor to mark it not-alive and trigger the cancellation token. The supervisor will then `join()` tasks.
- **Supervisor stop**: `sup.stop().await` requests global cancellation and waits for actor tasks to complete.
- **Backpressure**: Sending via `Context::send` awaits channel capacity. For time-sensitive flows, wrap calls in `tokio::time::timeout` or adopt a try-send pattern in your own code.

## Idea

The project is not an out-of-blue idea - it emerged from my own experience while
working on project [Charon](https://github.com/ddrcode/charon) where I designed a system like that.
