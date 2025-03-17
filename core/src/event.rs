use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct Event {
    pub event_type: String,
    pub data: String,
    pub source: Option<String>,
    pub timestamp: u64,
}

impl Event {
    pub fn new(event_type: &str, data: &str, source: Option<&str>, timestamp: u64) -> Self {
        Self {
            event_type: event_type.to_string(),
            data: data.to_string(),
            source: source.map(|s| s.to_string()),
            timestamp,
        }
    }
}

pub struct EventHandler {
    events: HashMap<String, Vec<Event>>,
}

impl EventHandler {
    pub fn new() -> Self {
        Self {
            events: HashMap::new(),
        }
    }

    pub fn add_event(&mut self, event: Event) {
        self.events
            .entry(event.event_type.clone())
            .or_insert_with(Vec::new)
            .push(event);
    }

    pub fn get_events(&self, event_type: &str) -> Option<&Vec<Event>> {
        self.events.get(event_type)
    }

    pub fn get_all_events(&self) -> &HashMap<String, Vec<Event>> {
        &self.events
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add_event() {
        let mut handler = EventHandler::new();
        let event = Event::new("test_type", "test_data", Some("test_source"), 1234567890);
        handler.add_event(event.clone());

        let events = handler.get_events("test_type").unwrap();
        assert_eq!(events.len(), 1);
        assert_eq!(events[0], event);
    }

    #[test]
    fn test_get_events() {
        let mut handler = EventHandler::new();
        let event1 = Event::new("test_type", "test_data1", Some("test_source1"), 1234567890);
        let event2 = Event::new("test_type", "test_data2", Some("test_source2"), 1234567891);
        handler.add_event(event1.clone());
        handler.add_event(event2.clone());

        let events = handler.get_events("test_type").unwrap();
        assert_eq!(events.len(), 2);
        assert_eq!(events[0], event1);
        assert_eq!(events[1], event2);
    }

    #[test]
    fn test_get_all_events() {
        let mut handler = EventHandler::new();
        let event1 = Event::new("type1", "data1", Some("source1"), 1234567890);
        let event2 = Event::new("type2", "data2", Some("source2"), 1234567891);
        handler.add_event(event1.clone());
        handler.add_event(event2.clone());

        let all_events = handler.get_all_events();
        assert_eq!(all_events.len(), 2);
        assert_eq!(all_events.get("type1").unwrap().len(), 1);
        assert_eq!(all_events.get("type2").unwrap().len(), 1);
    }
}
