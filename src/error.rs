use std::sync::mpsc::RecvError;

use tokio::sync::mpsc::error::SendError;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Actor's context must be set by this point")]
    ContextNotSet,

    #[error("Couldn't send the message")]
    SendError,

    #[error("Couldn't receive the message")]
    ReceiveError(#[from] RecvError),
}
