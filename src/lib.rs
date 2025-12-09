//! Maiko — an Event-based actor runtime
//!
//! A tiny actor runtime inspired by event-driven systems like Kafka,
//! designed for ergonomic, loosely-coupled concurrency in Rust.

mod actor;
mod config;
mod context;
mod envelope;
mod error;
mod event;
mod meta;
mod supervisor;
mod topic;

pub mod internal;

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
    pub use crate::actor::Actor;
    pub use crate::context::Context;
    pub use crate::error::Error as MaikoError;
    pub use crate::event::Event;
    pub use crate::meta::Meta;
    pub use crate::supervisor::Supervisor;
    pub use crate::topic::Topic;
}
