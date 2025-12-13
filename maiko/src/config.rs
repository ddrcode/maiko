pub struct Config {
    pub channel_size: usize,
    pub drain_limit: usize,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            channel_size: 128,
            drain_limit: 10,
        }
    }
}

impl Config {
    pub fn with_channel_size(mut self, size: usize) -> Self {
        self.channel_size = size;
        self
    }

    pub fn with_drain_limit(mut self, limit: usize) -> Self {
        self.drain_limit = limit;
        self
    }
}
