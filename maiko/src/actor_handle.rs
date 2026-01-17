use std::sync::Arc;

pub struct ActorHandle {
    name: Arc<str>,
}

impl ActorHandle {
    pub fn new(name: Arc<str>) -> Self {
        Self { name }
    }

    pub fn name(&self) -> &str {
        &self.name
    }
}

impl<'a> From<&'a ActorHandle> for &'a str {
    fn from(val: &'a ActorHandle) -> Self {
        val.name.as_ref()
    }
}
