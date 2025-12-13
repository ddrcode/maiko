//! Maiko — Event-based actor runtime
//!
//! A tiny actor runtime inspired by event-driven systems like Kafka,
//! designed for ergonomic, loosely-coupled concurrency in Rust.
//!
//! See `examples/guesser.rs` and `examples/pingpong.rs`.

mod actor;
mod config;
mod context;
mod envelope;
mod error;
mod event;
mod meta;
mod supervisor;
mod topic;

mod internal;

pub use actor::Actor;
pub use config::Config;
pub use context::Context;
pub use envelope::Envelope;
pub use error::Error;
pub use event::Event;
pub use meta::Meta;
pub use supervisor::Supervisor;
pub use topic::{DefaultTopic, Topic};

pub use maiko_macros::Event;

pub type Result<T> = std::result::Result<T, Error>;
