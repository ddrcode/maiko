# Test Harness

Maiko provides a test harness for observing and asserting on event flow. Enable it with the `test-harness` feature:

```toml
[dev-dependencies]
maiko = { version = "0.2", features = ["test-harness"] }
```

## Overview

The test harness enables:
- **Event recording** — capture all event deliveries during a test
- **Event injection** — send events as if they came from specific actors
- **Spies** — inspect events from different perspectives (event, actor, topic)
- **Queries** — filter and search recorded events

## Basic Usage

```rust
#[tokio::test]
async fn test_event_flow() -> Result<()> {
    let mut sup = Supervisor::<MyEvent>::default();
    let producer = sup.add_actor("producer", |ctx| Producer::new(ctx), &[DefaultTopic])?;
    let consumer = sup.add_actor("consumer", |ctx| Consumer::new(ctx), &[DefaultTopic])?;

    // Initialize harness BEFORE starting
    let mut test = Harness::new(&mut sup).await;
    sup.start().await?;

    // Record events
    test.start_recording().await;
    let id = test.send_as(&producer, MyEvent::Data(42)).await?;
    test.stop_recording().await;

    // Assert on recorded events
    assert!(test.event(id).was_delivered_to(&consumer));
    assert_eq!(1, test.actor(&consumer).events_received());

    sup.stop().await
}
```

## Recording Control

```rust
// Start recording events
test.start_recording().await;

// ... send events, run test scenario ...

// Stop recording and capture snapshot
test.stop_recording().await;

// Clear recorded events for next test phase
test.reset().await;
```

## Event Injection

Send events as if they came from a specific actor:

```rust
let event_id = test.send_as(&producer, MyEvent::Data(42)).await?;
```

The returned `event_id` can be used to inspect the event's delivery.

## Settling

`stop_recording()` automatically calls `settle()` internally, so for most tests you don't need to call it explicitly:

```rust
test.start_recording().await;
test.send_as(&producer, MyEvent::Trigger).await?;
test.stop_recording().await;  // Settles and captures snapshot
```

For cascading events (actor A triggers actor B triggers actor C), call `settle()` between sends to let each level propagate:

```rust
test.settle().await;  // First cascade level
test.settle().await;  // Second cascade level
```

For actors that continuously produce events, use `settle_with_timeout()`:

```rust
if !test.settle_with_timeout(Duration::from_millis(100)).await {
    // Timeout elapsed, system may still have events in flight
}
```

## Spies

Spies provide focused views into recorded events.

### EventSpy

Inspect a specific event by ID:

```rust
let spy = test.event(event_id);

spy.was_delivered()              // true if delivered to any actor
spy.was_delivered_to(&consumer)  // true if delivered to specific actor
spy.not_delivered_to(&consumer)  // true if NOT delivered to specific actor
spy.was_delivered_to_all(&[&a, &b])  // true if delivered to all listed actors
spy.delivery_ratio(&[&a, &b, &c])   // fraction of listed actors that received it
spy.sender()                     // name of sending actor
spy.receivers()                  // list of receiving actors
spy.receivers_count()            // number of receivers
spy.children()                   // query for correlated child events
```

### ActorSpy

Inspect events from an actor's perspective:

```rust
let spy = test.actor(&consumer);

// Inbound (events received)
spy.inbound()              // EventQuery of received events
spy.events_received()      // number of events received
spy.last_received()        // most recent received event
spy.received_from()        // actors that sent events to this actor
spy.received_from_count()  // count of distinct senders

// Outbound (events sent)
spy.outbound()             // EventQuery of sent events
spy.events_sent()          // number of distinct events sent
spy.last_sent()            // most recent sent event
spy.sent_to()              // actors that received events from this actor
spy.sent_to_count()        // count of distinct receivers
```

### TopicSpy

Inspect events on a specific topic:

```rust
let spy = test.topic(MyTopic::Data);

spy.was_published()    // true if any events on this topic
spy.event_count()      // number of event deliveries
spy.receivers()        // actors that received events on this topic
spy.receivers_count()  // count of distinct receivers
spy.events()           // EventQuery for further filtering
```

## EventQuery

`EventQuery` provides a fluent API for filtering recorded events:

```rust
// Get all recorded events
let query = test.events();

// Chain filters
let orders = test.events()
    .sent_by(&trader)
    .received_by(&exchange)
    .matching_event(|e| matches!(e, MyEvent::Order(_)))
    .count();

// Available filters
query.sent_by(&actor)           // events sent by actor
query.received_by(&actor)       // events received by actor
query.with_topic(topic)         // events on specific topic
query.with_id(event_id)         // events with specific ID
query.correlated_with(id)       // events correlated to parent ID
query.with_label("MyVariant")   // events with specific label (requires Label trait)
query.matching_event(|e| ...)   // custom event predicate
query.matching(|entry| ...)     // custom entry predicate (access to metadata)

// Accessors
query.count()           // number of matching events
query.is_empty()        // true if no matches
query.first()           // first matching event
query.last()            // last matching event
query.nth(n)            // nth matching event
query.iter()            // iterator over matches
query.collect()         // unique events (deduplicated by event ID)
query.all_deliveries()  // all delivery records (including duplicates)
query.senders()         // unique sender actor IDs
query.receivers()       // unique receiver actor IDs
query.count_by_label()  // HashMap<String, usize> of event counts per label
```

Queries can be chained from spies:

```rust
// Events sent by normalizer that were received by trader
let events = test.actor(&normalizer)
    .outbound()
    .received_by(&trader)
    .count();
```

## Debugging

Dump all recorded events for debugging:

```rust
test.dump();
// Output:
// Recorded events (3 deliveries):
//   0: [Producer] --> [Consumer]  (id: 123...)
//   1: [Consumer] --> [Logger]    (id: 456...)
//   2: [Consumer] --> [Database]  (id: 456...)
```

Get event count:

```rust
let count = test.event_count();
```

## Example: Testing Event Cascades

```rust
#[tokio::test]
async fn test_order_processing_pipeline() -> Result<()> {
    let mut sup = Supervisor::<OrderEvent, OrderTopic>::default();

    let gateway = sup.add_actor("gateway", |ctx| Gateway::new(ctx), &[OrderTopic::Incoming])?;
    let validator = sup.add_actor("validator", |ctx| Validator::new(ctx), &[OrderTopic::Incoming])?;
    let processor = sup.add_actor("processor", |ctx| Processor::new(ctx), &[OrderTopic::Validated])?;
    let notifier = sup.add_actor("notifier", |ctx| Notifier::new(ctx), &[OrderTopic::Processed])?;

    let mut test = Harness::new(&mut sup).await;
    sup.start().await?;

    test.start_recording().await;

    // Inject order at gateway
    test.send_as(&gateway, OrderEvent::NewOrder(order)).await?;

    // Wait for full cascade: gateway -> validator -> processor -> notifier
    test.settle().await;
    test.settle().await;
    test.settle().await;

    test.stop_recording().await;

    // Verify pipeline
    assert_eq!(1, test.actor(&validator).events_received());
    assert_eq!(1, test.actor(&processor).events_received());
    assert_eq!(1, test.actor(&notifier).events_received());

    // Verify event flow
    assert!(test.topic(OrderTopic::Incoming).was_published());
    assert!(test.topic(OrderTopic::Validated).was_published());
    assert!(test.topic(OrderTopic::Processed).was_published());

    sup.stop().await
}
```

## Complete Example

See [`examples/arbitrage.rs`](../maiko/examples/arbitrage.rs) for a comprehensive demonstration of the test harness, including all spy types, queries, and assertion patterns.

## Limitations

- **Async timing**: `settle()` waits for actors to receive events, but not necessarily for them to finish processing. For long-running handlers, you may need additional synchronization.
- **Recording overhead**: When the test harness is enabled, there's minimal overhead even when not actively recording.
- **Single supervisor**: The harness is tied to a single supervisor instance.

## Performance Considerations

> **The test harness is designed for testing only. Do not use it in production.**

### Why Not Production?

The test harness uses an **unbounded channel** to collect events. This design choice prioritizes correctness and simplicity for testing:

- Events are never dropped, ensuring test assertions are reliable
- No backpressure that could affect actor timing during tests
- Simple implementation without complex flow control

However, in production this means:

- **Unbounded memory growth** — A fast producer with a slow consumer will accumulate events indefinitely
- **No backpressure** — The system won't slow down when overwhelmed
- **Memory exhaustion risk** — Long-running systems can run out of memory

### For Production Monitoring

If you need production observability, use the [monitoring API](monitoring.md) directly with a custom `Monitor` implementation that:

- Uses bounded channels or ring buffers
- Samples events under high load
- Batches writes to external systems
- Handles backpressure appropriately

### Settle Timing

The `settle()` method uses a timeout-based approach:

```rust
pub const DEFAULT_SETTLE_WINDOW: Duration = Duration::from_millis(1);
pub const DEFAULT_MAX_SETTLE: Duration = Duration::from_millis(10);
```

- **Settle window** (1ms): Returns when no events arrive for this duration
- **Max settle** (10ms): Maximum time to wait, even if events keep arriving

For chatty actors that continuously produce events, the max settle prevents infinite waiting. Adjust with `settle_with_timeout()`:

```rust
// Longer settle for slow systems
test.settle_with_timeout(Duration::from_millis(5), Duration::from_millis(50)).await;

// Shorter settle for chatty actors
test.settle_with_timeout(Duration::from_millis(1), Duration::from_millis(5)).await;
```
