use core::marker::Send;
use std::future::Future;

use crate::{Error, Event, Meta, Result};

/// Core trait implemented by user-defined actors.
///
/// An actor processes incoming events and can optionally perform periodic
/// work in `tick`, as well as lifecycle hooks in `on_start` and `on_shutdown`.
///
/// Implementors typically hold any state they need, and use the runtime-provided
/// `Context<E>` (via a constructor/factory passed to `Supervisor::add_actor`) to
/// emit events and stop gracefully.
///
/// Ergonomics:
/// - Although the trait methods return futures, you can implement them as `async fn`
///   with a simple `Result<()>` return. The compiler will produce the appropriate
///   future type automatically.
/// - No `#[async_trait]` is required.
///
/// See also: [`crate::Context`], [`crate::Supervisor`].
#[allow(unused_variables)]
// #[async_trait]
pub trait Actor: Send {
    type Event: Event + Send;

    /// Handle a single incoming event.
    ///
    /// Called for every event routed to this actor. Return `Ok(())` when
    /// processing succeeds, or an error to signal failure. Use `Context::send`
    /// to emit follow-up events as needed.
    fn handle(
        &mut self,
        event: &Self::Event,
        meta: &Meta,
    ) -> impl Future<Output = Result<()>> + Send {
        async { Ok(()) }
    }

    /// Optional periodic work called when the event queue is empty.
    ///
    /// This runs in a `select!` loop alongside event reception. What you `.await`
    /// inside `tick()` determines when your actor wakes up.
    ///
    /// # Common Patterns
    ///
    /// **Time-Based Producer** (polls periodically):
    /// ```rust,ignore
    /// async fn tick(&mut self) -> Result<()> {
    ///     tokio::time::sleep(Duration::from_secs(1)).await;
    ///     let data = generate_data();
    ///     self.ctx.send(DataEvent(data)).await
    /// }
    /// ```
    ///
    /// **External Event Source** (driven by I/O):
    /// ```rust,ignore
    /// async fn tick(&mut self) -> Result<()> {
    ///     let frame = self.websocket.read().await?;
    ///     self.ctx.send(WebSocketEvent(frame)).await
    /// }
    /// ```
    ///
    /// **Housekeeping Only** (runs after processing events):
    /// ```rust,ignore
    /// async fn tick(&mut self) -> Result<()> {
    ///     if self.should_flush() {
    ///         self.flush_buffer().await?;
    ///     }
    ///     Ok(())  // Returns immediately
    /// }
    /// ```
    ///
    /// **No Periodic Logic** (pure event processor):
    /// ```rust,ignore
    /// async fn tick(&mut self) -> Result<()> {
    ///     self.ctx.pending().await  // Never returns - actor only reacts to events
    /// }
    /// ```
    ///
    /// # Default Behavior
    ///
    /// The default implementation returns a pending future that never completes,
    /// making the actor purely event-driven with no periodic work.
    ///
    /// [`Config::max_events_per_tick`]: crate::Config::max_events_per_tick
    fn tick(&mut self) -> impl Future<Output = Result<()>> + Send {
        async {
            std::future::pending::<()>().await;
            Ok(())
        }
    }

    /// Lifecycle hook called once before the event loop starts.
    fn on_start(&mut self) -> impl Future<Output = Result<()>> + Send {
        async { Ok(()) }
    }

    /// Lifecycle hook called once after the event loop stops.
    fn on_shutdown(&mut self) -> impl Future<Output = Result<()>> + Send {
        async { Ok(()) }
    }

    /// Called when an error is returned by [`handle`](Actor::handle) or [`tick`](Actor::tick).
    ///
    /// Return `Ok(())` to swallow the error and continue processing,
    /// or `Err(error)` to propagate and stop the actor.
    ///
    /// # Default Behavior
    ///
    /// By default, all errors propagate (actor stops). Override this to implement
    /// custom error handling, logging, or recovery logic.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use maiko::{Actor, Error, Event, Result};
    /// # #[derive(Clone, Event)]
    /// # struct MyEvent;
    /// # struct MyActor;
    /// # impl Actor for MyActor {
    /// #     type Event = MyEvent;
    /// fn on_error(&self, error: Error) -> Result<()> {
    ///     eprintln!("Actor error: {}", error);
    ///     Ok(())  // Swallow and continue
    /// }
    /// # }
    /// ```
    fn on_error(&self, error: Error) -> Result<()> {
        Err(error)
    }
}
