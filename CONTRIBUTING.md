# Contributing to Maiko

Contributions are welcome — bug reports, feature ideas, documentation improvements, or code. This guide covers a few things that will help your contribution land smoothly.

## Building and Testing

```bash
just check          # Format check + clippy
just test           # All tests including doctests
```

Or directly with cargo:

```bash
cargo test --workspace --all-features
cargo clippy --workspace --all-targets --all-features -- -D warnings
```

## Branching

**0.2.x** is the current stable series. The `main` branch tracks the latest 0.2.x release. Version 0.2.6 is intended to be the final release in this series — only bugfixes are accepted for 0.2.

**0.3.0** is the next major development target, with API changes still being designed. Work targeting 0.3 should be based on the `v0.3.0` branch.

| What you're contributing | Target branch |
|--------------------------|---------------|
| Bugfix for current release | `main` |
| Documentation improvement | `main` |
| New feature or API change | `v0.3.0` |

If you're unsure which branch to target, just ask in the issue or PR — happy to help.

## Issues and Labels

Not all open issues are ready for implementation. Some are rough ideas or proposals still being discussed.

- **`ready`** — problem is understood, scope is clear, ready to be picked up
- **`good first issue`** — a good starting point for new contributors
- **`feature proposal`** — an idea open for discussion. Feedback and opinions are welcome, but please don't start implementing until the approach is agreed on

When in doubt, leave a comment on the issue before writing code. A quick conversation upfront prevents wasted effort.

## Pull Requests

- Keep PRs focused — one concern per PR
- Include tests for new functionality
- Run `just check` before submitting
- Describe what the PR does and why, not just how

## Questions?

Open an issue or start a discussion. If something is unclear, that's worth fixing.
