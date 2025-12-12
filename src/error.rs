use std::sync::Arc;

use tokio::sync::mpsc::error::{SendError, TrySendError};

use crate::{Envelope, Event};

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Actor's context must be set by this point")]
    ContextNotSet,

    #[error("Couldn't send the message: {0}")]
    SendError(String),

    #[error("Actor task join error: {0}")]
    ActorJoinError(#[from] tokio::task::JoinError),

    #[error("Broker has already started.")]
    BrokerAlreadyStarted,

    #[error("The message channel has reached its capacity.")]
    ChannelIsFull,

    #[error("Subscriber with name '{0}' already exists.")]
    SubscriberAlreadyExists(Arc<str>),
}

impl<E: Event> From<SendError<Envelope<E>>> for Error {
    fn from(e: SendError<Envelope<E>>) -> Self {
        Error::SendError(e.to_string())
    }
}

impl<E: Event> From<TrySendError<Envelope<E>>> for Error {
    fn from(e: TrySendError<Envelope<E>>) -> Self {
        Error::SendError(e.to_string())
    }
}
