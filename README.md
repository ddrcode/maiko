# Maiko

<div align="center">

**Topic-based pub/sub for Tokio**

[![Crates.io](https://img.shields.io/crates/v/maiko.svg)](https://crates.io/crates/maiko)
[![Documentation](https://docs.rs/maiko/badge.svg)](https://docs.rs/maiko)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)

</div>

---

## What is Maiko?

**Maiko** is a lightweight [actor](https://en.wikipedia.org/wiki/Actor_model) runtime for Tokio.

Think **Kafka+Microservices, but for Tokio tasks** instead of distributed systems. Actors subscribe to topics, publish events, and Maiko handles all the routing - no manual channel wiring needed.


### How it compares

| | Maiko | Actix/Ractor | Kafka |
|---|-------|--------------|-------|
| Routing | Topic-based pub/sub | Direct addressing | Topic-based pub/sub |
| Coupling | Loose (actors don't know each other) | Tight (need actor addresses) | Loose |
| Communication | Events | Request-response | Events |
| Scope | In-process | In-process | Distributed |


### Where it fits

Event-centric systems:

- System event processing (device monitoring, signals, inotify)
- Data pipelines (sensor data, stock ticks, telemetry)
- Game engines (entity systems, input handling)
- Reactive architectures (event sourcing, CQRS)

Not ideal for request-response APIs or RPC patterns.

### Why "Maiko"?

**Maiko** (舞妓) are traditional Japanese performers known for their coordinated dances. Like maiko who respond to music and each other in harmony, Maiko actors coordinate through events in the Tokio runtime.

---

## The Problem Maiko Solves

Building concurrent Tokio applications often leads to **channel spaghetti** - manually creating, cloning, and wiring channels between tasks:

```rust
// Without Maiko: manual channel wiring
let (tx1, rx1) = mpsc::channel(32);
let (tx2, rx2) = mpsc::channel(32);
let (tx3, rx3) = mpsc::channel(32);
// Clone tx1 for task B, clone tx2 for task C, wire rx1 to...
```

```rust
// With Maiko: declare subscriptions, routing happens automatically
sup.add_actor("sensor",    |ctx| Sensor::new(ctx),    Subscribe::none())?;      // produces events
sup.add_actor("processor", |ctx| Processor::new(ctx), &[Topic::SensorData])?;   // handles sensor data
sup.add_actor("logger",    |ctx| Logger::new(ctx),    Subscribe::all())?;       // observes everything
```

---

## Quick Start

```sh
cargo add maiko
```

```rust
use maiko::*;

#[derive(Event, Clone, Debug)]
enum MyEvent {
    Hello(String),
}

struct Greeter;

impl Actor for Greeter {
    type Event = MyEvent;

    async fn handle_event(&mut self, envelope: &Envelope<Self::Event>) -> Result<()> {
        if let MyEvent::Hello(name) = envelope.event() {
            println!("Hello, {}!", name);
        }
        Ok(())
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let mut sup = Supervisor::<MyEvent>::default();
    sup.add_actor("greeter", |_ctx| Greeter, &[DefaultTopic])?;

    sup.start().await?;
    sup.send(MyEvent::Hello("World".into())).await?;
    sup.stop().await
}
```

### Examples

See the [`examples/`](maiko/examples/) directory:

- **[`pingpong.rs`](maiko/examples/pingpong.rs)** - Event exchange between actors
- **[`guesser.rs`](maiko/examples/guesser.rs)** - Multi-actor game with topics and timing
- **[`monitoring.rs`](maiko/examples/monitoring.rs)** - Observing event flow

```bash
cargo run --example pingpong
cargo run --example guesser
```

---

## Core Concepts

| Concept | Description |
|---------|-------------|
| **Event** | Messages that flow through the system (`#[derive(Event)]`) |
| **Topic** | Routes events to interested actors |
| **Actor** | Processes events via `handle_event()` and produces events via `step()` |
| **Context** | Provides actors with `send()`, `stop()`, and metadata access |
| **Supervisor** | Manages actor lifecycles and the runtime |
| **Envelope** | Wraps events with metadata (sender, correlation ID) |

For detailed documentation, see **[Core Concepts](doc/concepts.md)**.

---

## Test Harness

Maiko includes a test harness for observing and asserting on event flow:

```rust
#[tokio::test]
async fn test_event_delivery() -> Result<()> {
    let mut sup = Supervisor::<MyEvent>::default();
    let producer = sup.add_actor("producer", |ctx| Producer::new(ctx), &[DefaultTopic])?;
    let consumer = sup.add_actor("consumer", |ctx| Consumer::new(ctx), &[DefaultTopic])?;

    let mut test = Harness::new(&mut sup).await;
    sup.start().await?;

    test.start_recording().await;
    let id = test.send_as(&producer, MyEvent::Data(42)).await?;
    test.stop_recording().await;

    assert!(test.event(id).was_delivered_to(&consumer));
    assert_eq!(1, test.actor(&consumer).inbound_count());

    sup.stop().await
}
```

Enable with `features = ["test-harness"]`. See **[Test Harness Documentation](doc/testing.md)** for details.

---

## Monitoring

The monitoring API provides hooks into the event lifecycle - useful for debugging, metrics, and logging:

```rust
use maiko::monitoring::Monitor;

struct EventLogger;

impl<E: Event, T: Topic<E>> Monitor<E, T> for EventLogger {
    fn on_event_handled(&self, envelope: &Envelope<E>, topic: &T, receiver: &ActorId) {
        println!("[handled] {} by {}", envelope.id(), receiver.name());
    }
}

let handle = sup.monitors().add(EventLogger).await;
```

Enable with `features = ["monitoring"]`. See **[Monitoring Documentation](doc/monitoring.md)** for details.

---

## Documentation

- **[Core Concepts](doc/concepts.md)** - Events, Topics, Actors, Context, Supervisor
- **[Monitoring](doc/monitoring.md)** - Event lifecycle hooks, metrics collection
- **[Test Harness](doc/testing.md)** - Recording, spies, queries, assertions
- **[Advanced Topics](doc/advanced.md)** - Error handling, configuration, design philosophy
- **[API Reference](https://docs.rs/maiko)** - Complete API documentation

---

## Roadmap

**Near-term:**
- Dynamic actor spawning/removal at runtime
- Improved supervision and error handling strategies

**Future:**
- `maiko-actors` crate with reusable actors (IPC bridges, WebSocket, OpenTelemetry)
- Cross-process communication via bridge actors

---

## Used In

Maiko powers the daemon in [**Charon**](https://github.com/ddrcode/charon) - a USB keyboard pass-through device built on Raspberry Pi. The daemon uses Maiko actors to read input from multiple keyboards, map and translate key events, output USB HID to the host, and coordinate telemetry, IPC, and power management.

---

## Contributing

Contributions welcome! Whether it's a bug report, feature idea, or pull request - all input is appreciated.

- **Try it out** and let us know what you think
- **Report issues** via [GitHub Issues](https://github.com/ddrcode/maiko/issues)
- **Looking to contribute code?** Check out [good first issues](https://github.com/ddrcode/maiko/issues?q=is%3Aissue+state%3Aopen+label%3A%22good+first+issue%22)

---

## Acknowledgments

Inspired by [Kafka](https://kafka.apache.org/) (topic-based routing) and built on [Tokio](https://tokio.rs/) (async runtime).

---

## License

Licensed under the [MIT License](LICENSE).
