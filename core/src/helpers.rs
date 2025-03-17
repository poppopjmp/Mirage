pub fn is_valid_email(email: &str) -> bool {
    let email_regex = regex::Regex::new(r"^[\w\.-]+@[\w\.-]+\.\w+$").unwrap();
    email_regex.is_match(email)
}

pub fn is_valid_url(url: &str) -> bool {
    let url_regex = regex::Regex::new(r"^(http|https)://[\w\.-]+\.\w+").unwrap();
    url_regex.is_match(url)
}

pub fn is_valid_ip(ip: &str) -> bool {
    let ip_regex = regex::Regex::new(r"^(\d{1,3}\.){3}\d{1,3}$").unwrap();
    ip_regex.is_match(ip)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_valid_email() {
        assert!(is_valid_email("test@example.com"));
        assert!(!is_valid_email("invalid-email"));
    }

    #[test]
    fn test_is_valid_url() {
        assert!(is_valid_url("http://example.com"));
        assert!(is_valid_url("https://example.com"));
        assert!(!is_valid_url("invalid-url"));
    }

    #[test]
    fn test_is_valid_ip() {
        assert!(is_valid_ip("192.168.0.1"));
        assert!(!is_valid_ip("999.999.999.999"));
    }
}
