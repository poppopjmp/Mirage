//! Common functionality for Mirage OSINT platform

pub mod auth;
pub mod config;
pub mod database;
pub mod error;
pub mod event;
pub mod health;
pub mod models;
pub mod target;
pub mod utils;

// Re-exports
pub use error::{Error, Result};
pub use models::*;
pub use utils::*;

/// Common result type used throughout the platform
pub type MirageResult<T> = std::result::Result<T, error::Error>;
