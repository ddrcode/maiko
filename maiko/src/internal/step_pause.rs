#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub(crate) enum StepPause {
    #[default]
    None,
    AwaitEvent,
    Suppressed,
}
