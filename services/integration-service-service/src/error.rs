use thiserror::Error;

#[derive(Error, Debug)]
pub enum IntegrationError {
    #[error("Database error: {0}")]
    Database(String),
    
    #[error("Validation error: {0}")]
    Validation(String),
    
    #[error("Not found: {0}")]
    NotFound(String),
    
    #[error("Provider error: {0}")]
    Provider(String),
    
    #[error("Authentication error: {0}")]
    Authentication(String),
    
    #[error("External API error: {0}")]
    ExternalApi(String),
    
    #[error("Crypto error: {0}")]
    Crypto(String),
    
    #[error("Scheduling error: {0}")]
    Scheduling(String),
    
    #[error("Internal error: {0}")]
    Internal(String),
}

impl From<sqlx::Error> for IntegrationError {
    fn from(err: sqlx::Error) -> Self {
        IntegrationError::Database(format!("{}", err))
    }
}

impl From<redis::RedisError> for IntegrationError {
    fn from(err: redis::RedisError) -> Self {
        IntegrationError::Internal(format!("Redis error: {}", err))
    }
}

impl From<reqwest::Error> for IntegrationError {
    fn from(err: reqwest::Error) -> Self {
        IntegrationError::ExternalApi(format!("{}", err))
    }
}

impl From<serde_json::Error> for IntegrationError {
    fn from(err: serde_json::Error) -> Self {
        IntegrationError::Internal(format!("JSON error: {}", err))
    }
}

impl From<std::io::Error> for IntegrationError {
    fn from(err: std::io::Error) -> Self {
        IntegrationError::Internal(format!("IO error: {}", err))
    }
}

// Conversion to mirage_common::Error for API handlers
impl From<IntegrationError> for mirage_common::Error {
    fn from(err: IntegrationError) -> Self {
        match err {
            IntegrationError::Database(msg) => mirage_common::Error::Database(msg),
            IntegrationError::Validation(msg) => mirage_common::Error::Validation(msg),
            IntegrationError::NotFound(msg) => mirage_common::Error::NotFound(msg),
            IntegrationError::Provider(msg) => mirage_common::Error::Internal(format!("Provider error: {}", msg)),
            IntegrationError::Authentication(msg) => mirage_common::Error::Unauthorized(msg),
            IntegrationError::ExternalApi(msg) => mirage_common::Error::ExternalApi(msg),
            IntegrationError::Crypto(msg) => mirage_common::Error::Internal(format!("Crypto error: {}", msg)),
            IntegrationError::Scheduling(msg) => mirage_common::Error::Internal(format!("Scheduling error: {}", msg)),
            IntegrationError::Internal(msg) => mirage_common::Error::Internal(msg),
        }
    }
}

pub type IntegrationResult<T> = Result<T, IntegrationError>;
