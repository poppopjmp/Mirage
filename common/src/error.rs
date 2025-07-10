use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Database error: {0}")]
    Database(String),
    
    #[error("Authentication error: {0}")]
    Auth(String),
    
    #[error("Validation error: {0}")]
    Validation(String),
    
    #[error("Configuration error: {0}")]
    Config(String),
    
    #[error("Network error: {0}")]
    Network(String),
    
    #[error("Serialization error: {0}")]
    Serialization(String),
    
    #[error("Internal error: {0}")]
    Internal(String),
    
    #[error("Not found: {0}")]
    NotFound(String),
    
    #[error("Unauthorized: {0}")]
    Unauthorized(String),
    
    #[error("Forbidden: {0}")]
    Forbidden(String),
    
    #[error("Authorization error: {0}")]
    Authorization(String),

    #[error("Resource not found: {0}")]
    ResourceNotFound(String),

    #[error("Conflict error: {0}")]
    Conflict(String),

    #[error("External API error: {0}")]
    ExternalApi(String),

    #[error("Module execution error: {0}")]
    ModuleExecution(String),
    
    #[error("Rate limit exceeded: {0}")]
    RateLimited(String),
    
    #[error("Timeout error: {0}")]
    Timeout(String),
}

impl From<serde_json::Error> for Error {
    fn from(err: serde_json::Error) -> Self {
        Error::Serialization(err.to_string())
    }
}

impl From<std::io::Error> for Error {
    fn from(err: std::io::Error) -> Self {
        Error::Internal(format!("IO error: {}", err))
    }
}

impl From<reqwest::Error> for Error {
    fn from(err: reqwest::Error) -> Self {
        if err.is_timeout() {
            Error::Timeout(err.to_string())
        } else if err.is_connect() {
            Error::Network(err.to_string())
        } else {
            Error::ExternalApi(err.to_string())
        }
    }
}

pub type Result<T> = std::result::Result<T, Error>;

// Helper function to map status codes to error types
pub fn map_status_error(status: reqwest::StatusCode, message: &str) -> Error {
    match status.as_u16() {
        400 => Error::Validation(message.to_string()),
        401 => Error::Auth(message.to_string()),
        403 => Error::Authorization(message.to_string()),
        404 => Error::NotFound(message.to_string()),
        409 => Error::Conflict(message.to_string()),
        429 => Error::RateLimited(message.to_string()),
        500..=599 => Error::ExternalApi(message.to_string()),
        _ => Error::Internal(format!("Unexpected status code: {}", status))
    }
}
