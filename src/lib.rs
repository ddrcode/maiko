//! Maiko — Event-based actor runtime
//!
//! A tiny actor runtime inspired by event-driven systems like Kafka,
//! designed for ergonomic, loosely-coupled concurrency in Rust.
//!
//! Work in progress. Stay tuned!

mod actor;
mod broker;
mod config;
mod context;
mod envelope;
mod error;
mod event;
mod meta;
mod subscriber;
mod supervisor;
mod topic;

pub use actor::Actor;
pub use broker::Broker;
pub use config::Config;
pub use context::Context;
pub use envelope::Envelope;
pub use error::Error;
pub use event::Event;
pub use meta::Meta;
pub(crate) use subscriber::Subscriber;
pub use supervisor::Supervisor;
pub use {topic::DefaultTopic, topic::Topic};

pub type Result<T> = std::result::Result<T, Error>;

pub mod macros {
    pub use maiko_macros::*;
}

pub mod prelude {
    pub use crate::actor::Actor;
    pub use crate::context::Context;
    pub use crate::error::Error as MaikoError;
    pub use crate::meta::Meta;
    pub use crate::supervisor::Supervisor;
    pub use crate::topic::Topic;
    pub use maiko_macros::Event;
}
