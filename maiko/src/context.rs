use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};

use tokio::sync::mpsc::Sender;

use crate::{ActorId, Envelope, Event, EventId, Meta, Result};

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
    pub(crate) actor_id: ActorId,
    pub(crate) sender: Sender<Arc<Envelope<E>>>,
    pub(crate) alive: Arc<AtomicBool>,
}

impl<E: Event> Context<E> {
    pub fn new(
        actor_id: ActorId,
        sender: Sender<Arc<Envelope<E>>>,
        alive: Arc<AtomicBool>,
    ) -> Self {
        Self {
            actor_id,
            sender,
            alive,
        }
    }

    /// Send an event to the broker. The envelope will carry this actor's name.
    /// This awaits channel capacity (backpressure) to avoid silent drops.
    pub async fn send(&self, event: E) -> Result<()> {
        let envelope = Envelope::new(event, self.actor_id.clone());
        self.send_envelope(envelope).await
    }

    /// Send an event with an explicit correlation id.
    pub async fn send_with_correlation(&self, event: E, correlation_id: EventId) -> Result<()> {
        self.send_envelope(Envelope::with_correlation(
            event,
            self.actor_id.clone(),
            correlation_id,
        ))
        .await
    }

    /// Emit a child event correlated to the given parent `Meta`.
    pub async fn send_child_event(&self, event: E, meta: &Meta) -> Result<()> {
        self.send_envelope(Envelope::with_correlation(
            event,
            self.actor_id.clone(),
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

    #[inline]
    pub fn actor_id(&self) -> &ActorId {
        &self.actor_id
    }

    /// The actor's name as registered with the supervisor.
    #[inline]
    pub fn actor_name(&self) -> &str {
        self.actor_id.name()
    }

    /// Whether the actor is considered alive by the runtime.
    #[inline]
    pub fn is_alive(&self) -> bool {
        self.alive.load(Ordering::Acquire)
    }

    /// Returns a future that never completes.
    ///
    /// **Note:** This method is largely obsolete. The default `step()` implementation
    /// now returns `StepAction::Never`, which disables stepping entirely. You only need
    /// `pending()` if you want to block inside a custom `step()` implementation.
    ///
    /// ```rust,ignore
    /// // Prefer this (default behavior):
    /// async fn step(&mut self) -> Result<StepAction> {
    ///     Ok(StepAction::Never)
    /// }
    ///
    /// // Or simply don't implement step() at all - it defaults to Never
    /// ```
    #[inline]
    pub async fn pending(&self) -> Result<()> {
        std::future::pending::<()>().await;
        Ok(())
    }

    #[inline]
    pub fn is_sender_full(&self) -> bool {
        self.sender.capacity() == 0
    }
}
