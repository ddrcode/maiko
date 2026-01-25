use std::sync::Arc;

/// A lightweight handle to a registered actor.
///
/// Returned by [`Supervisor::add_actor`](crate::Supervisor::add_actor) and
/// [`ActorBuilder::build`](crate::ActorBuilder::build). Use handles to:
///
/// - Identify actors in test assertions
/// - Reference actors for event injection in tests
///
/// Handles are cheap to clone and can be stored for later use.
///
/// # Example
///
/// ```ignore
/// let producer = sup.add_actor("producer", |ctx| Producer::new(ctx), &[DefaultTopic])?;
/// let consumer = sup.add_actor("consumer", |ctx| Consumer::new(ctx), &[DefaultTopic])?;
///
/// // Use handles in test harness
/// let mut test = sup.init_test_harness().await;
/// test.send_as(&producer, MyEvent::Data(42)).await?;
/// assert!(test.actor(&consumer).inbound_count() > 0);
/// ```
#[derive(Debug, Clone)]
pub struct ActorHandle {
    pub(crate) name: Arc<str>,
}

impl ActorHandle {
    /// Create a new actor handle with the given name.
    pub fn new(name: Arc<str>) -> Self {
        Self { name }
    }

    /// Returns the actor's name as registered with the supervisor.
    pub fn name(&self) -> &str {
        &self.name
    }
}
