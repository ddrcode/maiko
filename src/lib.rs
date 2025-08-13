//! Maiko - A lightweight, topic-based actor system
//!
//! Maiko provides a simple but powerful actor system based on topics rather than
//! direct addressing. It emphasizes loose coupling and event-driven design.

mod actor;
mod error;
mod event;

pub use actor::{Actor, ActorExt};
pub use error::Error;
pub use event::{Event, Topic};

/// Re-export common types for convenience
pub mod prelude {
    pub use crate::{Actor, ActorExt, Error, Result};
}

pub type Result<T> = std::result::Result<T, Error>;

