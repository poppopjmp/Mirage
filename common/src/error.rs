use thiserror::Error;
use std::fmt;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Internal error: {0}")]
    Internal(String),

    #[error("Validation error: {0}")]
    Validation(String),

    #[error("Authentication error: {0}")]
    Authentication(String),

    #[error("Authorization error: {0}")]
    Authorization(String),

    #[error("Resource not found: {0}")]
    NotFound(String),

    #[error("Conflict error: {0}")]
    Conflict(String),

    #[error("Database error: {0}")]
    Database(String),

    #[error("External API error: {0}")]
    ExternalApi(String),

    #[error("Module execution error: {0}")]
    ModuleExecution(String),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self)
    }
}

pub type Result<T> = std::result::Result<T, Error>;
