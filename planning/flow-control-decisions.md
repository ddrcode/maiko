# Flow Control Design Decisions - Maiko 0.3.0

*Distilled from multi-session design discussion. Captures what was decided, what was rejected, and why.*

## Problem

Maiko's message passing has an asymmetry:
- **Actor to Broker**: blocking `send().await` (bounded channel, 128 slots)
- **Broker to Actor**: non-blocking `try_send()` - events silently dropped on full channel

This surfaced in Charon: Typist produces HID reports faster than KeyWriter can push them to `/dev/hidg0` (USB gadget `write_all()` blocks ~1ms at 1000Hz polling). Typist self-throttles with a 20ms sleep - safe but slow. Removing the sleep floods KeyWriter's mailbox. The broker logs a warning and drops events. No recourse.

Two gaps exposed:
1. No overflow handling (broker drops silently)
2. No backpressure propagation (producers can't know consumers are overloaded)

---

## Decision 1: OverflowPolicy on Topic, not Actor

**Chosen**: Policy is a method on the `Topic` trait with a default of `Drop`.

**Alternatives considered**:
- Per-actor (on `Subscribe` / `add_actor`): Natural configuration point, but an actor receiving both critical (HID) and expendable (telemetry) topics would need one policy. Doesn't fit.
- Per-subscription (actor + topic pair): Maximum granularity, but verbose and premature. No use case demands it yet.

**Reasoning**: Flow semantics are properties of **data**, not consumers. HID reports are critical regardless of who processes them. Telemetry is expendable regardless of who produces it. Topic already defines "what is this data" via `from_event()` - adding "how important is this data" is a natural extension.

```rust
pub enum OverflowPolicy {
    Drop,   // Discard event on full channel, warn, continue
    Fail,   // Close consumer channel - actor dies
    Block,  // Wait for space (cascading backpressure)
}
```

`Drop` as default preserves current behavior exactly. Existing users see zero change.

Per-subscription override remains a clean upgrade path for 0.4 without breaking Topic-level defaults.

---

## Decision 2: Two-Phase Broker Dispatch

**Chosen**: Broker dispatches each event in two phases - instant (Drop/Fail) then concurrent wait (Block).

**Discussion**: The core tension is that Block subscribers must not delay Drop/Fail subscribers. If the broker dispatches sequentially and one Block subscriber is slow, every other subscriber waits.

Phase 1 handles Drop and Fail via `try_send()` - instant, non-blocking. Phase 2 collects all Block subscribers into futures and `join_all().await` - concurrent, not sequential. Drop/Fail subscribers always receive promptly regardless of Block subscriber state.

**Single-broker limitation** (0.3): During Phase 2, the broker is blocked. Other events queued in the broker's input channel wait. In multi-broker (0.4), each broker only blocks its own topic group - this limitation disappears.

**Fail semantics**: `Fail` closes the consumer's channel. The actor sees a distinct `OverflowClosed` error - distinguishable from "system shutting down." The actor dies; the system decides what to do next (recovery, restart, cascade). Fail means "data loss is unacceptable for this consumer, halt it rather than continue with gaps."

---

## Decision 3: Design for Multi-Broker, Ship Single

**Chosen**: All abstractions designed as if multiple brokers exist. Single broker is the 0.3 special case.

**Discussion**: This was a pivotal reframing. The initial approach designed around single-broker constraints (pressure gauges, Context-side throttling). Stepping back and imagining the multi-broker target revealed that many "problems" (Block delays all topics, global input channel contention) are artifacts of single-broker deployment, not design flaws.

Multi-broker model:
```
                    +-- Broker(Hid) ------> KeyWriter
Actor -> Context ---+
                    +-- Broker(Stats) ----> Telemetry
                    +-- Broker(Control) --> ModeManager
```

Block on Hid broker doesn't affect Stats broker. Backpressure is natural channel backpressure. No pressure gauge needed at the framework level.

The upgrade from single to multi-broker should be a supervisor wiring change, not an API change. This drives the EventRouter abstraction (Decision 4).

---

## Decision 4: EventRouter Abstraction

**Chosen**: Context routes events through an internal `EventRouter` trait, not directly through a channel sender.

```rust
// Internal - not part of public API
pub(crate) trait EventRouter<E: Event>: Send + Sync {
    fn send(&self, envelope: Arc<Envelope<E>>) -> Pin<Box<dyn Future<Output = Result<()>> + Send + '_>>;
    fn try_send(&self, envelope: Arc<Envelope<E>>) -> Result<()>;
}
```

**0.3**: Router wraps a single channel sender (current behavior).
**0.4**: Router maps events to per-topic-group broker channels via `T::from_event()`.

Context stays `Context<E>` - no Topic type parameter. Actor trait unchanged. The upgrade from single to multi-broker is invisible to users.

**Why not expose the trait**: It's an internal mechanism for topology abstraction. Users don't need to implement custom routers. If that changes, we can promote it to public API later.

---

## Decision 5: `send` only + `send_latency` (replacing 3 methods with 1)

**Chosen**: One sending method on Context, accepting `impl Into<Envelope<E>>`. One observability method.

```rust
ctx.send(MyEvent::Pong).await?;           // simple
ctx.send(Envelope::correlated(MyEvent::Pong, parent_id)).await?;  // correlated

// Self-regulating actor
ctx.send(event).await?;
Ok(StepAction::Defer(ctx.send_latency()))
```

**What it replaces**: Current API has `send()`, `send_with_correlation()`, `send_child_event()` - three methods that differ only in envelope construction. Correlation is the user's concern (they know the parent event). Sender identity is the framework's concern (Context stamps actor_id).

`From<E> for Envelope<E>` enables the ergonomics - pass a raw event and it auto-wraps.

**Why no try_send**: Extended debate explored `try_send`, `offer`, capacity guards, `pressure()`, and `has_capacity()`. Each was rejected for specific reasons:

- `try_send` bypasses Tokio cooperative scheduling, enabling aggressive producers that starve other actors. The per-call overhead vs `send().await` on empty channel is ~10-30ns - negligible.
- `offer` (Java BlockingQueue naming) was foreign to Rust idiom.
- Capacity guard (75% threshold on try_send) was "building on a lie" - telling producers the system is full when it's not.
- `has_capacity() -> bool` is a different lie - a step function where reality is a gradient.
- `pressure() -> f32` is useful but is observability infrastructure, not flow control.

Resolution: `send().await` is the only sending path. Actors that need to adapt use `send_latency()` - the actual `Duration` of the last `send()` call. This feeds directly into `StepAction::Defer`, enabling self-regulating actors through existing machinery. No new concepts, no magic numbers, no lies.

**send_latency()**: Two `Instant` reads per send, stores one `Duration`. Topology-independent - works identically in single-broker (0.3) and multi-broker (0.4). Measures the actor's real experience, not internal channel state. Like TCP congestion control using RTT as the signal.

---

## Decision 7: Stage 1 Channel Capacity

**Chosen**: Sum of all actor channel capacities (Stage 2), computed at `supervisor.start()` time. Configurable override on supervisor builder.

**Current state**: Hardcoded 128 for the broker input channel.

**Reasoning**: Stage 1 (actors to broker) should be able to absorb everything Stage 2 (broker to actors) can hold. If all actor channels are full and the broker can't dispatch, Stage 1 buffers the overflow. If Stage 1 also fills, producers block - correct backpressure.

**Alternatives considered**:
- Fixed constant (128): doesn't scale. 10 actors and 1000 actors get the same buffer.
- Max of actor capacities: bounded but ignores producer count.
- `N * factor`: arbitrary factor.

Sum scales naturally: 10 actors at 128 each = 1,280. 100 actors at 128 = 12,800. Large but bounded by system definition. Override available for users who know their workload.

---

## Decision 8: Envelope Refactoring

**Chosen**: `Envelope::new(event)` creates without sender identity. Context fills in actor_id before dispatch.

**Current state**: `Envelope::new(event, actor_id)` requires the sender at construction time. This works but means Context must thread actor_id into every send variant.

**New approach**: Envelope can be created "incomplete" (no sender). Context stamps its actor_id before routing. This enables `From<E> for Envelope<E>` and the `impl Into<Envelope>` ergonomics.

`Envelope::correlated(event, parent_id)` creates with correlation but still without sender. Context fills both.

---

## What Was Deferred and Why

| Feature | Reasoning |
|---------|-----------|
| **Pressure / capacity API** | `pressure() -> f32`, `has_capacity() -> bool`, `capacity() -> usize` all evaluated and deferred. Pressure is a snapshot that becomes ambiguous in multi-broker. Boolean capacity is a lie (gradient, not step function). Raw usize leaks channel size. `send_latency()` covers the immediate need. Richer observability comes with 0.4. |
| **Rate limiting (`max_rate`)** | Orthogonal to overflow policy. Rate limiting is about "how fast should data flow" - a separate concern from "what happens when it's too fast." Clean addition for 0.3.1 or 0.4 without affecting overflow design. |
| **Per-topic brokers** | Requires the multi-broker EventRouter implementation. The abstraction is ready (Decision 4), but the wiring isn't trivial and single-broker is sufficient for Maiko's current scale. |
| **Supervisor overrides** | Topic defaults should be sufficient for 0.3. Runtime overrides add API surface. Wait until a user actually needs to override a Topic's compiled-in policy. |
| **Priority levels** | Potentially redundant with OverflowPolicy. Topics with Block are implicitly higher priority than Drop. Explicit priority raises questions like "what does Block + Low priority mean?" Needs more design thought. |
| **DropOldest** | Desirable (keep latest, discard stale) but requires a custom channel implementation. Tokio mpsc doesn't support front-pop from sender side. Separate effort. |
| **Dynamic rate / quota** | System-determined rate limits based on load patterns. Powerful but needs the pressure gauge infrastructure first. Multi-broker prerequisite. |

---

## Discussion Highlights

### The `try_send` Debate (resolved: dropped)

Extended discussion about whether Context should expose non-blocking send. The journey:

1. **try_send proposed** - for boundary actors (WebSocket bridges) that can't block
2. **Capacity guard added** - 75% threshold to prevent aggressive producers starving send() callers
3. **"Building on a lie"** - the guard tells producers "full" when channel is 25% free
4. **offer() explored** - intent-based naming (Java BlockingQueue), rejected as un-Rusty
5. **has_capacity() explored** - separates information from action, but boolean is still a lie
6. **pressure() explored** - gradient not step function, but scope creep into observability
7. **send_latency() discovered** - actual measurement, feeds into StepAction::Defer, no lies

Final resolution: `send().await` is the only sending path. `try_send` conflates two concerns (action + information). Actors that need to self-regulate use `send_latency()` with existing `StepAction::Defer` machinery. TCP congestion control analogy: use actual round-trip time as the signal, not a synthetic metric.

Key insight: "try_send is shared responsibility used irresponsibly - 100% system liability. send().await is shared responsibility by design."

### Block Scope

Debated whether Block should be approximate (single broker dispatches sequentially) or exact (dedicated broker per blocking topic). Resolved by Decision 3 - design for exact (multi-broker), accept approximate as 0.3 limitation. The limitation is deployment, not design.

### External Feedback

Three external reviewers provided input (captured in overflow.md):

- **@serle**: Highlighted that Block cascades backward to producers, potentially breaking the decoupled abstraction. Suggested producer guarantees and subscription-time compatibility checks. Addressed by the two-phase dispatch - Block's cascade is intentional and documented.

- **@WojtekRewaj**: Suggested watermarks/thresholds and circuit breakers for slow actors. Watermarks deferred (pressure gauge territory). Circuit breaker is an interesting future addition.

- **@awilde**: Argued against consumer-side responsibility (you send something and don't know what happened). Suggested producer-side quotas. This perspective influenced `send_latency()` - giving producers honest feedback about system responsiveness without exposing internal channel state.

---

## Single-Broker Limitations (documented, accepted)

With one broker handling all topics:

1. **Block delays all topics**: When broker waits on Block subscribers, Drop/Fail dispatch for other topics is delayed (bounded by slowest Block consumer response time).

2. **Global input channel**: All producers share one channel to the broker. When broker is busy with Block dispatch, input fills and all producers block regardless of their topic.

For Maiko's target scale (tens to hundreds of actors, moderate event rates), the delay is typically under 10ms and acceptable. These limitations disappear with multi-broker in 0.4.
