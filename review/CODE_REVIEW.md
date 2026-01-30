# Maiko Code Review

## 1. Overview
Maiko is a compact actor runtime for Tokio. It relies on a "Broker" model where a central event loop dispatches messages to actors via `mpsc` channels.

**Strengths:**
*   **Simplicity**: The codebase is small and easy to understand.
*   **Type Safety**: Leveraging Rust's type system (`Event` trait, generics) ensures compile-time safety for messages.
*   **Tokio Integration**: Native compatibility with the Tokio ecosystem.

## 2. Critical Findings

### [Stability] Slow Consumers Can Crash the Broker
**Severity: Critical**
**Location**: `maiko/src/internal/broker.rs:61-81`

The broker uses `try_send` to dispatch events to subscribers. If *any* subscriber's channel is full (slow consumer), `try_send` returns an error. This error is propagated up through `try_for_each` -> `send_event` -> `run`.

```rust
// maiko/src/internal/broker.rs

self.subscribers.iter()
    // ...
    .try_for_each(|subscriber| {
        // If channel is full, this returns Err
        let res = subscriber.sender.try_send(e.clone());
        // ...
        res
    })?; // '?' propagates the error
```

If this happens, `Broker::run` terminates, and the entire messaging system stops working. A singe slow actor will bring down the entire application.

**Recommendation**: The broker should handle `TrySendError::Full` gracefully (e.g., log a warning and drop the message, or use a bounded blocking send with timeout), rather than crashing.

### [Flexibility] No Dynamic Actor Registration
**Severity: Medium**
**Location**: `maiko/src/supervisor.rs:145`

The supervisor prevents adding actors after the system has started.

```rust
// maiko/src/supervisor.rs

let mut broker = self.broker
    .try_lock() // Fails if broker loop is running
    .map_err(|_| Error::BrokerAlreadyStarted)?;
```

This confirms the "start-once" design. While documented, it severely limits use cases (e.g., handling per-connection actors for a WebSocket server).

## 3. Performance & Scalability

### [Performance] O(N) Routing
**Location**: `maiko/src/internal/broker.rs:61`

```rust
self.subscribers
    .iter()
    .filter(|s| s.topics.contains(&topic))
```

The broker iterates over *all* registered subscribers for *every* event.
*   **Impact**: For small systems (tens of actors), this is negligible.
*   **Scalability**: As actor count grows, routing latency increases linearly. A topic-to-subscriber map (lookup table) would be O(1) or O(K).

### [Memory] Event Cloning
Events are `Arc<Envelope<E>>` when passed through channels, which is efficient (zero-copy routing). However, the initial creation of the `Envelope` requires moving/cloning the event. The design encourages `Clone`-heavy events.

## 4. API Design

### [Ergonomics] Safe & Clean
The `Actor` trait and `Context` API are well-designed.
*   `step()` returning `StepAction` is a great pattern for controlling actor scheduling without complex `select!` loops in user code.
*   `ActorId` being serializable (implied by design) supports future distributed use cases.

## 5. Summary
Maiko is a solid foundation for small-to-medium static actor topologies. However, the **Slow Consumer** issue is a critical vulnerability that must be addressed before using it in production systems where load spikes could occur.
