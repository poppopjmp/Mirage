use crate::core::event::Event;
use crate::correlations::{CorrelationRule, Alert, Severity};
use regex::Regex;
use lazy_static::lazy_static;

lazy_static! {
    static ref S3_BUCKET_REGEX: Regex = Regex::new(r"https?://([a-zA-Z0-9-]+)\.s3\.amazonaws\.com").unwrap();
    static ref GCS_BUCKET_REGEX: Regex = Regex::new(r"https?://storage\.googleapis\.com/([a-zA-Z0-9-]+)").unwrap();
    static ref AZURE_BLOB_REGEX: Regex = Regex::new(r"https?://([a-zA-Z0-9-]+)\.blob\.core\.windows\.net").unwrap();
}

/// Correlation rule for detecting open cloud storage buckets
pub struct CloudBucketOpenRule;

impl CloudBucketOpenRule {
    pub fn new() -> Self {
        CloudBucketOpenRule
    }
    
    fn is_open_bucket_url(&self, url: &str) -> bool {
        // In a real implementation, this would check if the bucket is publicly accessible
        // For now, we'll just check if it matches a cloud bucket pattern
        S3_BUCKET_REGEX.is_match(url) || 
        GCS_BUCKET_REGEX.is_match(url) || 
        AZURE_BLOB_REGEX.is_match(url)
    }
    
    fn extract_bucket_name(&self, url: &str) -> Option<String> {
        if let Some(captures) = S3_BUCKET_REGEX.captures(url) {
            return captures.get(1).map(|m| m.as_str().to_string());
        }
        
        if let Some(captures) = GCS_BUCKET_REGEX.captures(url) {
            return captures.get(1).map(|m| m.as_str().to_string());
        }
        
        if let Some(captures) = AZURE_BLOB_REGEX.captures(url) {
            return captures.get(1).map(|m| m.as_str().to_string());
        }
        
        None
    }
}

impl CorrelationRule for CloudBucketOpenRule {
    fn name(&self) -> &str {
        "Cloud Bucket Open"
    }
    
    fn description(&self) -> &str {
        "Detects publicly accessible cloud storage buckets"
    }
    
    fn analyze(&self, events: &[Event]) -> Vec<Alert> {
        let mut alerts = Vec::new();
        let mut seen_buckets = std::collections::HashSet::new();
        
        for event in events {
            if event.event_type() == "URL_FOUND" || event.event_type() == "CLOUD_STORAGE" {
                let url = event.data();
                
                if self.is_open_bucket_url(url) {
                    if let Some(bucket_name) = self.extract_bucket_name(url) {
                        if seen_buckets.insert(bucket_name.clone()) {
                            let alert = Alert::new(
                                &format!("Open Cloud Bucket: {}", bucket_name),
                                &format!("Potentially publicly accessible cloud storage bucket found: {}", url),
                                Severity::High,
                                vec![event.clone()],
                                std::time::SystemTime::now()
                                    .duration_since(std::time::UNIX_EPOCH)
                                    .unwrap()
                                    .as_secs()
                            );
                            alerts.push(alert);
                        }
                    }
                }
            }
        }
        
        alerts
    }
}
