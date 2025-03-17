use std::collections::HashMap;
use serde::{Serialize, Deserialize};

/// Represents an event in the system with type, data, source, and timestamp.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Event {
    event_type: String,
    data: String,
    source: Option<String>,
    timestamp: u64,
}

impl Event {
    /// Create a new event with the given type, data, source, and timestamp.
    pub fn new(event_type: &str, data: &str, source: Option<&str>, timestamp: u64) -> Self {
        Event {
            event_type: event_type.to_string(),
            data: data.to_string(),
            source: source.map(|s| s.to_string()),
            timestamp,
        }
    }

    /// Get the event type.
    pub fn event_type(&self) -> &str {
        &self.event_type
    }

    /// Get the event data.
    pub fn data(&self) -> &str {
        &self.data
    }

    /// Get the event source, if available.
    pub fn source(&self) -> Option<&str> {
        self.source.as_deref()
    }

    /// Get the event timestamp.
    pub fn timestamp(&self) -> u64 {
        self.timestamp
    }
}

/// Handles event management, including adding and retrieving events.
#[derive(Debug)]
pub struct EventHandler {
    events: HashMap<String, Vec<Event>>,
}

impl EventHandler {
    /// Create a new event handler.
    pub fn new() -> Self {
        EventHandler {
            events: HashMap::new(),
        }
    }

    /// Add an event to the handler.
    pub fn add_event(&mut self, event: Event) {
        let events = self.events
            .entry(event.event_type().to_string())
            .or_insert_with(Vec::new);
        events.push(event);
    }

    /// Get all events of a specific type.
    pub fn get_events(&self, event_type: &str) -> Option<&Vec<Event>> {
        self.events.get(event_type)
    }

    /// List all event types.
    pub fn list_event_types(&self) -> Vec<&str> {
        self.events.keys().map(|k| k.as_str()).collect()
    }

    /// Count events of a specific type.
    pub fn count_events(&self, event_type: &str) -> usize {
        self.events.get(event_type).map_or(0, |events| events.len())
    }

    /// Count all events.
    pub fn total_events(&self) -> usize {
        self.events.values().map(|events| events.len()).sum()
    }
}

impl Default for EventHandler {
    fn default() -> Self {
        Self::new()
    }
}
