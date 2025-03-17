use chrono::{DateTime, Utc, TimeZone};
use uuid::Uuid;
use std::net::IpAddr;

/// Utility functions for IP address operations
pub mod ip {
    use super::*;
    use std::net::{Ipv4Addr, Ipv6Addr};
    use std::str::FromStr;

    pub fn is_valid_ip(ip: &str) -> bool {
        IpAddr::from_str(ip).is_ok()
    }

    pub fn is_internal_ip(ip: &str) -> bool {
        if let Ok(ip) = IpAddr::from_str(ip) {
            match ip {
                IpAddr::V4(ipv4) => {
                    ipv4.is_private() || 
                    ipv4.is_loopback() || 
                    ipv4.is_link_local() || 
                    ipv4.is_documentation()
                }
                IpAddr::V6(ipv6) => {
                    ipv6.is_loopback() || 
                    ipv6.is_unspecified()
                }
            }
        } else {
            false
        }
    }
}

/// Utility functions for domain operations
pub mod domain {
    use regex::Regex;
    use lazy_static::lazy_static;

    lazy_static! {
        static ref DOMAIN_REGEX: Regex = Regex::new(
            r"^(?:[a-z0-9](?:[a-z0-9-]{0,61}[a-z0-9])?\.)+[a-z0-9][a-z0-9-]{0,61}[a-z0-9]$"
        ).unwrap();
    }

    pub fn is_valid_domain(domain: &str) -> bool {
        DOMAIN_REGEX.is_match(domain)
    }
}

/// Utility functions for email operations
pub mod email {
    use regex::Regex;
    use lazy_static::lazy_static;

    lazy_static! {
        static ref EMAIL_REGEX: Regex = Regex::new(
            r"^[a-zA-Z0-9.!#$%&'*+/=?^_`{|}~-]+@[a-zA-Z0-9](?:[a-zA-Z0-9-]{0,61}[a-zA-Z0-9])?(?:\.[a-zA-Z0-9](?:[a-zA-Z0-9-]{0,61}[a-zA-Z0-9])?)*$"
        ).unwrap();
    }

    pub fn is_valid_email(email: &str) -> bool {
        EMAIL_REGEX.is_match(email)
    }
}

/// Timing utilities
pub mod timing {
    use super::*;

    pub fn get_elapsed_time(start_time: &DateTime<Utc>) -> i64 {
        Utc::now().timestamp() - start_time.timestamp()
    }

    pub fn format_duration(seconds: i64) -> String {
        let hours = seconds / 3600;
        let minutes = (seconds % 3600) / 60;
        let seconds = seconds % 60;
        
        if hours > 0 {
            format!("{}h {}m {}s", hours, minutes, seconds)
        } else if minutes > 0 {
            format!("{}m {}s", minutes, seconds)
        } else {
            format!("{}s", seconds)
        }
    }
}

/// UUID utilities
pub mod id {
    use super::*;
    
    pub fn generate_id() -> Uuid {
        Uuid::new_v4()
    }
    
    pub fn parse_id(id: &str) -> Option<Uuid> {
        Uuid::parse_str(id).ok()
    }
    
    pub fn is_valid_uuid(uuid_str: &str) -> bool {
        Uuid::parse_str(uuid_str).is_ok()
    }
}

// String manipulation utilities
pub fn truncate_string(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else {
        format!("{}...", &s[0..max_len-3])
    }
}

// Date utilities
pub fn format_datetime(dt: &DateTime<Utc>) -> String {
    dt.format("%Y-%m-%d %H:%M:%S UTC").to_string()
}

pub fn parse_datetime(date_str: &str) -> Result<DateTime<Utc>, chrono::ParseError> {
    match DateTime::parse_from_rfc3339(date_str) {
        Ok(dt) => Ok(dt.with_timezone(&Utc)),
        Err(e) => Err(e),
    }
}

// Simple validation utilities
pub fn is_valid_email(email: &str) -> bool {
    if email.is_empty() || !email.contains('@') {
        return false;
    }
    
    let parts: Vec<&str> = email.split('@').collect();
    if parts.len() != 2 || parts[0].is_empty() || parts[1].is_empty() {
        return false;
    }
    
    let domain_parts: Vec<&str> = parts[1].split('.').collect();
    domain_parts.len() >= 2 && !domain_parts.iter().any(|part| part.is_empty())
}

// URL manipulation utilities
pub fn normalize_url(url: &str) -> String {
    let mut normalized = url.to_lowercase();
    
    // Ensure the URL starts with a scheme
    if !normalized.starts_with("http://") && !normalized.starts_with("https://") {
        normalized = format!("http://{}", normalized);
    }
    
    // Remove trailing slash if present
    if normalized.ends_with('/') {
        normalized.pop();
    }
    
    normalized
}
