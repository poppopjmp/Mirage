use serde::{Serialize, Deserialize};
use uuid::Uuid;
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Target {
    pub id: Uuid,
    pub target_type: TargetType,
    pub value: String,
    pub metadata: HashMap<String, String>,
    pub tags: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum TargetType {
    Domain,
    Subdomain,
    IPAddress,
    URL,
    Email,
    Person,
    Organization,
    Username,
    PhoneNumber,
    SocialMediaProfile,
    Other(String),
}

impl Target {
    pub fn new(target_type: TargetType, value: &str) -> Self {
        Self {
            id: Uuid::new_v4(),
            target_type,
            value: value.to_string(),
            metadata: HashMap::new(),
            tags: Vec::new(),
        }
    }
    
    pub fn add_metadata(&mut self, key: &str, value: &str) -> &mut Self {
        self.metadata.insert(key.to_string(), value.to_string());
        self
    }
    
    pub fn add_tag(&mut self, tag: &str) -> &mut Self {
        self.tags.push(tag.to_string());
        self
    }
}

/// Manages targets for scanning operations
#[derive(Debug, Default)]
pub struct TargetManager {
    targets: Vec<Target>,
}

impl TargetManager {
    pub fn new() -> Self {
        TargetManager {
            targets: Vec::new(),
        }
    }

    pub fn add_target(&mut self, target: Target) {
        self.targets.push(target);
    }

    pub fn get_target(&self, id: &Uuid) -> Option<&Target> {
        self.targets.iter().find(|t| t.id == *id)
    }

    pub fn get_targets_by_type(&self, target_type: &TargetType) -> Vec<&Target> {
        self.targets
            .iter()
            .filter(|t| t.target_type == *target_type)
            .collect()
    }

    pub fn list_all_targets(&self) -> &[Target] {
        &self.targets
    }
}
