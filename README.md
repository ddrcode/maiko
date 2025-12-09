# Maiko

A lightweight, topic-based actor library for Tokio. Unlike many actor libraries (like Actix or Ractor), that are inspired by
Erlang/Akka, Maiko focuses on maximum decoupling and is bringing distributed system patterns
(like event broker, topics, etc) to local actor model.

From code perspective it tries to be non-invasive and lets you implement the code your way (or, well - Tokio way).
What it brings is the runtime that orchestrates communication between actors and supervises the state of actors.
No new concepts to learn: just a few traits to implement and standard Tokio channels.

```rust
#[derive(Clone, Debug)]
enum PingPongEvent {
    Ping,
    Pong,
}
impl Event for PingPongEvent {}

struct PingPong;

#[async_trait]
impl Actor for PingPong {
    type Event = PingPongEvent;

    async fn on_start(&mut self, ctx: &Context<Self::Event>) -> Result<()> {
        if ctx.name() == "ping-side" {
            ctx.send(PingPongEvent::Pong).await?;
        }
        Ok(())
    }

    async fn handle(&mut self, event: &Self::Event, _meta: &Meta) -> Result<Option<Self::Event>> {
        println!("Event: {event:?}");
        match event {
            PingPongEvent::Ping => Ok(Some(PingPongEvent::Pong)),
            PingPongEvent::Pong => Ok(Some(PingPongEvent::Ping)),
        }
    }
}

#[tokio::main]
pub async fn main() -> Result<()> {
    let mut sup = Supervisor::<PingPongEvent, DefaultTopic>::default();
    sup.add_actor("ping-side", PingPong, vec![DefaultTopic])?;
    sup.add_actor("pong-side", PingPong, vec![DefaultTopic])?;
    sup.start()
}
```


## Core Ideas

- Events over messages
- Topics over addresses
- One-directional event flows
- Tokio-first, zero-magic API
- Supervisor orchestration with graceful error handling
- Designed for daemons, bots, UIs, embedded — wherever flows matter

## Idea

The project is not an out-of-blue idea - it emerged from my own experience while
working on project [Charon](https://github.com/ddrcode/charon) where I designed a system like that.
