# maiko-macros

Procedural macros for the Maiko actor runtime.

- `#[derive(Event)]`: Implements `maiko::Event` for your type.

Usage:

```rust
use maiko_macros::Event;

#[derive(Clone, Debug, Event)]
enum MyEvent { Foo, Bar }
```

Publishing notes:
- This crate must be published before `maiko` can depend on it by version.
 - Edition: 2024.

Notes:
- The derive macro requires the `maiko` crate in your dependency tree because it implements the `maiko::Event` trait for your type.
- Generic and `where` clauses are preserved when deriving.
- Recommended import path in applications: `use maiko_macros::Event;`
