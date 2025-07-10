use thiserror::Error;

#[derive(Error, Debug)]
pub enum ScannerError {
    #[error("Database error: {0}")]
    Database(String),

    #[error("Validation error: {0}")]
    Validation(String),

    #[error("Not found: {0}")]
    NotFound(String),

    #[error("Integration error: {0}")]
    Integration(String),

    #[error("Queue error: {0}")]
    Queue(String),

    #[error("Internal error: {0}")]
    Internal(String),
}

impl From<sqlx::Error> for ScannerError {
    fn from(err: sqlx::Error) -> Self {
        ScannerError::Database(format!("{}", err))
    }
}

impl From<redis::RedisError> for ScannerError {
    fn from(err: redis::RedisError) -> Self {
        ScannerError::Queue(format!("{}", err))
    }
}

impl From<reqwest::Error> for ScannerError {
    fn from(err: reqwest::Error) -> Self {
        ScannerError::Integration(format!("{}", err))
    }
}

impl From<serde_json::Error> for ScannerError {
    fn from(err: serde_json::Error) -> Self {
        ScannerError::Internal(format!("JSON error: {}", err))
    }
}

// Conversion to mirage_common::Error for API handlers
impl From<ScannerError> for mirage_common::Error {
    fn from(err: ScannerError) -> Self {
        match err {
            ScannerError::Database(msg) => mirage_common::Error::Database(msg),
            ScannerError::Validation(msg) => mirage_common::Error::Validation(msg),
            ScannerError::NotFound(msg) => mirage_common::Error::NotFound(msg),
            ScannerError::Integration(msg) => mirage_common::Error::ExternalApi(msg),
            ScannerError::Queue(msg) => {
                mirage_common::Error::Internal(format!("Queue error: {}", msg))
            }
            ScannerError::Internal(msg) => mirage_common::Error::Internal(msg),
        }
    }
}

pub type ScannerResult<T> = Result<T, ScannerError>;
