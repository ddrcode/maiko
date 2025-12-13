use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};

use tokio::sync::mpsc::Sender;
use tokio_util::sync::CancellationToken;

use crate::{Envelope, Event, Meta, Result};

/// Runtime-provided context for an actor to interact with the system.
///
/// Use it to:
/// - `send(event)`: emit events into the broker tagged with this actor's name
/// - `stop()`: request graceful shutdown of this actor (and trigger global cancel)
/// - `name()`: retrieve the actor's name for logging/identity
/// - `is_alive()`: check whether the actor loop should continue running
#[derive(Clone)]
pub struct Context<E: Event> {
    pub(crate) name: Arc<str>,
    pub(crate) sender: Sender<Envelope<E>>,
    pub(crate) alive: Arc<AtomicBool>,
    pub(crate) cancel_token: Arc<CancellationToken>,
}

impl<E: Event> Context<E> {
    /// Send an event to the broker. The envelope will carry this actor's name.
    pub async fn send(&self, event: E) -> Result<()> {
        self.send_envelope(Envelope::new(event, self.name.clone()))
    }

    pub async fn send_with_correlation(&self, event: E, correlation_id: u128) -> Result<()> {
        self.send_envelope(Envelope::with_correlation_id(
            event,
            self.name.clone(),
            correlation_id,
        ))
    }

    pub async fn send_child_event(&self, event: E, meta: &Meta) -> Result<()> {
        self.send_envelope(Envelope::with_correlation_id(
            event,
            self.name.clone(),
            meta.id(),
        ))
    }

    #[inline]
    fn send_envelope(&self, envelope: Envelope<E>) -> Result<()> {
        self.sender.try_send(envelope)?;
        Ok(())
    }

    /// Signal this actor to stop and trigger the cancellation token.
    pub fn stop(&self) {
        self.alive.store(false, Ordering::Release);
        self.cancel_token.cancel();
    }

    /// The actor's name as registered with the supervisor.
    #[inline]
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Whether the actor is considered alive by the runtime.
    #[inline]
    pub fn is_alive(&self) -> bool {
        self.alive.load(Ordering::Relaxed)
    }
}
