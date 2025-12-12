//! Maiko — Event-based actor runtime
//!
//! A tiny actor runtime inspired by event-driven systems like Kafka,
//! designed for ergonomic, loosely-coupled concurrency in Rust.
//!
//! Quick start:
//! - Define your `Event` enum and `Topic` mapping.
//! - Implement `Actor` for your type; the runtime injects an `ActorHandle` for emitting events and stopping.
//! - Register actors via `Supervisor::add_actor` and call `start()`.
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
pub use {topic::DefaultTopic, topic::Topic};

pub type Result<T> = std::result::Result<T, Error>;

pub mod prelude {
    //! Convenience imports for typical Maiko usage.
    pub use crate::actor::Actor;
    pub use crate::context::Context;
    pub use crate::error::Error as MaikoError;
    pub use crate::event::Event;
    pub use crate::meta::Meta;
    pub use crate::supervisor::Supervisor;
    pub use crate::topic::Topic;
}
