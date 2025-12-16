/// Runtime configuration for the supervisor and actors.
///
/// Controls channel buffer sizes and event batching behavior. Use the builder
/// pattern to customize, or use [`Default`] for sensible defaults.
///
/// # Examples
///
/// ```rust
/// use maiko::Config;
///
/// let config = Config::default()
///     .with_channel_size(256)            // Larger buffers for high throughput
///     .with_max_events_per_tick(20);     // Process more events per cycle
/// ```
pub struct Config {
    /// Size of the channel buffer for each actor (and the broker).
    /// Determines how many events can be queued before backpressure applies.
    /// Default: 128
    pub channel_size: usize,

    /// Maximum number of events an actor will process in a single tick cycle
    /// before yielding control back to the scheduler.
    /// Lower values improve fairness, higher values improve throughput.
    /// Default: 10
    pub max_events_per_tick: usize,

    /// Duration to wait during shutdown before cancelling actors.
    /// This gives the broker time to process in-flight events.
    /// Default: 10 ms. Set to Duration::ZERO for immediate shutdown.
    pub sleep_on_shutdown: tokio::time::Duration,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            channel_size: 128,
            max_events_per_tick: 10,
            sleep_on_shutdown: tokio::time::Duration::from_millis(10),
        }
    }
}

impl Config {
    /// Set the channel buffer size for actors and the broker.
    ///
    /// Larger buffers allow more queued events but use more memory.
    /// When the buffer is full, senders will block (backpressure).
    pub fn with_channel_size(mut self, size: usize) -> Self {
        self.channel_size = size;
        self
    }

    /// Set the maximum number of events processed per tick cycle.
    ///
    /// This controls batching behavior in the actor event loop.
    /// After processing this many events, the actor yields to allow
    /// other tasks to run and to call [`Actor::tick`].
    ///
    /// Trade-offs:
    /// - Lower values (1-5): Better fairness, more responsive `tick()`, higher overhead
    /// - Higher values (50-100): Better throughput, potential starvation of `tick()`
    ///
    /// [`Actor::tick`]: crate::Actor::tick
    pub fn with_max_events_per_tick(mut self, limit: usize) -> Self {
        self.max_events_per_tick = limit;
        self
    }
}
