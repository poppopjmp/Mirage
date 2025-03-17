use thiserror::Error;

#[derive(Error, Debug)]
pub enum DiscoveryError {
    #[error("Redis error: {0}")]
    Redis(String),
    
    #[error("Validation error: {0}")]
    Validation(String),
    
    #[error("Not found: {0}")]
    NotFound(String),
    
    #[error("Service error: {0}")]
    Service(String),
    
    #[error("Internal error: {0}")]
    Internal(String),
}

impl From<redis::RedisError> for DiscoveryError {
    fn from(err: redis::RedisError) -> Self {
        DiscoveryError::Redis(format!("{}", err))
    }
}

impl From<reqwest::Error> for DiscoveryError {
    fn from(err: reqwest::Error) -> Self {
        DiscoveryError::Service(format!("{}", err))
    }
}

impl From<serde_json::Error> for DiscoveryError {
    fn from(err: serde_json::Error) -> Self {
        DiscoveryError::Internal(format!("JSON error: {}", err))
    }
}

// Conversion to mirage_common::Error for API handlers
impl From<DiscoveryError> for mirage_common::Error {
    fn from(err: DiscoveryError) -> Self {
        match err {
            DiscoveryError::Redis(msg) => mirage_common::Error::Database(msg),
            DiscoveryError::Validation(msg) => mirage_common::Error::Validation(msg),
            DiscoveryError::NotFound(msg) => mirage_common::Error::NotFound(msg),
            DiscoveryError::Service(msg) => mirage_common::Error::ExternalApi(msg),
            DiscoveryError::Internal(msg) => mirage_common::Error::Internal(msg),
        }
    }
}

pub type DiscoveryResult<T> = Result<T, DiscoveryError>;
