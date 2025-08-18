#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Actor's context must be set by this point")]
    ContextNotSet
}
