//! Maiko — Event-based actor runtime
//!
//! A tiny actor runtime inspired by event-driven systems like Kafka,
//! designed for ergonomic, loosely-coupled concurrency in Rust.
//!
//! Work in progress. Stay tuned!

mod actor;
mod context;
mod event;
mod topic;

pub use actor::Actor;
pub use context::Context;
pub use event::Event;
pub use topic::Topic;

pub fn not_ready_yet() {}
