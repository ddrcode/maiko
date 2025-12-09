pub struct Config {
    pub channel_size: usize,
}

impl Default for Config {
    fn default() -> Self {
        Config { channel_size: 128 }
    }
}

impl Config {
    pub fn with_channel_size(mut self, size: usize) -> Self {
        self.channel_size = size;
        self
    }
}
