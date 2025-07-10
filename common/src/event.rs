use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;

/// Represents an event in the system
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Event {
    pub id: Uuid,
    pub event_type: EventType,
    pub source: String,
    pub timestamp: DateTime<Utc>,
    pub data: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum EventType {
    #[serde(rename = "entity_created")]
    EntityCreated,

    #[serde(rename = "entity_updated")]
    EntityUpdated,

    #[serde(rename = "relationship_created")]
    RelationshipCreated,

    #[serde(rename = "scan_started")]
    ScanStarted,

    #[serde(rename = "scan_completed")]
    ScanCompleted,

    #[serde(rename = "module_executed")]
    ModuleExecuted,

    #[serde(rename = "user_action")]
    UserAction,

    #[serde(rename = "system_alert")]
    SystemAlert,

    #[serde(rename = "custom")]
    Custom(String),
}

impl Event {
    pub fn new(event_type: EventType, source: &str, data: Value) -> Self {
        Self {
            id: Uuid::new_v4(),
            event_type,
            source: source.to_string(),
            timestamp: Utc::now(),
            data,
        }
    }
}

/// Manages events throughout the system
#[derive(Debug, Default)]
pub struct EventHandler {
    events: Vec<Event>,
}

impl EventHandler {
    pub fn new() -> Self {
        EventHandler { events: Vec::new() }
    }

    pub fn add_event(&mut self, event: Event) {
        self.events.push(event);
    }

    pub fn get_events(&self, event_type: &EventType) -> Option<Vec<&Event>> {
        let filtered = self
            .events
            .iter()
            .filter(|e| &e.event_type == event_type)
            .collect::<Vec<_>>();

        if filtered.is_empty() {
            None
        } else {
            Some(filtered)
        }
    }

    pub fn list_all_events(&self) -> &[Event] {
        &self.events
    }
}
