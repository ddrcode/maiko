//! # Maiko
//!
//! A lightweight actor runtime for Tokio with topic-based pub/sub routing.
//!
//! Maiko provides independent actors that communicate through asynchronous events—no shared
//! memory, no locks, no channel wiring. Each actor processes messages sequentially from its
//! own mailbox, making concurrent systems easier to reason about.
//!
//! ## Quick Start
//!
//! ```rust,no_run
//! use maiko::*;
//!
//! #[derive(Event, Clone, Debug)]
//! enum MyEvent {
//!     Hello(String),
//! }
//!
//! struct Greeter;
//!
//! impl Actor for Greeter {
//!     type Event = MyEvent;
//!
//!     async fn handle_event(&mut self, envelope: &Envelope<Self::Event>) -> Result<()> {
//!         if let MyEvent::Hello(name) = envelope.event() {
//!             println!("Hello, {}!", name);
//!         }
//!         Ok(())
//!     }
//! }
//!
//! #[tokio::main]
//! async fn main() -> Result<()> {
//!     let mut sup = Supervisor::<MyEvent>::default();
//!     sup.add_actor("greeter", |_ctx| Greeter, &[DefaultTopic])?;
//!
//!     sup.start().await?;
//!     sup.send(MyEvent::Hello("World".into())).await?;
//!     sup.stop().await
//! }
//! ```
//!
//! ## Core Types
//!
//! | Type | Description |
//! |------|-------------|
//! | [`Event`] | Marker trait for event types (use `#[derive(Event)]`) |
//! | [`Actor`] | Trait for implementing actors |
//! | [`Topic`] | Routes events to interested actors |
//! | [`Supervisor`] | Manages actor lifecycles and runtime |
//! | [`Context`] | Allows actors to send events and interact with runtime |
//! | [`Envelope`] | Wraps events with metadata (sender, correlation ID) |
//! | [`ActorId`] | Unique identifier for a registered actor |
//!
//! ## Topic-Based Routing
//!
//! Actors subscribe to topics, and events are automatically routed to all interested subscribers.
//! Use [`DefaultTopic`] for simple broadcasting, or implement [`Topic`] for custom routing:
//!
//! ```rust,ignore
//! #[derive(Debug, Hash, Eq, PartialEq, Clone)]
//! enum MyTopic { Data, Control }
//!
//! impl Topic<MyEvent> for MyTopic {
//!     fn from_event(event: &MyEvent) -> Self {
//!         match event {
//!             MyEvent::Data(_) => MyTopic::Data,
//!             MyEvent::Control(_) => MyTopic::Control,
//!         }
//!     }
//! }
//!
//! sup.add_actor("processor", |ctx| Processor::new(ctx), &[MyTopic::Data])?;
//! ```
//!
//! ## Features
//!
//! - **`macros`** (default) — Enables `#[derive(Event)]` macro
//! - **`test-harness`** — Enables test utilities for asserting on event flow
//!
//! ## Examples
//!
//! See the [`examples/`](https://github.com/ddrcode/maiko/tree/main/maiko/examples) directory:
//!
//! - `pingpong.rs` — Simple event exchange between actors
//! - `guesser.rs` — Multi-actor game with topics and timing
//! - `arbitrage.rs` — Test harness demonstration

mod actor;
mod actor_builder;
mod actor_id;
mod config;
mod context;
mod envelope;
mod error;
mod event;
mod meta;
mod step_action;
mod supervisor;
mod topic;

mod internal;

#[cfg(feature = "test-harness")]
pub mod testing;

#[cfg(feature = "monitoring")]
pub mod monitoring;

pub use actor::Actor;
pub use actor_builder::ActorBuilder;
pub use actor_id::ActorId;
pub use config::Config;
pub use context::Context;
pub use envelope::Envelope;
pub use error::Error;
pub use event::Event;
pub use meta::Meta;
pub use step_action::StepAction;
pub use supervisor::Supervisor;
pub use topic::{DefaultTopic, Topic};

#[cfg(feature = "macros")]
pub use maiko_macros::Event;

pub type Result<T = ()> = std::result::Result<T, Error>;
pub type EventId = u128;
