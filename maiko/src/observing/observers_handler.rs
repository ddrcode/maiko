use std::sync::Arc;

use tokio_util::sync::CancellationToken;

use crate::{Event, Topic};

pub struct ObserversHandler<E: Event, T: Topic<E>> {
    cancel_token: Arc<CancellationToken>,
}

impl<E: Event, T: Topic<E>> ObserversHandler<E, T> {
    pub(crate) fn new() -> Self {
        Self {
            cancel_token: Arc::new(CancellationToken::new()),
        }
    }

    pub(crate) fn stop(&self) {
        self.cancel_token.cancel();
    }
}
