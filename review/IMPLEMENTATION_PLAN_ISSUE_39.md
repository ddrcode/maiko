# Implement Event Recorder (Issue #39)

This plan outlines the implementation of a file-based event recorder for Maiko, enabling events to be serialized to JSON and saved to a file for later inspection.

## User Review Required

> [!NOTE]
> This change introduces a new feature flag `recorder` which pulls in `serde_json`.

## Proposed Changes

### Maiko Package

#### [MODIFY] [Cargo.toml](file:///c:/Users/Wojtek/.gemini/antigravity/scratch/maiko/maiko/Cargo.toml)
- Add `serde_json` to dependencies (optional).
- Add `recorder` feature including `monitoring`, `serde`, and `serde_json`.
- Add `fs` feature to `tokio` (if needed for async file I/O, though standard `std::fs` might be used inside `RefCell`/`Mutex` if we don't want to await in `Monitor` callbacks which are sync... Wait, `Monitor` methods are `&self` and synchronous? Let's check `Monitor` trait again).
    - `Monitor` methods are NOT async (e.g., `fn on_event_handled(...)`).
    - So we should use `std::fs::File` and `std::io::BufWriter` protected by `std::sync::Mutex` (since `Monitor: Send`).

#### [MODIFY] [src/monitoring/mod.rs](file:///c:/Users/Wojtek/.gemini/antigravity/scratch/maiko/maiko/src/monitoring/mod.rs)
- Add `pub mod recorder;` guarded by `#[cfg(feature = "recorder")]`.

#### [NEW] [src/monitoring/recorder.rs](file:///c:/Users/Wojtek/.gemini/antigravity/scratch/maiko/maiko/src/monitoring/recorder.rs)
- Define `Recorder` struct.
- Implement `Monitor` trait.
    - `on_event_dispatched`: Serialize event to JSON and write to file.
    - Use `serde_json::to_writer` or write newline-delimited JSON.

### Examples

#### [NEW] [examples/recorder.rs](file:///c:/Users/Wojtek/.gemini/antigravity/scratch/maiko/maiko/examples/recorder.rs)
- Demonstrate usage of `Recorder`.

## Verification Plan

### Automated Tests
- **Unit Test**: Add a test in `src/monitoring/recorder.rs` that:
    1. Creates a `Recorder` with a temporary file.
    2. Simulates an event dispatch.
    3. Verifies the file content matches expected JSON.
- **Example Run**:
    - Run the new example: `cargo run -p maiko --example recorder --features recorder`
    - Check the output file.

### Manual Verification
- None required beyond running the example.
