# Backpressure Implementation Plan

## Context

Maiko has a critical bug in the broker's event dispatch (`broker.rs:47-81`). When any actor's mailbox is full, `try_send()` fails, `try_for_each` stops iterating (remaining subscribers miss the event), and the error propagates up — crashing the entire broker and system. One slow consumer kills everything.

This plan fixes it with two overflow strategies and prepares the API for a future `LatestOnly` strategy.

## Overflow strategies

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Overflow {
    #[default]
    DropNewest,   // drop the incoming event for this subscriber, continue to others
    Fail,         // propagate error, stop broker (current behavior, for strict/testing)
}
```

**Future: `LatestOnly`** — watch-like semantics (capacity-1 mailbox, always holds the most recent event). Useful for price feeds, sensor data, etc. Can be built on mpsc + shared slot. Deferred from this PR.

**No `Block` strategy** — would stall the single-threaded broker, blocking delivery to ALL actors while waiting for one slow consumer.

## API: overflow config on `Subscribe`

Add `overflow` and `channel_size` as optional fields on `Subscribe`. No new Supervisor method — `add_actor` signature stays identical.

```rust
// Existing API — unchanged:
sup.add_actor("a", factory, Subscribe::all())?;
sup.add_actor("a", factory, &[Topic::A])?;

// New — per-actor overflow:
sup.add_actor("a", factory,
    Subscribe::all().with_overflow(Overflow::Fail))?;

// New — per-actor channel size:
sup.add_actor("a", factory,
    Subscribe::to(&[Topic::A]).with_channel_size(256))?;
```

When `with_overflow` / `with_channel_size` is not called, the system-wide default from `Config` is used. The `&[T]` shorthand works unchanged — its `From` impl sets both to `None`.

### `Subscribe` struct change

```rust
// Before (tuple struct):
pub struct Subscribe<E: Event, T: Topic<E>>(
    pub(crate) Subscription<T>,
    PhantomData<fn() -> E>,
);

// After (named fields):
pub struct Subscribe<E: Event, T: Topic<E>> {
    pub(crate) subscription: Subscription<T>,
    pub(crate) overflow: Option<Overflow>,
    pub(crate) channel_size: Option<usize>,
    _event: PhantomData<fn() -> E>,
}
```

### System-wide default via `Config`

```rust
pub overflow: Overflow,  // default: Overflow::DropNewest
```

With builder: `Config::default().with_overflow(Overflow::Fail)`

## Core fix: broker dispatch

Replace `try_for_each` + `?` in `broker.rs:send_event` with a `for` loop. Per-subscriber overflow handling:

```rust
for subscriber in self.subscribers.iter()
    .filter(|s| s.topics.contains(&topic))
    .filter(|s| !s.sender.is_closed())
    .filter(|s| s.actor_id != *e.meta().actor_id())
{
    match subscriber.sender.try_send(e.clone()) {
        Ok(()) => {
            // monitoring: EventDispatched
        }
        Err(TrySendError::Full(_)) => match subscriber.overflow {
            Overflow::DropNewest => {
                // monitoring: EventDropped, then continue to next subscriber
            }
            Overflow::Fail => return Err(Error::ChannelIsFull),
        },
        Err(TrySendError::Closed(_)) => {
            // actor exited — skip silently, cleanup interval removes it
        }
    }
}
```

Key behavioral changes:
- **DropNewest**: event is dropped for this subscriber only; other subscribers still receive it
- **Closed channels**: no longer propagate errors (currently, `TrySendError::Closed` converts to `SendError` and crashes). Silently skip — the maintenance cleanup removes dead subscribers.

## Subscriber changes

Add `overflow: Overflow` field to `Subscriber` (`internal/subscriber.rs`):

```rust
pub(crate) struct Subscriber<E: Event, T: Topic<E>> {
    pub actor_id: ActorId,
    pub topics: Subscription<T>,
    pub sender: Sender<Arc<Envelope<E>>>,
    pub overflow: Overflow,  // NEW
}
```

## Supervisor wiring

In `supervisor.rs`, `add_actor` extracts overflow/channel_size from `Subscribe`, falls back to `Config` defaults, and passes them through `register_actor`:

```rust
pub fn add_actor<A, F, S>(&mut self, name: &str, factory: F, topics: S) -> Result<ActorId>
where ...
{
    let subscribe = topics.into();
    let overflow = subscribe.overflow.unwrap_or(self.config.overflow);
    let channel_size = subscribe.channel_size.unwrap_or(self.config.channel_size);
    // ... create channel with channel_size, create subscriber with overflow
}
```

`register_actor` signature gains `overflow: Overflow` and `channel_size: usize` parameters.

## Monitoring integration

- New `MonitoringEvent::EventDropped(envelope, topic, actor_id)` variant
- New `Monitor::on_event_dropped(&self, envelope, topic, receiver)` callback (default no-op)
- `dispatcher.rs`: handle `EventDropped` → call `on_event_dropped` on monitors
- Tracer monitor: `tracing::warn!` on dropped events

## Files to modify

| # | File | Change |
|---|------|--------|
| 1 | **New: `src/overflow.rs`** | `Overflow` enum |
| 2 | `src/subscribe.rs` | Tuple struct → named fields, add `overflow`/`channel_size`, builder methods |
| 3 | `src/config.rs` | Add `overflow: Overflow` field + `with_overflow()` builder |
| 4 | `src/internal/subscriber.rs` | Add `overflow: Overflow` field, update constructor |
| 5 | `src/internal/broker.rs` | Replace `try_for_each`+`?` with overflow-aware `for` loop |
| 6 | `src/supervisor.rs` | Extract overflow/channel_size from Subscribe, thread through `register_actor` |
| 7 | `src/lib.rs` | Export `Overflow` |
| 8 | `src/monitoring/monitoring_event.rs` | Add `EventDropped` variant |
| 9 | `src/monitoring/monitor.rs` | Add `on_event_dropped()` callback |
| 10 | `src/monitoring/dispatcher.rs` | Handle `EventDropped` |
| 11 | `src/monitors/tracer.rs` | Implement `on_event_dropped` |

## Implementation order

1. `overflow.rs` — new enum
2. `subscribe.rs` — struct change + builder methods + update `From` impls
3. `config.rs` — add `overflow` field
4. `internal/subscriber.rs` — add `overflow` field, update constructor
5. `internal/broker.rs` — core dispatch fix
6. `supervisor.rs` — wire overflow/channel_size through
7. `lib.rs` — export Overflow
8. Monitoring files — EventDropped support
9. Tests

## Verification

1. `cargo test` — all existing tests pass (broker test needs `Overflow` param added to `Subscriber::new`)
2. `cargo test --features monitoring` — monitoring tests pass
3. New tests:
   - Broker: DropNewest skips full subscriber but others receive; Fail propagates error; mixed policies
   - Subscribe: `with_overflow()` and `with_channel_size()` builder methods
   - Config: `with_overflow()` builder
4. Run existing examples (`cargo run --example pingpong`, `cargo run --example guesser`) — behavior unchanged
