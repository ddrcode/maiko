# Maiko

<div align="center">

**Event-driven actors for Tokio**

[![Crates.io](https://img.shields.io/crates/v/maiko.svg)](https://crates.io/crates/maiko)
[![Documentation](https://docs.rs/maiko/badge.svg)](https://docs.rs/maiko)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)

</div>

---

## What is Maiko?

**Maiko** is a lightweight actor runtime for building event-driven concurrent systems in Rust. Unlike traditional actor frameworks, Maiko actors communicate through **topic-based pub/sub** rather than direct addressing, making them loosely coupled and ideal for stream processing workloads.

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

Maiko excels at processing **unidirectional event streams** where actors don't need to know about each other:

- **System event processing** - inotify, epoll, signals, device monitoring
- **Data stream handling** - stock ticks, sensor data, telemetry pipelines
- **Network event processing** - packet handling, protocol parsing
- **Reactive architectures** - event sourcing, CQRS patterns
- **Game engines** - entity systems, event-driven gameplay

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

**Use Case Guide:**

| Use Case | Best Choice |
|----------|-------------|
| Event streams, pub/sub patterns | **Maiko** ✅ |
| Request/reply, actor addressing | Actix, Ractor |
| Supervision trees, fault tolerance | Actix, Ractor |
| Simple async tasks | Tokio tasks |
| Distributed messaging | Kafka, NATS |

---

## Quick Start

Add Maiko to your `Cargo.toml`:

```toml
[dependencies]
maiko = "0.1.0"
tokio = { version = "1", features = ["full"] }
```

### Hello World Example

```rust
use maiko::*;

// Define your events
#[derive(Event, Clone, Debug)]
enum MyEvent {
    Hello(String),
}

// Create an actor
struct Greeter;

impl Actor for Greeter {
    type Event = MyEvent;

    async fn handle(&mut self, event: &Self::Event, meta: &Meta) -> Result<()> {
        match event {
            MyEvent::Hello(name) => {
                println!("Hello, {}! (from {})", name, meta.actor_name());
            }
        }
        Ok(())
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let mut sup = Supervisor::<MyEvent>::default();

    // Add actor and subscribe it to all topics (Broadcast)
    sup.add_actor("greeter", |_ctx| Greeter, &[Broadcast])?;

    // Start the supervisor and send a message
    sup.start().await?;
    sup.send(MyEvent::Hello("World".into())).await?;

    // Graceful shutdown (it attempts to process all events already in the queue)
    sup.stop().await?;
    Ok(())
}
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

Or use `Broadcast` to broadcast to all actors:

```rust
sup.add_actor("processor", factory, &[Broadcast])?;
```

### 3. Actors

Actors are independent units that process events asynchronously:

```rust
struct PacketProcessor {
    ctx: Context<NetworkEvent>,
    stats: PacketStats,
}

impl Actor for PacketProcessor {
    type Event = NetworkEvent;

    async fn on_start(&mut self) -> Result<()> {
        println!("Processor starting...");
        Ok(())
    }

    async fn handle(&mut self, event: &Self::Event, meta: &Meta) -> Result<()> {
        match event {
            NetworkEvent::PacketReceived(data) => {
                self.stats.increment();
                self.process_packet(data).await?;
            }
            _ => {}
        }
        Ok(())
    }

    async fn tick(&mut self) -> Result<()> {
        // Called periodically when no events are queued
        // Useful for polling external sources, timeouts, housekeeping, etc.
        if self.stats.should_report() {
            println!("Processed {} packets", self.stats.count);
        }
        Ok(())
    }

    async fn on_stop(&mut self) -> Result<()> {
        println!("Final count: {}", self.stats.count);
        Ok(())
    }
}
```

### 4. Context

The `Context` provides actors with capabilities:

```rust
// Send events to topics
ctx.send(NetworkEvent::PacketReceived(data)).await?;

// Send with correlation (for tracking related events)
ctx.send_child_event(NetworkEvent::Response(data)).await?;

// Check if system is still running
if !ctx.is_alive() {
    return Ok(());
}

// Get actor's name
let name = ctx.name();
```

### 5. Actor Patterns: Handle vs Tick

Maiko actors typically follow one of two patterns:

**Handle-Heavy Actors** (Event Processors):
```rust
// Telemetry actor - processes many incoming events
impl Actor for TelemetryCollector {
    type Event = MetricEvent;

    async fn handle(&mut self, event: &Self::Event, _meta: &Meta) -> Result<()> {
        // Main logic here - process incoming metrics
        self.export_to_otel(event).await?;
        self.aggregate(event);
        Ok(())
    }

    async fn tick(&mut self) -> Result<()> {
        // Minimal - just periodic cleanup
        self.flush_buffer().await
    }
}
```

**Tick-Heavy Actors** (Event Producers):
```rust
// Stock data reader - polls external source, emits many events
impl Actor for StockReader {
    type Event = StockEvent;

    async fn handle(&mut self, event: &Self::Event, _meta: &Meta) -> Result<()> {
        // Rarely receives events - maybe just control messages
        if let StockEvent::Stop = event {
            self.websocket.close().await?;
        }
        Ok(())
    }

    async fn tick(&mut self) -> Result<()> {
        // Main logic here - poll WebSocket and emit events
        if let Some(tick) = self.websocket.next().await {
            let event = StockEvent::Tick(tick.symbol, tick.price);
            self.ctx.send(event).await?;
        }
        Ok(())
    }
}

// System event monitor - polls inotify/epoll
impl Actor for SystemMonitor {
    type Event = SystemEvent;

    async fn tick(&mut self) -> Result<()> {
        // Poll for file system events
        for event in self.inotify.read_events()? {
            self.ctx.send(SystemEvent::FileChanged(event.path)).await?;
        }
        Ok(())
    }
}
```

The `tick()` method runs when the event queue is empty, making it perfect for:
- **Polling external sources** (WebSockets, file descriptors, system APIs)
- **Periodic tasks** (metrics reporting, health checks)
- **Timeout logic** (detecting stale connections)
- **Housekeeping** (buffer flushing, cache cleanup)

### 6. Supervisor

The `Supervisor` manages actor lifecycles:

```rust
let mut sup = Supervisor::<NetworkEvent>::new();

// Add actors with subscriptions
sup.add_actor("ingress", |ctx| IngressActor::new(ctx), &[NetworkTopic::Ingress])?;
sup.add_actor("egress", |ctx| EgressActor::new(ctx), &[NetworkTopic::Egress])?;
sup.add_actor("monitor", |ctx| MonitorActor::new(ctx), &[Broadcast])?;

// Start all actors
sup.start().await?;

// ... application runs ...

// Graceful shutdown
sup.stop().await?;
```

---

## Design Philosophy

### Loose Coupling Through Topics

Maiko actors **don't know about each other**. They only know about:
- Events they can send
- Topics they subscribe to

This is fundamentally different from Akka/Actix where actors have addresses:

```rust
// ❌ Traditional actors (tight coupling)
actor_ref.tell(message);  // Must know the actor's address

// ✅ Maiko (loose coupling)
ctx.send(event).await?;   // Only knows about event types
```

### Unidirectional Flow

Events typically flow in one direction:

```
System Event → Parser → Validator → Processor → Logger
```

This makes Maiko ideal for **pipeline architectures** and **stream processing**.

### Type Safety

Event types are checked at compile time. Invalid event routing is impossible:

```rust
// Compile error - wrong event type!
ctx.send(DifferentEvent::Foo).await?;
```

---

## Advanced Features

### Correlation IDs

Track related events across actors:

```rust
async fn handle(&mut self, event: &Self::Event, meta: &Meta) -> Result<()> {
    if let Some(correlation_id) = meta.correlation_id() {
        println!("Event chain: {}", correlation_id);
    }

    // Child events inherit correlation
    self.ctx.send_child_event(ResponseEvent::Ok).await?;

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

## Examples

See the [`examples/`](maiko/examples/) directory for complete programs:

- **[pingpong.rs](maiko/examples/pingpong.rs)** - Simple event exchange between actors
- **[guesser.rs](maiko/examples/guesser.rs)** - Multi-actor game with topics and timing

Run examples with:

```bash
cargo run --example pingpong
cargo run --example guesser
```

---

## Roadmap

### v0.1.0 (Current - Single Process)
- Topic-based event routing
- Async actor lifecycle hooks
- Graceful shutdown via cancellation tokens
- Correlation ID tracking
- Flexible error handling

### v0.2.0 (Supervision & Control)
- Actor restart policies and strategies
- Supervisor metrics and monitoring
- Dynamic actor spawning at runtime
- Backpressure configuration
- Enhanced error recovery

### v0.3.0 (Cross-Process Communication)
- IPC bridge actors (Unix sockets, TCP)
- Event serialization framework (bincode, JSON, protobuf)
- Remote topic subscriptions
- Multi-supervisor coordination
- Process-level fault isolation

### v0.4.0+ (Ready-to-Use Actor Library)
- **Inter-supervisor communication** - Unix socket, gRPC bridge actors
- **Networking actors** - HTTP client/server, WebSocket, TCP/UDP handlers
- **Telemetry actors** - OpenTelemetry integration, metrics exporters
- **Storage actors** - Database connectors, file watchers, cache adapters
- Authentication and encryption for network bridges

---

## Architecture

```
┌─────────────────────────────────────────────────────┐
│                    Supervisor                       │
│  ┌────────────────────────────────────────────────┐ │
│  │              Central Broker                    │ │
│  │  (Topic-based routing & filtering)             │ │
│  └────────────────────────────────────────────────┘ │
│         │              │              │             │
│    ┌────▼───┐     ┌────▼───┐     ┌────▼───┐         │
│    │Actor A │     │Actor B │     │Actor C │         │
│    │        │     │        │     │        │         │
│    │Topics: │     │Topics: │     │Topics: │         │
│    │  [T1]  │     │[T1,T2] │     │  [T2]  │         │
│    └────┬───┘     └───┬────┘     └───┬────┘         │
│         │             │              │              │
│         └─────────────┴──────────────┘              │
│              Events flow up to broker               │
└─────────────────────────────────────────────────────┘
```

**Key Components:**

- **Supervisor** - Manages actor lifecycles and shutdown
- **Broker** - Routes events to subscribed actors based on topics
- **Actors** - Independent event processors with isolated state
- **Context** - Actor's interface to send events and check system state

---

## Contributing

Contributions are welcome! Please feel free to:

- Report bugs via [GitHub Issues](https://github.com/ddrcode/maiko/issues)
- Suggest features and improvements
- Improve documentation
- Submit pull requests

### Code Philosophy

Maiko is **100% human-written code**, crafted with passion for Rust and genuine love for coding. While AI tools have been valuable for architectural discussions, code reviews, and documentation, every line of implementation code comes from human creativity and expertise.

We believe in:
- **Thoughtful design** over automated generation
- **Deep understanding** of the code we write
- **Human craftsmanship** in software engineering

Contributors are expected to write their own code. AI may assist with reviews, discussions, and documentation, but implementations should reflect your own understanding and skills.

---

## License

Licensed under the [MIT License](LICENSE).

---

## Acknowledgments

Inspired by:
- **Kafka** - Topic-based event streaming
- **Akka Streams** - Reactive stream processing
- **Tokio** - Async runtime foundation

Built with ❤️ and by humans, for humans 🦀
