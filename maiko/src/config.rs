pub struct Config {
    pub channel_size: usize,
    pub max_events_per_tick: usize,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            channel_size: 128,
            max_events_per_tick: 10,
        }
    }
}

impl Config {
    pub fn with_channel_size(mut self, size: usize) -> Self {
        self.channel_size = size;
        self
    }

    pub fn with_max_events_per_tick(mut self, limit: usize) -> Self {
        self.max_events_per_tick = limit;
        self
    }
}
