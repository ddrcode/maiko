use core::marker::Send;

use async_trait::async_trait;

use crate::{Error, Event, Meta, Result};

/// Core trait implemented by user-defined actors.
///
/// An actor processes incoming events and can optionally perform periodic
/// work in `tick`, as well as lifecycle hooks in `on_start` and `on_shutdown`.
///
/// Implementors typically hold any state they need, and use the runtime-provided
/// `Context<E>` (via a constructor/factory passed to `Supervisor::add_actor`) to
/// emit events and stop gracefully.

#[allow(unused_variables)]
#[async_trait]
pub trait Actor: Send {
    type Event: Event + Send;

    /// Handle a single incoming event.
    ///
    /// Called for every event routed to this actor. Return `Ok(())` when
    /// processing succeeds, or an error to signal failure. Use `Context::send`
    /// to emit follow-up events as needed.
    async fn handle(&mut self, event: &Self::Event, meta: &Meta) -> Result<()> {
        Ok(())
    }

    /// Optional periodic work.
    ///
    /// If implemented, this will be polled in the actor loop alongside
    /// event reception. Keep it lightweight and non-blocking.
    async fn tick(&mut self) -> Result<()> {
        // std::future::pending::<()>().await;
        Ok(())
    }

    /// Lifecycle hook called once before the event loop starts.
    async fn on_start(&mut self) -> Result<()> {
        Ok(())
    }

    /// Lifecycle hook called once after the event loop stops.
    async fn on_shutdown(&mut self) -> Result<()> {
        Ok(())
    }

    /// Called when an error is returned by `handle` or `tick`.
    /// Return `true` to propagate (terminate the actor), or `false` to
    /// swallow and continue.
    fn on_error(&self, error: &Error) -> bool {
        true
    }
}
