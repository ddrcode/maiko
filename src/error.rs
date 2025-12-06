use tokio::sync::mpsc::error::SendError;

use crate::{Envelope, Event};

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Actor's context must be set by this point")]
    ContextNotSet,

    #[error("Couldn't send the message: {0}")]
    SendError(String),

    #[error("Couldn't receive the message")]
    ReceiveError(#[from] std::sync::mpsc::RecvError),
}

impl<E: Event> From<SendError<Envelope<E>>> for Error {
    fn from(e: SendError<Envelope<E>>) -> Self {
        Error::SendError(e.to_string())
    }
}
