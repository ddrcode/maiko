# Core Concepts

Maiko is built around a small set of core abstractions that work together:

- **Events** are messages that flow through the system
- **Topics** determine which actors receive which events
- **Actors** are independent units that process events and maintain state
- **Context** allows actors to send events and interact with the runtime
- **Supervisor** manages actor lifecycles and coordinates the system
- **Envelopes** wrap events with metadata for tracing and correlation

This document covers each abstraction in detail.

## Events

Events are messages that flow through the system. They must implement the `Event` trait (via derive macro):

```rust
#[derive(Event, Clone, Debug)]
enum NetworkEvent {
    PacketReceived(Vec<u8>),
    ConnectionClosed(u32),
    Error(String),
}
```

Events should be:
- **Cloneable** — events may be delivered to multiple actors
- **Debuggable** — for logging and diagnostics
- **Lightweight** — prefer `Arc<T>` for large payloads

## Topics

Topics route events to interested actors. Each event maps to exactly one topic, determined by the `Topic::from_event()` implementation.

### Custom Topics

Define custom topics for fine-grained routing:

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

### DefaultTopic

Use `DefaultTopic` when you don't need routing — all events go to all subscribed actors:

```rust
sup.add_actor("processor", factory, &[DefaultTopic])?;
```

## Actors

Actors are independent units that process events. They implement the `Actor` trait with two core methods:

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
        if self.buffer.len() > 1000 {
            self.flush_buffer().await?;
        }
        Ok(StepAction::AwaitEvent)
    }
}
```

### Lifecycle Hooks

Actors can implement optional lifecycle methods:

- **`on_start`** — Called once when the actor starts
- **`on_shutdown`** — Called during graceful shutdown
- **`on_error`** — Handle errors (swallow or propagate)

```rust
async fn on_start(&mut self) -> Result<()> {
    println!("Actor starting...");
    Ok(())
}

fn on_error(&self, error: Error) -> Result<()> {
    eprintln!("Error: {}", error);
    Ok(())  // Swallow error, continue running
}
```

## Context

The `Context` provides actors with capabilities to interact with the system. **Context is optional**—actors that only consume events (without sending) don't need to store it:

```rust
// Pure consumer - no context needed
struct Logger;

impl Actor for Logger {
    type Event = MyEvent;

    async fn handle_event(&mut self, envelope: &Envelope<Self::Event>) -> Result<()> {
        println!("Received: {:?}", envelope.event());
        Ok(())
    }
}

// Producer/processor - stores context to send events
struct Processor {
    ctx: Context<MyEvent>,
}
```

Context capabilities:

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

## Supervisor

The `Supervisor` manages actor lifecycles and provides registration APIs.

### Simple API

```rust
let mut sup = Supervisor::<NetworkEvent>::new();

sup.add_actor("ingress", |ctx| IngressActor::new(ctx), &[NetworkTopic::Ingress])?;
sup.add_actor("egress", |ctx| EgressActor::new(ctx), &[NetworkTopic::Egress])?;
sup.add_actor("monitor", |ctx| MonitorActor::new(ctx), &[DefaultTopic])?;
```

### Builder API

For advanced configuration:

```rust
sup.build_actor::<IngressActor>("ingress")
    .actor(|ctx| IngressActor::new(ctx))
    .topics(&[NetworkTopic::Ingress])
    .build()?;
```

### Runtime Control

```rust
// Start all actors (non-blocking)
sup.start().await?;

// Wait for completion
sup.join().await?;

// Or combine both
sup.run().await?;

// Graceful shutdown
sup.stop().await?;

// Send events from outside
sup.send(MyEvent::Data(42)).await?;
```

## StepAction

The `step()` method returns `StepAction` to control scheduling:

| Action | Behavior |
|--------|----------|
| `StepAction::Continue` | Run step again immediately |
| `StepAction::Yield` | Yield to runtime, then run again |
| `StepAction::AwaitEvent` | Pause until next event arrives |
| `StepAction::Backoff(Duration)` | Sleep, then run again |
| `StepAction::Never` | Disable step permanently (default) |

### Common Patterns

**Event producer with interval:**
```rust
async fn step(&mut self) -> Result<StepAction> {
    self.ctx.send(HeartbeatEvent).await?;
    Ok(StepAction::Backoff(Duration::from_secs(5)))
}
```

**External I/O source:**
```rust
async fn step(&mut self) -> Result<StepAction> {
    let data = self.websocket.read().await?;
    self.ctx.send(DataEvent(data)).await?;
    Ok(StepAction::Continue)
}
```

**Pure event processor (default):**
```rust
async fn step(&mut self) -> Result<StepAction> {
    Ok(StepAction::Never)
}
```

## Envelope & Metadata

Events arrive wrapped in an `Envelope` containing metadata:

```rust
async fn handle_event(&mut self, envelope: &Envelope<Self::Event>) -> Result<()> {
    // Access the event
    let event = envelope.event();

    // Access metadata
    let sender = envelope.meta.actor_name();
    let event_id = envelope.meta.id();
    let correlation = envelope.meta.correlation_id();

    // Send correlated child event
    self.ctx.send_child_event(ResponseEvent::Ok, &envelope.meta).await?;

    Ok(())
}
```

Correlation tracking enables tracing event causality through the system.
