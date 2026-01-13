default: check

# Run formatting, linting, tests (including doctests), and build examples
check:
    cargo fmt --all -- --check
    cargo clippy --workspace --all-targets

# Quickly format the workspace
fmt:
    cargo fmt --all

# Build entire workspace including examples
build:
    cargo build --workspace --examples

# Run all tests
test:
    cargo test --workspace --all-features

# Lint only
lint:
    cargo clippy --workspace --all-targets --all-features -D warnings

# Build docs without dependencies
doc:
    cargo doc --workspace --no-deps

# Run spell checker
spell:
    cargo spellcheck

# Run all examples
examples:
    cargo run --example hello-world
    cargo run --example pingpong
    cargo run --example guesser

