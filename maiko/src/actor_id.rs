use std::{hash::Hash, ops::Deref, sync::Arc};

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
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ActorId(Arc<str>);

impl ActorId {
    pub(crate) fn new(id: Arc<str>) -> Self {
        Self(id)
    }

    /// Returns the actor's name as registered with the supervisor.
    #[inline]
    pub fn name(&self) -> &str {
        &self.0
    }
}

impl PartialEq for ActorId {
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.0, &other.0)
    }
}

impl Eq for ActorId {}

impl std::fmt::Display for ActorId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Hash for ActorId {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.0.hash(state);
    }
}

impl Deref for ActorId {
    type Target = str;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
