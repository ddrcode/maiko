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
