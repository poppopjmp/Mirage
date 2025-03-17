use crate::core::event::Event;
use crate::correlations::{CorrelationRule, Alert, Severity};
use std::collections::{HashMap, HashSet};

/// Correlation rule for detecting compromised email addresses
pub struct EmailBreachRule;

impl EmailBreachRule {
    pub fn new() -> Self {
        EmailBreachRule
    }
}

impl CorrelationRule for EmailBreachRule {
    fn name(&self) -> &str {
        "Email Breach Detection"
    }
    
    fn description(&self) -> &str {
        "Detects email addresses that have appeared in known data breaches"
    }
    
    fn analyze(&self, events: &[Event]) -> Vec<Alert> {
        let mut alerts = Vec::new();
        let mut breach_count: HashMap<String, Vec<&Event>> = HashMap::new();
        let mut seen_emails = HashSet::new();
        
        // Find all email breach events and group by email
        for event in events {
            if event.event_type() == "EMAIL_BREACH" || 
               (event.event_type() == "BREACH_DATA" && event.data().contains("@")) {
                
                // In a real implementation, we would extract the email more carefully
                let email = event.data().to_string();
                
                if !seen_emails.contains(&email) {
                    seen_emails.insert(email.clone());
                    breach_count.entry(email).or_insert_with(Vec::new).push(event);
                }
            }
        }
        
        // Create alerts for each breached email
        for (email, events) in breach_count {
            // Determine severity based on number of breach events
            let severity = match events.len() {
                1 => Severity::Low,
                2..=3 => Severity::Medium,
                4..=10 => Severity::High,
                _ => Severity::Critical,
            };
            
            let alert = Alert::new(
                &format!("Email Breach: {}", email),
                &format!("Email address {} has been found in {} data breach(es)", email, events.len()),
                severity,
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
