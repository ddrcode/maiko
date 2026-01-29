# Frequently Asked Questions

## What is Maiko?

Maiko is a lightweight actor framework for Tokio. It helps you structure applications as independent components (actors) that communicate purely through events.

Each actor:
- Has its own state and logic
- Runs in its own async task
- Receives events through a private mailbox
- Publishes events without knowing who receives them

Maiko handles all the routing. Actors subscribe to topics, and when an event is published, it's automatically delivered to interested subscribers. No manual channel wiring needed.

## How does Maiko relate to Erlang/OTP and Akka?

Maiko shares the same foundational ideas:

| Concept | Erlang/OTP | Akka | Maiko |
|---------|------------|------|-------|
| Isolated units | Processes | Actors | Actors |
| Communication | Message passing | Message passing | Event pub/sub |
| Addressing | PIDs / registered names | ActorRef | Topics |
| Supervision | Supervision trees | Supervision strategies | Single supervisor |

**Key difference**: Erlang and Akka use direct addressing (you send a message *to* a specific actor). Maiko uses topic-based routing (you publish an event *about* something, and interested actors receive it).

This makes Maiko actors more decoupled - a publisher doesn't need to know who's listening. It's closer to Kafka's model, but for in-process communication.

**What Maiko doesn't have (yet)**:
- Supervision trees and restart strategies (planned)
- Location transparency / distributed actors
- Hot code reloading

## How does Maiko differ from other Rust actor frameworks?

| | Maiko | Actix | Ractor | Stakker |
|---|-------|-------|--------|---------|
| **Routing** | Topic-based pub/sub | Direct addressing | Direct addressing | Direct addressing |
| **Coupling** | Loose (actors don't know each other) | Tight (need actor addresses) | Tight (need actor addresses) | Tight |
| **Communication** | Events | Request-response | Request-response | Messages |
| **Discovery** | Subscribe to topics | Pass addresses manually | Registry lookup | Pass references |
| **Best for** | Event-driven flows | Web services, RPC | Distributed systems | Real-time, games |

**When to choose Maiko over alternatives**:
- Your system is naturally event-driven (sensors, ticks, signals)
- You want loose coupling between components
- You don't need request-response patterns
- You prefer "publish and forget" over "send and await reply"

**When to choose alternatives**:
- You need request-response (Actix, Ractor)
- You need distributed actors across machines (Ractor)
- You need maximum performance with zero allocation (Stakker)

## What's the ideal application for Maiko?

Maiko excels at **event-centric systems** where data flows through independent processors.

**Example: Weather Station**

Imagine a weather monitoring system with multiple sensors. Here's how it maps to Maiko actors:

```
┌─────────────┐  ┌─────────────┐  ┌─────────────┐
│ Temperature │  │  Humidity   │  │   Wind      │
│   Sensor    │  │   Sensor    │  │   Sensor    │
└──────┬──────┘  └──────┬──────┘  └──────┬──────┘
       │                │                │
       └────────────────┼────────────────┘
                        │
                        ▼ Topic: SensorReading
       ┌────────────────┼────────────────┐
       │                │                │
       ▼                ▼                ▼
┌─────────────┐  ┌─────────────┐  ┌─────────────┐
│ Normalizer  │  │  Database   │  │   Alert     │
│  (C° → F°)  │  │   Writer    │  │  Monitor    │
└──────┬──────┘  └─────────────┘  └──────┬──────┘
       │                                 │
       ▼ Topic: NormalizedReading        ▼ Topic: Alert
┌─────────────┐                   ┌─────────────┐
│  Hourly     │                   │   Email     │
│  Reporter   │                   │   Sender    │
└─────────────┘                   └─────────────┘
```

Each box is an actor. They don't know about each other - they just publish events and subscribe to topics. Adding a new sensor? Just register another actor. Want SMS alerts? Add an actor subscribed to `Alert`. The existing code doesn't change.

**Domains where this pattern fits:**

- **IoT / Embedded**: Device signals, sensor readings, hardware events
- **Financial**: Market data feeds, trading signals, price ticks
- **Gaming**: Game events, entity updates, input handling
- **Telemetry**: Log processing, metrics aggregation, monitoring
- **Media**: Audio/video frame processing, effects pipelines

**Real-world example**: [Charon](https://github.com/ddrcode/charon) - a USB keyboard passthrough device. Its daemon uses Maiko actors for keyboard input, key mapping, USB HID output, telemetry, IPC, and power management.

**Not ideal for**:
- Request-response APIs (REST, gRPC)
- RPC-style communication
- Systems where you need "call this actor and get a response"

## How stable is Maiko?

**Current status**: Early-stage, API stabilizing.

| Version | Focus |
|---------|-------|
| 0.1.0 | Core actor model, basic routing |
| 0.2.0 | Monitoring, test harness, API refinements |
| 0.3.0 (planned) | Error handling, supervision, backpressure |
| 1.0.0 (future) | API stability guarantee |

**What this means**:
- Suitable for side projects, prototypes, and non-critical systems
- Used in production by Charon (single real-world deployment)
- API may change between minor versions
- Not recommended for mission-critical production systems yet

**Known limitations**:
- No dynamic actor registration after startup (planned)
- Limited error recovery (actors crash on unhandled errors)
- No backpressure strategy (slow consumers can affect the system)

## How do I test a Maiko application?

Maiko includes a **test harness** (built on the monitoring API) designed for testing actor systems. Enable it with `features = ["test-harness"]`.

The harness lets you:
- **Record events** during a test window
- **Inject events** as if sent by specific actors
- **Query event flow** - who sent what to whom
- **Assert on delivery** - verify events reached the right actors

```rust
#[tokio::test]
async fn test_temperature_alert() -> Result<()> {
    let mut sup = Supervisor::<WeatherEvent, WeatherTopic>::default();

    let sensor = sup.add_actor("sensor", |ctx| TempSensor::new(ctx), Subscribe::none())?;
    let alert = sup.add_actor("alert", |ctx| AlertMonitor::new(ctx), &[WeatherTopic::Reading])?;

    let mut harness = Harness::new(&mut sup).await;
    sup.start().await?;

    // Record what happens when we inject a high temperature reading
    harness.start_recording().await;
    let event_id = harness.send_as(&sensor, WeatherEvent::Temperature(45.0)).await?;
    harness.stop_recording().await;

    // Verify the alert actor received the reading
    assert!(harness.event(event_id).was_delivered_to(&alert));

    // Verify an alert was generated
    let alerts = harness.events().with_topic(&WeatherTopic::Alert);
    assert_eq!(alerts.count(), 1);

    sup.stop().await
}
```

The harness provides spies for inspecting actors, events, and topics:
- `harness.actor(&id)` - what did this actor send/receive?
- `harness.event(id)` - where did this event go?
- `harness.events()` - query all recorded events with filters

See the **[Test Harness Documentation](testing.md)** for the full API.

## Can I add or remove actors at runtime?

Not currently. All actors must be registered before calling `supervisor.start()`. This is a known limitation.

Dynamic actor registration is on the roadmap. Use cases like "spawn an actor per WebSocket connection" aren't supported yet.

## How does error handling work?

Currently minimal. If an actor's `handle_event` returns an error:
1. The `on_error` hook is called (you can override this to recover)
2. By default, the error propagates and the actor stops

Proper supervision (restart strategies, escalation policies) is planned for 0.3.0.

## What about performance?

Maiko is designed for correctness and simplicity first. That said:

- **Event routing**: O(N) over subscribers, filtered by topic. Fast for realistic actor counts (tens to low hundreds).
- **Memory**: Events are `Arc<Envelope<E>>` - zero-copy fan-out to multiple subscribers.
- **Overhead**: One `mpsc` channel per actor, one broker task doing the routing.

For most event-driven applications, Maiko won't be the bottleneck. If you need bare-metal performance with zero allocations, consider Stakker.

## Does Maiko support distributed systems?

Maiko is in-process only - all actors run within a single Tokio runtime. There's no built-in location transparency or actor clustering.

However, events are serializable. Enable the `serde` feature and your events can be sent over the wire. You can build bridge actors that forward events to external systems via IPC, WebSockets, or message queues.

**Real-world example**: Charon uses a bridge actor to expose Maiko events over a Linux Unix socket, allowing external processes to communicate with the daemon.

For true distributed actors across machines with automatic routing, consider Ractor.

## What does "Maiko" mean?

Maiko (舞妓) are apprentice geisha in Kyoto, known for their coordinated traditional dances. Like maiko performers who respond to music and each other in harmony, Maiko actors coordinate through events in the Tokio runtime.

## Can I contribute?

Yes, absolutely! Contributions of all kinds are welcome - bug reports, feature ideas, documentation improvements, or code.

**Ways to get involved**:
- **Try it out** and share your experience
- **Report issues** or suggest improvements via [GitHub Issues](https://github.com/ddrcode/maiko/issues)
- **Pick up a task** from [good first issues](https://github.com/ddrcode/maiko/issues?q=is%3Aissue+state%3Aopen+label%3A%22good+first+issue%22)
- **Ask questions** - if something's unclear, that's a documentation bug

Maiko is a young project and your input genuinely shapes its direction. Whether you're fixing a typo or proposing a major feature, it's appreciated.
