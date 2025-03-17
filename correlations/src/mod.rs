pub mod cloud_bucket_open;
pub mod dns_zone_transfer;
pub mod email_breach;

// Common types and traits for correlation rules
use crate::core::event::Event;
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Severity {
    Low,
    Medium,
    High,
    Critical,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Alert {
    title: String,
    description: String,
    severity: Severity,
    events: Vec<Event>,
    timestamp: u64,
}

impl Alert {
    pub fn new(title: &str, description: &str, severity: Severity, events: Vec<Event>, timestamp: u64) -> Self {
        Alert {
            title: title.to_string(),
            description: description.to_string(),
            severity,
            events,
            timestamp,
        }
    }
}

pub trait CorrelationRule {
    fn name(&self) -> &str;
    fn description(&self) -> &str;
    fn analyze(&self, events: &[Event]) -> Vec<Alert>;
}
