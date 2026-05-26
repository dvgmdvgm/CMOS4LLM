use thiserror::Error;

#[derive(Debug, Error)]
pub enum SubLmError {
    #[error("backend unavailable: {0}")]
    Unavailable(String),
    #[error("request failed: {0}")]
    RequestFailed(String),
    #[error("timeout after {0}s")]
    Timeout(u64),
    #[error("invalid response: {0}")]
    InvalidResponse(String),
    #[error("task queue full")]
    QueueFull,
    #[error("task cancelled")]
    Cancelled,
}
