//! Test harness for observing and asserting on event flow.
//!
//! Enable with the `test-harness` feature:
//!
//! ```toml
//! [dev-dependencies]
//! maiko = { version = "0.2", features = ["test-harness"] }
//! ```
//!
//! # Example
//!
//! ```ignore
//! let mut test = supervisor.init_test_harness().await;
//! supervisor.start().await?;
//!
//! test.start_recording().await;
//! let id = test.send_as(&producer, MyEvent::Data(42)).await?;
//! test.stop_recording().await;
//!
//! // Query using spies
//! assert!(test.event(id).was_delivered_to(&consumer));
//! assert_eq!(1, test.actor(&consumer).inbound_count());
//!
//! // Or use EventQuery for complex queries
//! let orders = test.events()
//!     .sent_by(&trader)
//!     .matching_event(|e| matches!(e, MyEvent::Order(_)))
//!     .count();
//! ```

mod actor_spy;
mod event_collector;
mod event_entry;
mod event_query;
mod event_spy;
mod harness;
mod test_event;
mod topic_spy;

pub use actor_spy::ActorSpy;
pub(crate) use event_collector::EventCollector;
pub use event_entry::EventEntry;
pub use event_query::EventQuery;
pub use event_spy::EventSpy;
pub use harness::Harness;
pub use topic_spy::TopicSpy;

pub(crate) type EventRecords<E, T> = Vec<EventEntry<E, T>>;
