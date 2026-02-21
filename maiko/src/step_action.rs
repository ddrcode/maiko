use std::{fmt, hash, time::Duration};

/// Action returned by an actor `step` to influence scheduling.
#[derive(Debug, Clone, Copy, PartialEq, Eq, hash::Hash)]
pub enum StepAction {
    /// Keep running and allow other branches to progress.
    Continue,
    /// No immediate need to run `step` again; defer to other branches.
    Yield,
    /// Pause periodic/backoff scheduling until the next event.
    AwaitEvent,
    /// Sleep before the next `step` to avoid busy looping.
    Backoff(Duration),
    /// Disable periodic/backoff scheduling. Stepping will be ignored by select!
    Never,
}

impl fmt::Display for StepAction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            StepAction::Continue => write!(f, "Continue"),
            StepAction::Yield => write!(f, "Yield"),
            StepAction::AwaitEvent => write!(f, "AwaitEvent"),
            StepAction::Backoff(_) => write!(f, "Backoff"),
            StepAction::Never => write!(f, "Never"),
        }
    }
}
