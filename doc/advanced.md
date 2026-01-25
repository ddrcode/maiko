# Advanced Topics

This document covers advanced Maiko features and patterns beyond the basics: error handling strategies, runtime configuration, design philosophy, and performance considerations.

## Error Handling

Control how errors propagate through actors:

```rust
impl Actor for MyActor {
    type Event = MyEvent;

    async fn handle_event(&mut self, envelope: &Envelope<Self::Event>) -> Result<()> {
        // ... processing that might fail
        Ok(())
    }

    fn on_error(&self, error: Error) -> Result<()> {
        match &error {
            Error::SendError(_) => {
                // Channel closed, probably shutting down
                eprintln!("Warning: {}", error);
                Ok(())  // Swallow, continue running
            }
            _ => {
                eprintln!("Fatal: {}", error);
                Err(error)  // Propagate, stop actor
            }
        }
    }
}
```

Returning `Ok(())` from `on_error` swallows the error and continues. Returning `Err(error)` stops the actor.

## Configuration

Fine-tune runtime behavior with `Config`:

```rust
let config = Config::default()
    .with_channel_size(100)           // Event queue size per actor (default: 32)
    .with_max_events_per_tick(50);    // Events processed per tick (default: 10)

let mut sup = Supervisor::new(config);
```

### Configuration Options

| Option | Default | Description |
|--------|---------|-------------|
| `channel_size` | 32 | Buffer size for actor event queues |
| `max_events_per_tick` | 10 | Max events an actor processes before yielding |
| `maintenance_interval` | 1s | How often broker cleans up closed channels |

## Design Philosophy

### Loose Coupling Through Topics

Maiko actors **don't know about each other**. They only know:
- Events they can send
- Topics they subscribe to

This is fundamentally different from traditional actor frameworks:

```rust
// Traditional actors (tight coupling)
actor_ref.tell(message);  // Must know the actor's address

// Maiko (loose coupling)
ctx.send(event).await?;   // Only knows about event types
```

### Unidirectional Flow

Events typically flow in one direction:

```
Input → Parser → Validator → Processor → Output
```

This makes Maiko ideal for **pipeline architectures** and **stream processing**.

### When Maiko Fits

- System event processing (device monitoring, signals, inotify)
- Data pipelines (sensor data, stock ticks, telemetry)
- Game engines (entity systems, input handling)
- Reactive architectures (event sourcing, CQRS)

### When to Consider Alternatives

- **Request-response APIs**: Use Actix Web or similar
- **RPC-style communication**: Use Ractor or Actix
- **Complex supervision trees**: Use Ractor

## Correlation Tracking

Track event causality with correlation IDs:

```rust
async fn handle_event(&mut self, envelope: &Envelope<Self::Event>) -> Result<()> {
    // Check correlation
    if let Some(parent_id) = envelope.meta.correlation_id() {
        println!("This event was triggered by: {}", parent_id);
    }

    // Send a correlated child event
    self.ctx.send_child_event(
        MyEvent::Response(data),
        &envelope.meta  // Links new event to this one
    ).await?;

    Ok(())
}
```

Child events carry their parent's ID as `correlation_id`, enabling tracing of event chains through the system.

## Performance Considerations

### Channel Sizing

- **Too small**: Backpressure, potential deadlocks in hot paths
- **Too large**: Memory overhead, delayed backpressure signals

Start with defaults and tune based on profiling.

### Event Processing Rate

`max_events_per_tick` balances:
- **Higher values**: Better throughput, longer latency for other actors
- **Lower values**: Fairer scheduling, more context switching

### Payload Size

For large payloads, wrap in `Arc`:

```rust
#[derive(Event, Clone, Debug)]
enum DataEvent {
    LargePayload(Arc<Vec<u8>>),  // Cloning is cheap
    SmallPayload(u32),           // Direct is fine
}
```
