use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct DnsZoneTransferEvent {
    pub domain: String,
    pub server: String,
    pub timestamp: u64,
}

impl DnsZoneTransferEvent {
    pub fn new(domain: &str, server: &str, timestamp: u64) -> Self {
        Self {
            domain: domain.to_string(),
            server: server.to_string(),
            timestamp,
        }
    }
}

pub struct DnsZoneTransferRule {
    events: HashMap<String, Vec<DnsZoneTransferEvent>>,
}

impl DnsZoneTransferRule {
    pub fn new() -> Self {
        Self {
            events: HashMap::new(),
        }
    }

    pub fn add_event(&mut self, event: DnsZoneTransferEvent) {
        self.events
            .entry(event.domain.clone())
            .or_insert_with(Vec::new)
            .push(event);
    }

    pub fn get_events(&self, domain: &str) -> Option<&Vec<DnsZoneTransferEvent>> {
        self.events.get(domain)
    }

    pub fn get_all_events(&self) -> &HashMap<String, Vec<DnsZoneTransferEvent>> {
        &self.events
    }

    pub fn check_zone_transfers(&self) -> Vec<&DnsZoneTransferEvent> {
        let mut zone_transfers = Vec::new();
        for events in self.events.values() {
            for event in events {
                zone_transfers.push(event);
            }
        }
        zone_transfers
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add_event() {
        let mut rule = DnsZoneTransferRule::new();
        let event = DnsZoneTransferEvent::new("example.com", "ns1.example.com", 1234567890);
        rule.add_event(event.clone());

        let events = rule.get_events("example.com").unwrap();
        assert_eq!(events.len(), 1);
        assert_eq!(events[0], event);
    }

    #[test]
    fn test_get_events() {
        let mut rule = DnsZoneTransferRule::new();
        let event1 = DnsZoneTransferEvent::new("example.com", "ns1.example.com", 1234567890);
        let event2 = DnsZoneTransferEvent::new("example.com", "ns2.example.com", 1234567891);
        rule.add_event(event1.clone());
        rule.add_event(event2.clone());

        let events = rule.get_events("example.com").unwrap();
        assert_eq!(events.len(), 2);
        assert_eq!(events[0], event1);
        assert_eq!(events[1], event2);
    }

    #[test]
    fn test_get_all_events() {
        let mut rule = DnsZoneTransferRule::new();
        let event1 = DnsZoneTransferEvent::new("example1.com", "ns1.example1.com", 1234567890);
        let event2 = DnsZoneTransferEvent::new("example2.com", "ns2.example2.com", 1234567891);
        rule.add_event(event1.clone());
        rule.add_event(event2.clone());

        let all_events = rule.get_all_events();
        assert_eq!(all_events.len(), 2);
        assert_eq!(all_events.get("example1.com").unwrap().len(), 1);
        assert_eq!(all_events.get("example2.com").unwrap().len(), 1);
    }

    #[test]
    fn test_check_zone_transfers() {
        let mut rule = DnsZoneTransferRule::new();
        let event1 = DnsZoneTransferEvent::new("example1.com", "ns1.example1.com", 1234567890);
        let event2 = DnsZoneTransferEvent::new("example2.com", "ns2.example2.com", 1234567891);
        rule.add_event(event1.clone());
        rule.add_event(event2.clone());

        let zone_transfers = rule.check_zone_transfers();
        assert_eq!(zone_transfers.len(), 2);
        assert_eq!(zone_transfers[0], &event1);
        assert_eq!(zone_transfers[1], &event2);
    }
}
