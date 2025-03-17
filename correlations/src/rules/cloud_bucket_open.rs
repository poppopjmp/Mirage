use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct CloudBucketOpenEvent {
    pub bucket_name: String,
    pub access_type: String,
    pub timestamp: u64,
}

impl CloudBucketOpenEvent {
    pub fn new(bucket_name: &str, access_type: &str, timestamp: u64) -> Self {
        Self {
            bucket_name: bucket_name.to_string(),
            access_type: access_type.to_string(),
            timestamp,
        }
    }
}

pub struct CloudBucketOpenRule {
    events: HashMap<String, Vec<CloudBucketOpenEvent>>,
}

impl CloudBucketOpenRule {
    pub fn new() -> Self {
        Self {
            events: HashMap::new(),
        }
    }

    pub fn add_event(&mut self, event: CloudBucketOpenEvent) {
        self.events
            .entry(event.bucket_name.clone())
            .or_insert_with(Vec::new)
            .push(event);
    }

    pub fn get_events(&self, bucket_name: &str) -> Option<&Vec<CloudBucketOpenEvent>> {
        self.events.get(bucket_name)
    }

    pub fn get_all_events(&self) -> &HashMap<String, Vec<CloudBucketOpenEvent>> {
        &self.events
    }

    pub fn check_open_buckets(&self) -> Vec<&CloudBucketOpenEvent> {
        let mut open_buckets = Vec::new();
        for events in self.events.values() {
            for event in events {
                if event.access_type == "public" {
                    open_buckets.push(event);
                }
            }
        }
        open_buckets
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add_event() {
        let mut rule = CloudBucketOpenRule::new();
        let event = CloudBucketOpenEvent::new("test_bucket", "public", 1234567890);
        rule.add_event(event.clone());

        let events = rule.get_events("test_bucket").unwrap();
        assert_eq!(events.len(), 1);
        assert_eq!(events[0], event);
    }

    #[test]
    fn test_get_events() {
        let mut rule = CloudBucketOpenRule::new();
        let event1 = CloudBucketOpenEvent::new("test_bucket", "public", 1234567890);
        let event2 = CloudBucketOpenEvent::new("test_bucket", "private", 1234567891);
        rule.add_event(event1.clone());
        rule.add_event(event2.clone());

        let events = rule.get_events("test_bucket").unwrap();
        assert_eq!(events.len(), 2);
        assert_eq!(events[0], event1);
        assert_eq!(events[1], event2);
    }

    #[test]
    fn test_get_all_events() {
        let mut rule = CloudBucketOpenRule::new();
        let event1 = CloudBucketOpenEvent::new("bucket1", "public", 1234567890);
        let event2 = CloudBucketOpenEvent::new("bucket2", "private", 1234567891);
        rule.add_event(event1.clone());
        rule.add_event(event2.clone());

        let all_events = rule.get_all_events();
        assert_eq!(all_events.len(), 2);
        assert_eq!(all_events.get("bucket1").unwrap().len(), 1);
        assert_eq!(all_events.get("bucket2").unwrap().len(), 1);
    }

    #[test]
    fn test_check_open_buckets() {
        let mut rule = CloudBucketOpenRule::new();
        let event1 = CloudBucketOpenEvent::new("bucket1", "public", 1234567890);
        let event2 = CloudBucketOpenEvent::new("bucket2", "private", 1234567891);
        rule.add_event(event1.clone());
        rule.add_event(event2.clone());

        let open_buckets = rule.check_open_buckets();
        assert_eq!(open_buckets.len(), 1);
        assert_eq!(open_buckets[0], &event1);
    }
}
