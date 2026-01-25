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
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ActorHandle {
    pub(crate) id: u64,
    pub(crate) name: Arc<str>,
}

impl ActorHandle {
    pub(crate) fn new(id: u64, name: Arc<str>) -> Self {
        Self { id, name }
    }

    #[inline(always)]
    pub fn id(&self) -> u64 {
        self.id
    }

    /// Returns the actor's name as registered with the supervisor.
    #[inline]
    pub fn name(&self) -> &str {
        &self.name
    }
}

impl PartialEq for ActorHandle {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl std::fmt::Display for ActorHandle {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name)
    }
}
