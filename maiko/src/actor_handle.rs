use std::sync::Arc;

#[derive(Debug, Clone)]
pub struct ActorHandle {
    pub(crate) name: Arc<str>,
}

impl ActorHandle {
    pub fn new(name: Arc<str>) -> Self {
        Self { name }
    }

    pub fn name(&self) -> &str {
        &self.name
    }
}
