use std::fmt;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum VisualizationError {
    #[error("Not found: {0}")]
    NotFound(String),

    #[error("Invalid request: {0}")]
    InvalidRequest(String),

    #[error("Rendering error: {0}")]
    RenderingError(String),

    #[error("Data source error: {0}")]
    DataSourceError(String),

    #[error("Internal error: {0}")]
    Internal(String),
}

impl From<VisualizationError> for mirage_common::Error {
    fn from(err: VisualizationError) -> Self {
        match err {
            VisualizationError::NotFound(msg) => mirage_common::Error::NotFound(msg),
            VisualizationError::InvalidRequest(msg) => mirage_common::Error::Validation(msg),
            VisualizationError::RenderingError(msg) => mirage_common::Error::Internal(msg),
            VisualizationError::DataSourceError(msg) => mirage_common::Error::ExternalApi(msg),
            VisualizationError::Internal(msg) => mirage_common::Error::Internal(msg),
        }
    }
}
