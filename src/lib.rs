//! Maiko — Event-based actor runtime
//!
//! A tiny actor runtime inspired by event-driven systems like Kafka,
//! designed for ergonomic, loosely-coupled concurrency in Rust.
//!
//! Work in progress. Stay tuned!

mod actor;
mod broker;
mod context;
mod envelope;
mod error;
mod event;
mod meta;
mod subscriber;
mod topic;

pub use actor::Actor;
pub use broker::Broker;
pub use context::Context;
pub use envelope::Envelope;
pub use error::Error;
pub use event::Event;
pub use meta::Meta;
pub(crate) use subscriber::Subscriber;
pub use topic::Topic;

pub type Result<T> = std::result::Result<T, Error>;
pub type Sender<T: Event> = tokio::sync::mpsc::Sender<Envelope<T>>;
pub type Receiver<T: Event> = tokio::sync::mpsc::Receiver<Envelope<T>>;
