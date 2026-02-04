//! Ready-to-use monitor implementations.
//!
//! This module contains concrete [`Monitor`](crate::monitoring::Monitor) implementations
//! for common use cases like event recording and logging.
//!
//! # Available Monitors
//!
//! - [`Recorder`] - Records events to a JSON Lines file (requires `recorder` feature)
//!
//! # Example
//!
//! ```ignore
//! use maiko::monitors::Recorder;
//!
//! let recorder = Recorder::new("events.jsonl")?;
//! sup.monitors().add(recorder).await;
//! ```

#[cfg(feature = "recorder")]
mod recorder;

#[cfg(feature = "recorder")]
pub use recorder::Recorder;
