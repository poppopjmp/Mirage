//! Functionality for handling various target types in the OSINT platform

use crate::{Error, Result};
use crate::models::TargetType;
use serde::{Serialize, Deserialize};
use std::str::FromStr;
use regex::Regex;
use once_cell::sync::Lazy;
use uuid::Uuid;
use std::collections::HashMap;

static DOMAIN_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"^([a-zA-Z0-9]([a-zA-Z0-9\-]{0,61}[a-zA-Z0-9])?\.)+[a-zA-Z]{2,}$").unwrap()
});

static EMAIL_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}$").unwrap()
});

static IPV4_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"^(?:(?:25[0-5]|2[0-4][0-9]|[01]?[0-9][0-9]?)\.){3}(?:25[0-5]|2[0-4][0-9]|[01]?[0-9][0-9]?)$").unwrap()
});

static URL_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"^(https?|ftp)://[^\s/$.?#].[^\s]*$").unwrap()
});

/// A lightweight representation of a target for parsing and validation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TargetInput {
    pub target_type: Option<TargetType>,
    pub value: String,
}

impl TargetInput {
    pub fn new(value: String, target_type: Option<TargetType>) -> Self {
        Self { target_type, value }
    }

    /// Validates the target and infers its type if not specified
    pub fn validate(&self) -> Result<TargetType> {
        if let Some(target_type) = &self.target_type {
            // If type is specified, validate against that type
            match target_type {
                TargetType::Domain => {
                    if !DOMAIN_REGEX.is_match(&self.value) {
                        return Err(Error::Validation(format!("Invalid domain: {}", self.value)));
                    }
                },
                TargetType::IpAddress => {
                    if !IPV4_REGEX.is_match(&self.value) {
                        return Err(Error::Validation(format!("Invalid IP address: {}", self.value)));
                    }
                },
                TargetType::Email => {
                    if !EMAIL_REGEX.is_match(&self.value) {
                        return Err(Error::Validation(format!("Invalid email: {}", self.value)));
                    }
                },
                TargetType::Url => {
                    if !URL_REGEX.is_match(&self.value) {
                        return Err(Error::Validation(format!("Invalid URL: {}", self.value)));
                    }
                },
                _ => (), // Other types don't have specific validation
            }
            Ok(target_type.clone())
        } else {
            // Try to infer the type
            infer_target_type(&self.value)
        }
    }
}

/// Attempts to infer the target type from the provided value
pub fn infer_target_type(value: &str) -> Result<TargetType> {
    if DOMAIN_REGEX.is_match(value) {
        return Ok(TargetType::Domain);
    }
    
    if EMAIL_REGEX.is_match(value) {
        return Ok(TargetType::Email);
    }
    
    if IPV4_REGEX.is_match(value) {
        return Ok(TargetType::IpAddress);
    }
    
    if URL_REGEX.is_match(value) {
        return Ok(TargetType::Url);
    }
    
    Err(Error::Validation(format!("Could not infer target type for: {}", value)))
}

/// Parse a string representation of target type
impl FromStr for TargetType {
    type Err = Error;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "domain" => Ok(TargetType::Domain),
            "ip_address" | "ip" => Ok(TargetType::IpAddress),
            "url" => Ok(TargetType::Url),
            "email" => Ok(TargetType::Email),
            "person" => Ok(TargetType::Person),
            "organization" | "org" => Ok(TargetType::Organization),
            "phone_number" | "phone" => Ok(TargetType::PhoneNumber),
            "social_media" | "social" => Ok(TargetType::SocialMedia),
            _ => Ok(TargetType::Custom(s.to_string())),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Target {
    pub id: Uuid,
    pub target_type: TargetType,
    pub value: String,
    pub metadata: HashMap<String, String>,
    pub tags: Vec<String>,
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
