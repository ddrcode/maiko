use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};

use tokio::sync::mpsc::Sender;

use crate::{Envelope, Event, Meta, Result};

/// Runtime-provided context for an actor to interact with the system.
///
/// Use it to:
/// - `send(event)`: emit events into the broker tagged with this actor's name
/// - `stop()`: request graceful shutdown of this actor (and trigger global cancel)
/// - `name()`: retrieve the actor's name for logging/identity
/// - `is_alive()`: check whether the actor loop should continue running
///
/// Correlation:
/// - `send_with_correlation(event, id)`: emit an event linked to a specific correlation id.
/// - `send_child_event(event, meta)`: convenience to set correlation id to the parent `meta.id()`.
///
/// See also: [`Envelope`], [`Meta`], [`crate::Supervisor`].
#[derive(Clone)]
pub struct Context<E: Event> {
    pub(crate) name: Arc<str>,
    pub(crate) sender: Sender<Arc<Envelope<E>>>,
    pub(crate) alive: Arc<AtomicBool>,
}

impl<E: Event> Context<E> {
    pub fn new(name: Arc<str>, sender: Sender<Arc<Envelope<E>>>, alive: Arc<AtomicBool>) -> Self {
        Self {
            name,
            sender,
            alive,
        }
    }

    /// Send an event to the broker. The envelope will carry this actor's name.
    /// This awaits channel capacity (backpressure) to avoid silent drops.
    pub async fn send(&self, event: E) -> Result<()> {
        self.send_envelope(Envelope::new(event, self.name.clone()))
            .await
    }

    /// Send an event with an explicit correlation id.
    pub async fn send_with_correlation(&self, event: E, correlation_id: u128) -> Result<()> {
        self.send_envelope(Envelope::with_correlation(
            event,
            self.name.clone(),
            correlation_id,
        ))
        .await
    }

    /// Emit a child event correlated to the given parent `Meta`.
    pub async fn send_child_event(&self, event: E, meta: &Meta) -> Result<()> {
        self.send_envelope(Envelope::with_correlation(
            event,
            self.name.clone(),
            meta.id(),
        ))
        .await
    }

    #[inline]
    pub async fn send_envelope(&self, envelope: Envelope<E>) -> Result<()> {
        self.sender.send(Arc::new(envelope)).await?;
        Ok(())
    }

    /// Signal this actor to stop
    #[inline]
    pub fn stop(&mut self) {
        self.alive.store(false, Ordering::Release);
    }

    /// The actor's name as registered with the supervisor.
    #[inline]
    pub fn name(&self) -> &str {
        &self.name
    }

    #[inline]
    pub fn clone_name(&self) -> Arc<str> {
        self.name.clone()
    }

    /// Whether the actor is considered alive by the runtime.
    #[inline]
    pub fn is_alive(&self) -> bool {
        self.alive.load(Ordering::Acquire)
    }

    /// Returns a future that never completes.
    ///
    /// Use this in [`Actor::tick`](crate::Actor::tick) when you don't need periodic logic
    /// and want your actor to be purely event-driven:
    ///
    /// ```rust,ignore
    /// impl Actor for MyEventProcessor {
    ///     async fn tick(&mut self) -> Result<()> {
    ///         self.ctx.pending().await  // Never returns - only reacts to events
    ///     }
    ///
    ///     async fn handle(&mut self, event: &Event, meta: &Meta) -> Result<()> {
    ///         // All logic here - purely reactive
    ///         process(event).await
    ///     }
    /// }
    /// ```
    ///
    /// This is more ergonomic than `std::future::pending()` and matches the
    /// `Result<()>` return type expected by `tick()`.
    #[inline]
    pub async fn pending(&self) -> Result<()> {
        std::future::pending::<()>().await;
        Ok(())
    }
}
