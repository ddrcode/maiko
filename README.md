# Maiko

<div align="center">

**Topic-based pub/sub for Tokio**

[![Crates.io](https://img.shields.io/crates/v/maiko.svg)](https://crates.io/crates/maiko)
[![Documentation](https://docs.rs/maiko/badge.svg)](https://docs.rs/maiko)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)

</div>

---

## What is Maiko?

**Maiko** is a lightweight actor runtime for Tokio applications. Actors are independent units that encapsulate state and communicate through asynchronous events—no shared memory, no locks, no channel wiring. Each actor processes messages sequentially from its own mailbox, making concurrent systems easier to reason about.

Maiko uses **topic-based routing**: actors subscribe to topics, and events are automatically delivered to all interested subscribers. This creates loose coupling—actors don't need to know about each other, only the events they care about.

> **Different from Actix/Ractor.** Traditional actor frameworks use direct addressing (you send messages to a specific actor). Maiko uses pub/sub routing (you publish events to topics). There are no actor addresses, no supervision trees, and no request-response patterns. Think of it as an in-process event bus with actor semantics.

### Why "Maiko"?

**Maiko** (舞妓) are traditional Japanese performers known for their coordinated dances. Like maiko who respond to music and each other in harmony, Maiko actors coordinate through events in the Tokio runtime. And yes, "Maiko" sounds like "my ko" (コ) — a nod to Tokio.

### The Problem Maiko Solves

Building concurrent Tokio applications often leads to **channel spaghetti**—manually creating, cloning, and wiring channels between tasks. With Maiko, channels disappear from your code:

```rust
// Actors just subscribe to topics - Maiko handles all routing
sup.add_actor("task_a", |ctx| TaskA { ctx }, &[Topic::Input])?;
sup.add_actor("task_b", |ctx| TaskB { ctx }, &[Topic::Processed])?;
sup.add_actor("task_c", |ctx| TaskC { ctx }, &[Topic::Input, Topic::Processed])?;

// No manual channel creation, cloning, or wiring needed!
```

### When to Use Maiko

Maiko excels at **unidirectional event pipelines** where components don't need to know about each other:

- **System event processing** — device monitoring, signals, inotify
- **Data pipelines** — sensor data, stock ticks, telemetry
- **Game engines** — entity systems, input handling
- **Reactive architectures** — event sourcing, CQRS

**Not ideal for:** Request-response APIs, RPC-style communication, complex supervision hierarchies.

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

See the [`examples/`](maiko/examples/) directory for complete programs:

- **[`pingpong.rs`](maiko/examples/pingpong.rs)** — Simple event exchange between actors
- **[`guesser.rs`](maiko/examples/guesser.rs)** — Multi-actor game with topics and timing

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
| **Supervisor** | Manages actor lifecycles and runtime |
| **Envelope** | Wraps events with metadata (sender, correlation ID) |
| **ActorId** | Unique identifier for actors, serializable across IPC boundaries |

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

    let mut test = sup.init_test_harness().await;
    sup.start().await?;

    test.start_recording().await;
    let id = test.send_as(&producer, MyEvent::Data(42)).await?;
    test.stop_recording().await;

    assert!(test.event(id).was_delivered_to(&consumer));
    assert_eq!(1, test.actor(&consumer).inbound_count());

    sup.stop().await
}
```

Enable with `features = ["test-harness"]`. See **[Test Harness Documentation](doc/testing.md)** for full details.

---

## Documentation

- **[Core Concepts](doc/concepts.md)** — Events, Topics, Actors, Context, Supervisor
- **[Test Harness](doc/testing.md)** — Recording, spies, queries, assertions
- **[Advanced Topics](doc/advanced.md)** — Error handling, configuration, design philosophy
- **[API Reference](https://docs.rs/maiko)** — Complete API documentation

---

## Roadmap

**Near-term:**
- Dynamic actor spawning/removal at runtime
- Actor introspection and metrics
- Enhanced error handling strategies

**Future:**
- `maiko-actors` crate with reusable actors (IPC bridges, WebSocket, OpenTelemetry)
- Cross-process communication via bridge actors
- Embassy port for embedded systems

---

## Contributing

Contributions welcome! Please feel free to:

- Report bugs via [GitHub Issues](https://github.com/ddrcode/maiko/issues)
- Submit pull requests
- Suggest features and improvements

### Code Philosophy

Maiko is developed through **human-AI collaboration**. The architecture, API design, and key decisions are human-driven, while AI serves as a capable pair programming partner—reviewing code, suggesting improvements, and helping implement well-specified designs.

---

## Acknowledgments

Inspired by [Kafka](https://kafka.apache.org/) (topic-based routing) and built on [Tokio](https://tokio.rs/) (async runtime).

---

## License

Licensed under the [MIT License](LICENSE).
