# [0.2.2](https://github.com/ddrcode/maiko/compare/v0.2.1...v0.2.2) (February 4th, 2026)

### Added

- `Recorder` monitor for recording events to JSON Lines files ([#49])
- `Tracer` monitor for logging event lifecycle via `tracing` crate ([#52])
- `monitors` module for built-in monitor implementations ([#52])

### Changed

- `Error` enum no longer implements `Clone` (allows deriving from non-Clone errors) ([#49])
- Reorganized monitoring: `monitoring` module for engine, `monitors` for implementations ([#52])
- Improved documentation

[#49]: https://github.com/ddrcode/maiko/pull/49
[#52]: https://github.com/ddrcode/maiko/pull/52

---

# [0.2.1](https://github.com/ddrcode/maiko/compare/v0.2.0...v0.2.1) (January 29th, 2026)

### Added

- [Frequently Asked Questions](https://github.com/ddrcode/maiko/blob/main/docs/faq.md)

### Changed

- `ActorId` has a public constructor now
- Improved documentation and README
- Moved `doc` to `docs`

---

# [0.2.0](https://github.com/ddrcode/maiko/compare/v0.1.1...v0.2.0) (January 27th, 2026)

** Contains Breaking changes **

### Key new Features

1. Adopted Maiko to work with project [Charon](https://github.com/ddrcode/charon)
as a first library use case. [#23]
2. Number of ergonomy improvements (API changes!) [#33], [#37]
3. Monitoring API [#36]
4. Test harness [#31]

### Added

- `StepAction` enum to control `step` function behavior
- `serde` feature that makes events serializable/deserializable
- `External` option in `Error` enum
- `clone_name` method in `Context`

### Changed

- Renamed `tick` to `step` in `Actor`
- Renamed `handle` to `handle_event` in `Actor`
- Fields of `Envelope` made private (use methods instead)

[#23]: https://github.com/ddrcode/maiko/pull/23
[#31]: https://github.com/ddrcode/maiko/pull/31
[#33]: https://github.com/ddrcode/maiko/pull/33
[#36]: https://github.com/ddrcode/maiko/pull/36
[#37]: https://github.com/ddrcode/maiko/pull/37


---

# [0.1.1](https://github.com/ddrcode/maiko/compare/v0.1.0...v0.1.1) (December 18th, 2025)

### Added

- `hello-wrold.rs` example
- `maintenance_interval` option added to `Config`
- `Broker` removes closed subscribers in periodical `cleanup` method
- `pending` method added to `Context`
- graceful shutdown - completes pending events before stop

### Changed

- Main `ActorHandle` loop to work with `tokio::select!`
- Detailed documentation aded to examples.
- `PingPong` example with events as topics.
- Improved performance of subscribers lookup in `Broker`
- `Broadcast` topic renamed to `DefaultTopic`
- `Broker` has now dedicated cancellation token (rather than shared one with actors)


---

# [0.1.0](https://github.com/ddrcode/maiko/compare/v0.0.2...v0.1.0) (December 14th, 2025)

**MVP**. Fully-functional, tested, yet quite minimal version.

### Added

- `Event` derive macro ([#3], [#9])
- Documentation and examples ([#4])
- Event correlation logic ([#7])

### Removed

- dependency on `async-trait` ([#5])

### Changed

- renamed `DefaultTopic` to `Broadcast` ([#11])
- changed channel data type from `Envelope<E>` to `Arc<Envelope<E>>` ([#11])
- made broker working with non-blocking send

[#3]: https://github.com/ddrcode/maiko/issues/3
[#4]: https://github.com/ddrcode/maiko/issues/4
[#5]: https://github.com/ddrcode/maiko/issues/5
[#7]: https://github.com/ddrcode/maiko/issues/7
[#9]: https://github.com/ddrcode/maiko/pull/9
[#11]: https://github.com/ddrcode/maiko/pull/11

---

# 0.0.2 (December 9th, 2025)

First working version
