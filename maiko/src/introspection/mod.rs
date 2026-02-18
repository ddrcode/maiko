//! Actor introspection API for runtime observability.
//!
//! Enable with the `introspection` feature:
//!
//! ```toml
//! [dependencies]
//! maiko = { version = "0.2", features = ["introspection"] }
//! ```
//!
//! # Overview
//!
//! The introspection system provides live visibility into actor state:
//! - List all registered actors and their current status
//! - Query mailbox queue depths in real time
//! - Track event throughput and error counts
//! - Take point-in-time system snapshots
//!
//! Built on the existing [`monitoring`](crate::monitoring) system using the
//! observer pattern.
//!
//! # Example
//!
//! ```ignore
//! let mut sup = Supervisor::<MyEvent>::default();
//! sup.add_actor("worker", |ctx| Worker::new(ctx), &[DefaultTopic])?;
//!
//! sup.start().await?;
//!
//! // Query actor state
//! let actors = sup.introspection().list_actors();
//! for actor in &actors {
//!     println!("{}: {} (queue: {}/{})",
//!         actor.actor_id.name(),
//!         actor.status,
//!         actor.mailbox_depth,
//!         actor.mailbox_capacity,
//!     );
//! }
//!
//! // Take a system snapshot
//! let snapshot = sup.introspection().snapshot();
//! ```

mod actor_info;
mod introspector;
pub(crate) mod shared_state;
pub(crate) mod state_tracker;

pub use actor_info::{ActorInfo, ActorStatus, SystemSnapshot};
pub use introspector::Introspector;
