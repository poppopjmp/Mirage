use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct EmailBreachEvent {
    pub email: String,
    pub breach_source: String,
    pub timestamp: u64,
}

impl EmailBreachEvent {
    pub fn new(email: &str, breach_source: &str, timestamp: u64) -> Self {
        Self {
            email: email.to_string(),
            breach_source: breach_source.to_string(),
            timestamp,
        }
    }
}

pub struct EmailBreachRule {
    events: HashMap<String, Vec<EmailBreachEvent>>,
}

impl EmailBreachRule {
    pub fn new() -> Self {
        Self {
            events: HashMap::new(),
        }
    }

    pub fn add_event(&mut self, event: EmailBreachEvent) {
        self.events
            .entry(event.email.clone())
            .or_insert_with(Vec::new)
            .push(event);
    }

    pub fn get_events(&self, email: &str) -> Option<&Vec<EmailBreachEvent>> {
        self.events.get(email)
    }

    pub fn get_all_events(&self) -> &HashMap<String, Vec<EmailBreachEvent>> {
        &self.events
    }

    pub fn check_breached_emails(&self) -> Vec<&EmailBreachEvent> {
        let mut breached_emails = Vec::new();
        for events in self.events.values() {
            for event in events {
                breached_emails.push(event);
            }
        }
        breached_emails
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add_event() {
        let mut rule = EmailBreachRule::new();
        let event = EmailBreachEvent::new("test@example.com", "test_source", 1234567890);
        rule.add_event(event.clone());

        let events = rule.get_events("test@example.com").unwrap();
        assert_eq!(events.len(), 1);
        assert_eq!(events[0], event);
    }

    #[test]
    fn test_get_events() {
        let mut rule = EmailBreachRule::new();
        let event1 = EmailBreachEvent::new("test@example.com", "source1", 1234567890);
        let event2 = EmailBreachEvent::new("test@example.com", "source2", 1234567891);
        rule.add_event(event1.clone());
        rule.add_event(event2.clone());

        let events = rule.get_events("test@example.com").unwrap();
        assert_eq!(events.len(), 2);
        assert_eq!(events[0], event1);
        assert_eq!(events[1], event2);
    }

    #[test]
    fn test_get_all_events() {
        let mut rule = EmailBreachRule::new();
        let event1 = EmailBreachEvent::new("email1@example.com", "source1", 1234567890);
        let event2 = EmailBreachEvent::new("email2@example.com", "source2", 1234567891);
        rule.add_event(event1.clone());
        rule.add_event(event2.clone());

        let all_events = rule.get_all_events();
        assert_eq!(all_events.len(), 2);
        assert_eq!(all_events.get("email1@example.com").unwrap().len(), 1);
        assert_eq!(all_events.get("email2@example.com").unwrap().len(), 1);
    }

    #[test]
    fn test_check_breached_emails() {
        let mut rule = EmailBreachRule::new();
        let event1 = EmailBreachEvent::new("email1@example.com", "source1", 1234567890);
        let event2 = EmailBreachEvent::new("email2@example.com", "source2", 1234567891);
        rule.add_event(event1.clone());
        rule.add_event(event2.clone());

        let breached_emails = rule.check_breached_emails();
        assert_eq!(breached_emails.len(), 2);
        assert_eq!(breached_emails[0], &event1);
        assert_eq!(breached_emails[1], &event2);
    }
}
