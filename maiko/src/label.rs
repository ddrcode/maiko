use std::borrow::Cow;

pub trait Label {
    /// Returns a human-readable label for this item.
    /// This is used for logging, monitoring, and diagram generation.
    fn label(&self) -> Cow<'static, str>;
}
