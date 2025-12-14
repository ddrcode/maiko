# [0.1.0](https://github.com/ddrcode/maiko/compare/v0.0.2...v0.1.0) (December 14th, 2025

**MVP**. Fully-functional, teset, yet quite minimal version.

## Added

- `Event` derive macro [#3](https://github.com/ddrcode/maiko/issues/3)
- Documentation and examples [#4](https://github.com/ddrcode/maiko/issues/4)
- Event correlation logic [#7](https://github.com/ddrcode/maiko/issues/7)

## Deleted

- dependency on `async-trait` ([#5](https://github.com/ddrcode/maiko/issues/5))

## Changed

- renamed `DefaultTopic` to `Broadcast`
- changed channel data type from `Envelope<E>` to `Arc<Envelope<E>>`
- made broker working with non-blocking send

# 0.0.2 (December 9th, 2025)

First working version
