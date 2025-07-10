//! Common functionality for Mirage OSINT platform

pub mod error;
pub mod models;
pub mod utils;
pub mod target;
pub mod config;
pub mod database;
pub mod auth;
pub mod health;
pub mod event;

// Re-exports
pub use error::{Error, Result};
pub use models::*;
pub use utils::*;

/// Common result type used throughout the platform
pub type MirageResult<T> = std::result::Result<T, error::Error>;
