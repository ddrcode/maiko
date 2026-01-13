# Maiko

<div align="center">

**Topic-based pub/sub for Tokio**

[![Crates.io](https://img.shields.io/crates/v/maiko.svg)](https://crates.io/crates/maiko)
[![Documentation](https://docs.rs/maiko/badge.svg)](https://docs.rs/maiko)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)

</div>

---

## What is Maiko?

**Maiko** is an in-process event bus for Tokio applications. It provides topic-based pub/sub routing where independent handlers ("actors") subscribe to event topics and communicate through a central broker—similar to Kafka, but lightweight and embedded in your application.

> **Not a traditional actor framework.** Unlike Actix or Ractor (which use addressable actors with request-response messaging), Maiko actors are anonymous, loosely coupled, and communicate only through unidirectional event streams. There are no actor addresses, no supervision trees, and no request-response patterns.

### The Problem Maiko Solves

Building complex Tokio applications often leads to **channel spaghetti**:

```rust
// Without Maiko: Manual channel orchestration
let (tx1, rx1) = mpsc::channel(32);
let (tx2, rx2) = mpsc::channel(32);
let (tx3, rx3) = mpsc::channel(32);

let tx1_clone = tx1.clone();
let tx2_clone = tx2.clone();

tokio::spawn(async move {
    // Task A needs to send to B and C...
    tx1_clone.send(data).await?;
    tx2_clone.send(data).await?;
});

tokio::spawn(async move {
    // Task B needs rx1 and tx3...
    while let Some(msg) = rx1.recv().await {
        tx3.send(process(msg)).await?;
    }
});

// ... and it gets worse with more tasks
```

**With Maiko, channels disappear from your code:**

```rust
// Actors just subscribe to topics - Maiko handles all routing
sup.add_actor("task_a", |ctx| TaskA { ctx }, &[Topic::Input])?;
sup.add_actor("task_b", |ctx| TaskB { ctx }, &[Topic::Processed])?;
sup.add_actor("task_c", |ctx| TaskC { ctx }, &[Topic::Input, Topic::Processed])?;

// No manual channel creation, cloning, or wiring needed!
```

Maiko manages the entire channel topology internally, letting you focus on business logic instead of coordination.

### Why "Maiko"?

**Maiko** (舞妓) are traditional Japanese performers known for their coordinated dances and artistic discipline. Like maiko who respond to music and each other in harmony, Maiko actors coordinate through events in the Tokio runtime.

---

## When to Use Maiko

Maiko excels at **unidirectional event pipelines** where components don't need to know about each other:

- **System event processing** — device monitoring, signals, inotify
- **Data pipelines** — sensor data, stock ticks, telemetry
- **Game engines** — entity systems, input handling
- **Reactive architectures** — event sourcing, CQRS

**Not ideal for:** Request-response APIs, RPC-style communication, complex supervision hierarchies.

### Maiko vs Alternatives

**Feature Comparison:**

| Feature | Maiko | Actix | Ractor | Tokio Channels |
|---------|-------|-------|--------|----------------|
| Pub/Sub Topics | ✅ | ❌ | ❌ | ❌ |
| Actor Addressing | ❌ | ✅ | ✅ | N/A |
| Supervision Trees | ❌ | ✅ | ✅ | N/A |
| Loose Coupling | ✅ | ❌ | ❌ | ✅ |
| Event Metadata | ✅ | ❌ | ❌ | ❌ |
| Correlation Tracking | ✅ | ❌ | ❌ | ❌ |
| Type-Safe Routing | ✅ | ✅ | ✅ | ✅ |
| Learning Curve | Low | Medium | Low | Low |

---

## Quick Start

Add Maiko to your `Cargo.toml`, by executing the following command:

```sh
cargo add maiko
```

### Hello World Example

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

### Other Examples

See the [`examples/`](maiko/examples/) directory for complete programs:

- **[`pingpong.rs`](maiko/examples/pingpong.rs)** - Simple event exchange between actors
- **[`guesser.rs`](maiko/examples/guesser.rs)** - Multi-actor game with topics and timing

Run examples with:

```bash
cargo run --example pingpong
cargo run --example guesser
```

---

## Core Concepts

### 1. Events

Events are messages that flow through the system. They must implement the `Event` trait:

```rust
#[derive(Event, Clone, Debug)]
enum NetworkEvent {
    PacketReceived(Vec<u8>),
    ConnectionClosed(u32),
    Error(String),
}
```

### 2. Topics

Topics route events to interested actors. Define custom topics for fine-grained control:

```rust
#[derive(Debug, Hash, Eq, PartialEq, Clone)]
enum NetworkTopic {
    Ingress,
    Egress,
    Control,
}

impl Topic<NetworkEvent> for NetworkTopic {
    fn from_event(event: &NetworkEvent) -> Self {
        match event {
            NetworkEvent::PacketReceived(_) => NetworkTopic::Ingress,
            NetworkEvent::ConnectionClosed(_) => NetworkTopic::Control,
            NetworkEvent::Error(_) => NetworkTopic::Control,
        }
    }
}
```

Or use `DefaultTopic` to broadcast to all actors:

```rust
sup.add_actor("processor", factory, &[DefaultTopic])?;
```

### 3. Actors

Actors implement two core methods:

- **`handle_event`** — Process incoming events
- **`step`** — Produce events or perform periodic work

```rust
struct PacketProcessor {
    ctx: Context<NetworkEvent>,
    buffer: Vec<u8>,
}

impl Actor for PacketProcessor {
    type Event = NetworkEvent;

    async fn handle_event(&mut self, envelope: &Envelope<Self::Event>) -> Result<()> {
        match envelope.event() {
            NetworkEvent::PacketReceived(data) => {
                self.buffer.extend(data);
            }
            _ => {}
        }
        Ok(())
    }

    async fn step(&mut self) -> Result<StepAction> {
        // Called in select! loop alongside event handling
        if self.buffer.len() > 1000 {
            self.flush_buffer().await?;
        }
        Ok(StepAction::AwaitEvent)  // Wait for next event
    }

    async fn on_start(&mut self) -> Result<()> {
        println!("Processor starting...");
        Ok(())
    }
}
```

### 4. Context

The `Context` provides actors with capabilities:

```rust
// Send events
ctx.send(NetworkEvent::PacketReceived(data)).await?;

// Send with correlation (for tracking related events)
ctx.send_child_event(ResponseEvent::Ok, &envelope.meta).await?;

// Stop this actor
ctx.stop();

// Get actor's name
let name = ctx.name();
```

### 5. Supervisor

The `Supervisor` manages actor lifecycles and provides two registration APIs:

**Simple API** (most common):
```rust
let mut sup = Supervisor::<NetworkEvent>::new();

// Add actors with subscriptions
sup.add_actor("ingress", |ctx| IngressActor::new(ctx), &[NetworkTopic::Ingress])?;
sup.add_actor("egress", |ctx| EgressActor::new(ctx), &[NetworkTopic::Egress])?;
sup.add_actor("monitor", |ctx| MonitorActor::new(ctx), &[DefaultTopic])?;
```

**Builder API** (for advanced configuration):
```rust
// Fine-grained control over actor registration
sup.build_actor::<IngressActor>("ingress")
    .actor(|ctx| IngressActor::new(ctx))
    .topics(&[NetworkTopic::Ingress])
    .build()?;
```

**Runtime control:**
```rust
// Start all actors
sup.start().await?;

// ... application runs ...

// Graceful shutdown
sup.stop().await?;
```


### 6. StepAction

The `step()` method returns `StepAction` to control scheduling:

| Action | Behavior |
|--------|----------|
| `StepAction::Continue` | Run step again immediately |
| `StepAction::Yield` | Yield to runtime, then run again |
| `StepAction::AwaitEvent` | Pause until next event arrives |
| `StepAction::Backoff(Duration)` | Sleep, then run again |
| `StepAction::Never` | Disable step permanently (default) |

**Common patterns:**

```rust
// Event producer with interval
async fn step(&mut self) -> Result<StepAction> {
    self.ctx.send(HeartbeatEvent).await?;
    Ok(StepAction::Backoff(Duration::from_secs(5)))
}

// External I/O source (WebSocket, device, etc.)
async fn step(&mut self) -> Result<StepAction> {
    let data = self.websocket.read().await?;
    self.ctx.send(DataEvent(data)).await?;
    Ok(StepAction::Continue)
}

// Pure event processor (no step logic needed)
async fn step(&mut self) -> Result<StepAction> {
    Ok(StepAction::Never)  // Default behavior
}
```


---

## Design Philosophy

### Loose Coupling Through Topics

Maiko actors **don't know about each other**. They only know about:
- Events they can send
- Topics they subscribe to

This is fundamentally different from Akka/Actix where actors have addresses:

```rust
// Traditional actors (tight coupling)
actor_ref.tell(message);  // Must know the actor's address

// Maiko (loose coupling)
ctx.send(event).await?;   // Only knows about event types
```

### Unidirectional Flow

Events typically flow in one direction:

```
System Event → Parser → Validator → Processor → Logger
```

This makes Maiko ideal for **pipeline architectures** and **stream processing**.

That means Maiko may be not best suited for **request-response** patterns. Although req-resp is possible in theory (with two separate event types), it's not the primary use case, and solutions like [Actix Web](https://actix.rs/) or [Ractor](https://docs.rs/ractor/latest/ractor/) are better suited for this.

---

## Advanced Features

### Envelope & Metadata

Events arrive wrapped in an `Envelope` containing metadata:

```rust
async fn handle_event(&mut self, envelope: &Envelope<Self::Event>) -> Result<()> {
    // Access the event
    match envelope.event() {
        MyEvent::Data(x) => self.process(x).await?,
        _ => {}
    }
    
    // Access metadata when needed
    let sender = envelope.meta.actor_name();
    let correlation = envelope.meta.correlation_id();
    
    // Send correlated child event
    self.ctx.send_child_event(ResponseEvent::Ok, &envelope.meta).await?;
    
    Ok(())
}
```

### Error Handling

Control error propagation:

```rust
impl Actor for MyActor {
    // ...

    fn on_error(&self, error: Error) -> Result<()> {
        match error {
            Error::Recoverable(_) => {
                eprintln!("Warning: {}", error);
                Ok(())  // Swallow error, continue
            }
            Error::Fatal(_) => {
                eprintln!("Fatal: {}", error);
                Err(error)  // Propagate error, stop actor
            }
        }
    }
}
```

### Custom Configuration

Fine-tune actor behavior:

```rust
let config = Config::default()
    .with_channel_size(100)           // Event queue size per actor
    .with_max_events_per_tick(50);    // Events processed per tick cycle

let mut sup = Supervisor::new(config);
```

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

Contributions are welcome! Please feel free to:

- Report bugs via [GitHub Issues](https://github.com/ddrcode/maiko/issues)
- Submit pull requests
- Suggest features and improvements
- Improve documentation

### Code Philosophy

Maiko is **100% human-written code**, crafted with passion for Rust and genuine love for coding. While AI tools have been valuable for architectural discussions, code reviews, and documentation, every line of implementation code comes from human creativity and expertise.

We believe in:
- **Thoughtful design** over automated generation
- **Deep understanding** of the code we write
- **Human craftsmanship** in software engineering

Contributors are expected to write their own code. AI may assist with reviews, discussions, and documentation, but implementations should reflect your own understanding and skills.

Maiko is built with ❤️ by humans, for humans 🦀

---

## Acknowledgments

Inspired by:
- **Kafka** - Topic-based event streaming
- **Akka Streams** - Reactive stream processing
- **Tokio** - Async runtime foundation

---

## License

Licensed under the [MIT License](LICENSE).
