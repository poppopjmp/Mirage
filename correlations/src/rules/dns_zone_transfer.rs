use crate::core::event::Event;
use crate::correlations::{CorrelationRule, Alert, Severity};

/// Correlation rule for detecting DNS zone transfers
pub struct DnsZoneTransferRule;

impl DnsZoneTransferRule {
    pub fn new() -> Self {
        DnsZoneTransferRule
    }
    
    fn is_zone_transfer_event(&self, event: &Event) -> bool {
        event.event_type() == "DNS_ZONE_TRANSFER" || 
        (event.event_type() == "DNS_RECORD" && event.data().contains("AXFR"))
    }
}

impl CorrelationRule for DnsZoneTransferRule {
    fn name(&self) -> &str {
        "DNS Zone Transfer"
    }
    
    fn description(&self) -> &str {
        "Detects unauthorized DNS zone transfers which can expose internal network information"
    }
    
    fn analyze(&self, events: &[Event]) -> Vec<Alert> {
        let mut alerts = Vec::new();
        let mut zone_transfer_events = Vec::new();
        
        // Group events by domain
        let mut domain_events: std::collections::HashMap<String, Vec<&Event>> = std::collections::HashMap::new();
        
        for event in events {
            if self.is_zone_transfer_event(event) {
                zone_transfer_events.push(event.clone());
                
                // Extract domain from event data
                // In a real implementation, we would parse the event data properly
                let domain = event.source().unwrap_or("unknown").to_string();
                
                let domain_events_entry = domain_events.entry(domain).or_insert_with(Vec::new);
                domain_events_entry.push(event);
            }
        }
        
        // Create an alert for each domain with zone transfer events
        for (domain, events) in domain_events {
            let alert = Alert::new(
                &format!("DNS Zone Transfer: {}", domain),
                &format!("DNS server is allowing zone transfers for domain {}, which can expose internal network information", domain),
                Severity::Medium,
                events.into_iter().cloned().collect(),
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_secs()
            );
            alerts.push(alert);
        }
        
        alerts
    }
}
