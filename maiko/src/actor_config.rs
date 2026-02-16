#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ActorConfig {
    channel_capacity: usize,
}

impl ActorConfig {
    pub fn new() -> Self {
        ActorConfig::default()
    }

    pub fn with_channel_capacity(mut self, channel_capacity: usize) -> Self {
        self.channel_capacity = channel_capacity;
        self
    }

    pub fn channel_capacity(&self) -> usize {
        self.channel_capacity
    }
}

impl Default for ActorConfig {
    fn default() -> Self {
        Self {
            channel_capacity: 128,
        }
    }
}
