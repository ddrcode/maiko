//! Ready-to-use monitor implementations.
//!
//! This module contains concrete [`Monitor`](crate::monitoring::Monitor) implementations
//! for common use cases like event recording and logging.
//!
//! # Available Monitors
//!
//! - [`Tracer`] - Logs event lifecycle via `tracing` crate
//! - [`Recorder`] - Records events to a JSON Lines file (requires `recorder` feature)
//!
//! # Example
//!
//! ```ignore
//! use maiko::monitors::Tracer;
//!
//! sup.monitors().add(Tracer).await;
//! ```

mod tracer;
pub use tracer::Tracer;

#[cfg(feature = "recorder")]
mod recorder;

#[cfg(feature = "recorder")]
pub use recorder::Recorder;
