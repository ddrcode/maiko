# Maiko

A lightweight, topic-based actor library for Tokio. Unlike many actor libraries (like Actix or Ractor), that are inspired by
Erlang/Akka, Maiko focuses and maximum decoupling and is bringing distributed system patterns
(like event broker, topics, etc) to local actor model.

From code perspective it tries to be non-invasive and lets you implement the code your way (or, well - Tokio way).
What it brings is the runtime that orchestrates communication between actors and supervises the state of actors. 
Noe new concepts to learn: just a few traits to implement and standard Tokio channels. 

```rust
#[derive(Debug)]
enum Event {
   Ping,
   Pong
}

impl maiko::Event for Event {}

struct PingPongActor {}

impl maiko::Actor<Event> for PingPongActor {
   async fn handle(&mut self, event: Event) -> maiko::Result<()> {
      println!("Event: {event:?}");
      match event {
         Event::Ping => self.send(Event::Pong).await,
         Event::Pong => self.send(Event::Ping).await,
      }
      Ok(())
   }
}

let sup = maiko::Supervisor::new()
   .spawn("pp1", PingPongActor{})
   .spawn("pp2", PingPongActor{})
   .run().await;

sup.broker().send_to("pp1", Event::Ping);
tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
sup.stop().await;
```
