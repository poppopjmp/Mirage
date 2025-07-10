//! Mirage OSINT Platform Workspace
//!
//! This is the root workspace crate that provides common functionality
//! and serves as the main entry point for the Mirage platform.

pub use mirage_common as common;

/// Re-export commonly used types and functions
pub mod prelude {
    pub use mirage_common::*;
}
