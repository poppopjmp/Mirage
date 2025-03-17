pub mod event;
pub mod target;
pub mod helpers;

// Re-exports for convenience
pub use event::{Event, EventHandler};
pub use target::{Target, TargetManager};
pub use helpers::{is_valid_email, is_valid_url, is_valid_ip};
