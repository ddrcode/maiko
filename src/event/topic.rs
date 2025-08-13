use std::fmt::Debug;
use std::hash::Hash;

/// A topic that can be used for message routing
pub trait Topic: Clone + Debug + Hash + Eq + Send + Sync + 'static {
    /// Returns the default topic name for this type
    fn name(&self) -> String;
}

/// Implements Topic for String with a direct name mapping
impl Topic for String {
    fn name(&self) -> String {
        self.clone()
    }
}