use thiserror::Error;

pub type CoreResult<T> = Result<T, CoreError>;

#[derive(Debug, Error)]
pub enum CoreError {
    #[error("core command channel is closed")]
    CommandChannelClosed,
    #[error("core event channel is closed")]
    EventChannelClosed,
    #[error("invalid command: {0}")]
    InvalidCommand(String),
    #[error("gateway error: {0}")]
    Gateway(String),
    #[error("store error: {0}")]
    Store(String),
}
