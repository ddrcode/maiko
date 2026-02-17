use crate::Config;

/// Per-actor configuration.
///
/// Controls settings that vary between actors, such as the mailbox channel
/// capacity. Created automatically by [`Supervisor::add_actor`] using global
/// defaults from [`Config`], or customized via [`ActorBuilder`].
///
/// # Examples
///
/// ```rust,ignore
/// // Via the builder (preferred)
/// sup.build_actor("writer", |ctx| Writer::new(ctx))
///     .channel_capacity(512)
///     .topics(&[Topic::Data])
///     .build()?;
///
/// // Standalone
/// let config = ActorConfig::new(sup.config())
///     .with_channel_capacity(512);
/// ```
///
/// [`Supervisor::add_actor`]: crate::Supervisor::add_actor
/// [`ActorBuilder`]: crate::ActorBuilder
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ActorConfig {
    channel_capacity: usize,
}

impl ActorConfig {
    /// Create a new config inheriting defaults from the global [`Config`].
    pub fn new(global_config: &Config) -> Self {
        Self {
            channel_capacity: global_config.default_actor_channel_capacity(),
        }
    }

    /// Set the actor's mailbox channel capacity.
    ///
    /// This is the number of events that can be queued for this actor
    /// before the broker's overflow policy takes effect.
    pub fn with_channel_capacity(mut self, channel_capacity: usize) -> Self {
        self.channel_capacity = channel_capacity;
        self
    }

    /// Returns the actor's mailbox channel capacity.
    pub fn channel_capacity(&self) -> usize {
        self.channel_capacity
    }
}
