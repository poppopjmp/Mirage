use regex::Regex;
use std::net::IpAddr;
use url::Url;
use lazy_static::lazy_static;

lazy_static! {
    static ref EMAIL_REGEX: Regex = Regex::new(r"^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}$").unwrap();
}

/// Validates if the given string is a valid email address.
pub fn is_valid_email(email: &str) -> bool {
    EMAIL_REGEX.is_match(email)
}

/// Validates if the given string is a valid URL.
pub fn is_valid_url(url_str: &str) -> bool {
    match Url::parse(url_str) {
        Ok(url) => url.scheme() == "http" || url.scheme() == "https",
        Err(_) => false,
    }
}

/// Validates if the given string is a valid IP address (IPv4 or IPv6).
pub fn is_valid_ip(ip_str: &str) -> bool {
    ip_str.parse::<IpAddr>().is_ok()
}

/// Extracts domain name from a URL.
pub fn extract_domain(url_str: &str) -> Option<String> {
    match Url::parse(url_str) {
        Ok(url) => url.host_str().map(String::from),
        Err(_) => None,
    }
}

/// Normalizes a URL by removing trailing slashes and fragments.
pub fn normalize_url(url_str: &str) -> Option<String> {
    match Url::parse(url_str) {
        Ok(mut url) => {
            url.set_fragment(None);
            Some(url.to_string().trim_end_matches('/').to_string())
        },
        Err(_) => None,
    }
}

/// Checks if a port is within valid range.
pub fn is_valid_port(port: u16) -> bool {
    port > 0 && port < 65536
}
