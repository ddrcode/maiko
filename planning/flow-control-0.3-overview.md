# Maiko 0.3.0 - Flow Control Overview

## Problem

Maiko currently drops events silently when an actor's mailbox is full. No overflow handling, no backpressure propagation. Producers have no feedback when consumers can't keep up.

## What's shipping in 0.3

### OverflowPolicy on Topic

```rust
pub enum OverflowPolicy {
    Drop,   // Discard event, warn, continue (default - preserves current behavior)
    Fail,   // Close consumer channel - actor dies with OverflowClosed error
    Block,  // Wait for space - cascading backpressure
}
```

Policy lives on the Topic trait, not on actors. Flow semantics are properties of data - HID reports are critical regardless of who processes them, telemetry is expendable regardless of who produces it.

```rust
impl Topic<MyEvent> for MyTopic {
    fn from_event(event: &MyEvent) -> Self { ... }

    fn overflow_policy(&self) -> OverflowPolicy {
        match self {
            MyTopic::Critical => OverflowPolicy::Block,
            MyTopic::Stats    => OverflowPolicy::Drop,
        }
    }
}
```

Existing code with DefaultTopic sees zero change - Drop is the default.

### Two-Phase Broker Dispatch

Broker handles each event in two phases:
1. **Instant** - Drop and Fail subscribers via `try_send()` (non-blocking)
2. **Concurrent wait** - Block subscribers via `join_all(send futures)` (concurrent, not sequential)

Drop/Fail subscribers always receive promptly, regardless of Block subscriber state.

### Simplified Context API

Three send methods (`send`, `send_with_correlation`, `send_child_event`) collapse into one:

```rust
// Simple
ctx.send(MyEvent::Pong).await?;

// With correlation - user builds Envelope, Context stamps sender identity
ctx.send(Envelope::correlated(MyEvent::Response, parent_id)).await?;
```

`send().await` is the only sending path. No `try_send`. Tokio's cooperative scheduling provides natural fairness.

### send_latency() for self-regulating actors

Actors that need to adapt to system load use `send_latency()` - the actual Duration of the last send call:

```rust
async fn step(&mut self) -> Result<StepAction> {
    let msg = self.source.next().await;
    self.ctx.send(msg.into()).await?;
    Ok(StepAction::Defer(self.ctx.send_latency()))
}
```

System healthy? Send returns in microseconds, actor continues immediately. System busy? Send blocks longer, actor naturally slows down. No magic thresholds, no synthetic pressure metrics - just real measurement feeding into existing scheduling.

### Broker Input Channel Sizing

Stage 1 channel (actors to broker) defaults to sum of all actor channel capacities, computed at start time. Configurable override on supervisor. Scales naturally with system size - if all actor channels are full, Stage 1 can absorb the overflow. If both stages fill, producers block (correct backpressure).

### EventRouter (internal)

Context routes through an internal EventRouter trait, isolating actors from broker topology. In 0.3 it wraps a single channel. In 0.4 it maps to per-topic-group brokers. Upgrade is invisible to users.

## What's NOT in 0.3

| Feature | Why | When |
|---------|-----|------|
| Rate limiting (`max_rate`) | Orthogonal to overflow policy | 0.4 |
| Per-topic brokers | Requires multi-broker EventRouter | 0.4 |
| Pressure / capacity API | `send_latency()` covers immediate need | 0.4 |
| Supervisor overrides | Topic defaults sufficient for now | 0.4 |
| Priority levels | May be redundant with policy | TBD |
| DropOldest | Requires custom channel | TBD |

## Known limitation (single broker)

When the broker waits on Block subscribers, dispatch for other topics is delayed. All producers share one input channel. This is a deployment limitation of single-broker, not a design flaw - it disappears with per-topic-group brokers in 0.4.

## Acknowledgments

Design shaped by input from @serle (cascading backpressure concerns, producer guarantees), @WojtekRewaj (watermarks, circuit breakers for slow actors), and @awilde (producer-side awareness, quota concepts). Full discussion history and rejected alternatives documented in `planning/flow-control-decisions.md`.
