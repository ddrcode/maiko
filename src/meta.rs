use std::time::{Instant, SystemTime};

use uuid::Uuid;

pub struct Meta {
    id: Uuid,
    timestamp: u64,
}

impl Meta {
    pub fn new() -> Self {
        Self {
            id: Uuid::new_v4(),
            timestamp: 0, // timestamp: Instant::now()
                          //     .duration_since(SystemTime::UNIX_EPOCH.)
                          //     .as_nanos(),
        }
    }

    pub fn id(&self) -> Uuid {
        self.id
    }

    pub fn timestamp(&self) -> u64 {
        self.timestamp
    }
}
