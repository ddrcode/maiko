use std::pin::Pin;

use tokio::time::Sleep;

use crate::internal::StepPause;

#[derive(Default)]
pub(crate) struct StepHandler {
    pub backoff: Option<Pin<Box<Sleep>>>,
    pub pause: StepPause,
}

impl StepHandler {
    pub fn is_delayed(&self) -> bool {
        self.backoff.is_some() && self.pause == StepPause::None
    }

    pub fn can_step(&self) -> bool {
        self.backoff.is_none() && self.pause == StepPause::None
    }

    pub fn reset(&mut self) {
        self.backoff = None;
        self.pause = StepPause::None;
    }
}
