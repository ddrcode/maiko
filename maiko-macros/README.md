# maiko-macros

Procedural macros for the [Maiko](https://crates.io/crates/maiko) actor runtime.

This crate is a companion to `maiko` and is re-exported from it — you typically don't need to depend on it directly.

## Macros

### `#[derive(Event)]`

Implements `maiko::Event` for your type. This is a marker trait that bundles `Clone + Send + Sync + 'static`.

```rust
use maiko_macros::Event;

#[derive(Clone, Debug, Event)]
enum MyEvent { Foo, Bar }
```

### `#[derive(Label)]`

Implements `maiko::Label` for enums, returning variant names as strings. Useful for logging, monitoring, and diagram generation.

```rust
use maiko_macros::Label;

#[derive(Label)]
enum MyTopic { SensorData, Alerts }

// MyTopic::SensorData.label() returns "SensorData"
```

Only works on enums.

### `#[derive(SelfRouting)]`

Implements `maiko::Topic<Self> for Self`, enabling the event-as-topic pattern where each enum variant acts as its own routing topic.

```rust
use maiko_macros::{Event, SelfRouting};

#[derive(Clone, Debug, Hash, PartialEq, Eq, Event, SelfRouting)]
enum PingPong { Ping, Pong }
```

Requires `Clone`, `Hash`, `PartialEq`, and `Eq`.

## Notes

- All derive macros preserve generics and `where` clauses.
- The macros generate impls referencing `maiko::` paths, so the `maiko` crate must be in your dependency tree.

## Minimum Supported Rust Version

1.85 (edition 2024)

---

## License

Licensed under either of [Apache License, Version 2.0](LICENSE-APACHE) or [MIT license](LICENSE-MIT)
at your option.

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in this
crate by you, as defined in the Apache-2.0 license, shall be dual licensed as above,
without any additional terms or conditions.
