use thiserror::Error;

#[derive(Error, Debug)]
pub enum DomainError {
    #[error("Inference failure: {0}")]
    Inference(String),

    #[error("State inconsistency: {0}")]
    InternalState(String),

    #[error("Command timeout")]
    Timeout,
}

pub type Result<T> = std::result::Result<T, DomainError>;
