use std::{sync::Arc, time::SystemTime};

use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct Meta {
    id: u128,
    timestamp: u64,
    sender: Arc<str>,
}

impl Meta {
    pub fn new(sender: &str) -> Self {
        Self {
            id: Uuid::new_v4().as_u128(),
            timestamp: SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .expect("SystemTime before Unix epoch")
                .as_nanos() as u64,
            sender: Arc::from(sender),
        }
    }

    pub fn id(&self) -> u128 {
        self.id
    }

    pub fn timestamp(&self) -> u64 {
        self.timestamp
    }

    pub fn sender(&self) -> &str {
        &self.sender
    }
}
