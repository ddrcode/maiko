use std::borrow::Cow;

/// Human-readable label for an event variant.
///
/// Used by the test harness for event matching (e.g. `chain.events().contains("KeyPress")`),
/// Mermaid diagram generation, and the `Recorder` monitor for JSON output.
///
/// Derive it automatically with `#[derive(Label)]` (requires the `macros` feature).
pub trait Label {
    /// Returns a human-readable label for this item.
    fn label(&self) -> Cow<'static, str>;
}
