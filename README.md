# Maiko

A lightweight, event-based actor library for Tokio. Unlike many actor libraries (like Actix or Ractor), that are inspired by
Erlang/Akka, Maiko focuses on maximum decoupling and is bringing distributed system patterns
(like event broker, topics, etc) to local actor model.

From code perspective it tries to be non-invasive and lets you implement the code your way (Tokio-first).
What it brings is the runtime that orchestrates communication between actors and supervises the state of actors.
No new concepts to learn: just a few trait methods via `async_trait` and standard Tokio channels.

```rust
use maiko::prelude::*;
use maiko_macros::Event;
use async_trait::async_trait;

#[derive(Clone, Debug, Event)]
enum PingPongEvent { Ping, Pong }

struct PingPong { ctx: Context<PingPongEvent> }

#[async_trait]
impl Actor for PingPong {
    type Event = PingPongEvent;
    async fn handle(&mut self, event: &Self::Event, _meta: &Meta) -> MaikoResult<()> {
        match event {
            PingPongEvent::Ping => self.ctx.send(PingPongEvent::Pong).await?,
            PingPongEvent::Pong => self.ctx.send(PingPongEvent::Ping).await?,
        }
        Ok(())
    }
}

#[tokio::main]
pub async fn main() -> MaikoResult<()> {
    let mut sup = Supervisor::<PingPongEvent>::default();
    sup.add_actor("ping", |ctx| PingPong { ctx }, &[DefaultTopic])?;
    sup.add_actor("pong", |ctx| PingPong { ctx }, &[DefaultTopic])?;

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

- `Event` (derive macro): implement the event trait for your enum without boilerplate.
    - Trait reference in bounds: use `maiko::EventTrait` to avoid name clash with the macro.
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

### Topic mapping example

```rust
use maiko::prelude::*;
use maiko_macros::Event;
use async_trait::async_trait;

#[derive(Clone, Debug, Event)]
enum AppEvent {
    Ping,
    Pong,
    Log(String),
}

#[derive(Debug, Hash, Eq, PartialEq, Clone)]
enum AppTopic { Control, Output }

impl Topic<AppEvent> for AppTopic {
    fn from_event(event: &AppEvent) -> Self {
        match event {
            AppEvent::Ping | AppEvent::Pong => AppTopic::Control,
            AppEvent::Log(_) => AppTopic::Output,
        }
    }
}

struct MyActor { ctx: Context<AppEvent> }

#[async_trait]
impl Actor for MyActor {
    type Event = AppEvent;
    async fn handle(&mut self, event: &Self::Event, _meta: &Meta) -> MaikoResult<()> {
        // Subscribe to topics in Supervisor:
        // sup.add_actor("actor", |ctx| MyActor { ctx }, &[AppTopic::Control]);
        Ok(())
    }
}
```

## Broker

- Central event bus receiving `Envelope<E>` and routing by `Topic<E>`.
- Each actor subscribes to one or more topics; matching events are delivered.
- Backpressure: the broker may use try-send internally for fairness; user code should use `Context::send(...).await` which awaits capacity.

## Edition

This project uses Rust edition `2024`.

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

## Correlation

- **Purpose**: Correlate related events across an interaction (e.g., parent → child).
- **Meta**: Every envelope carries `Meta { id, actor_name, correlation_id }`.
- **Emit with correlation**:
  - Explicit: `ctx.send_with_correlation(event, correlation_id).await?`
  - Child of parent: `ctx.send_child_event(event, &parent_meta).await?` (uses `parent_meta.id()`)

Example:
```rust
async fn handle(&mut self, event: &Self::Event, meta: &Meta) -> Result<()> {
    match event {
        Event::Request(req) => {
            // Do work, then emit a response correlated to the request
            self.ctx.send_child_event(Event::Response(req.id), meta).await?;
        }
        _ => {}
    }
    Ok(())
}
```

## Publishing

This workspace contains two crates:
- `maiko-macros` (proc-macros): publish this crate first.
- `maiko` (runtime): then switch the dependency from a path reference to a version, e.g. `maiko-macros = "0.1.0-alpha"`, and publish.

Crates.io steps:
- `cargo publish -p maiko-macros`
- Edit `maiko/Cargo.toml` to remove `path` from `maiko-macros` dependency.
- `cargo publish -p maiko`

## Macros

- Use `#[derive(Event)]` from `maiko-macros` to implement the `maiko::Event` trait for your event type without boilerplate.
- The derive preserves generics and `where` clauses.
- Recommended import: `use maiko_macros::Event;`
- Example:
```rust
use maiko_macros::Event;

#[derive(Clone, Debug, Event)]
enum MyEvent { Foo, Bar }
```

## Idea

The project is not an out-of-blue idea - it emerged from my own experience while
working on project [Charon](https://github.com/ddrcode/charon) where I designed a system like that.
