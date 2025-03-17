use regex::Regex;

pub fn validate_url(url: &str) -> bool {
    let url_regex = Regex::new(r"^(http|https)://").unwrap();
    url_regex.is_match(url)
}

pub fn validate_email(email: &str) -> bool {
    let email_regex = Regex::new(r"^[\w\.-]+@[\w\.-]+\.\w+$").unwrap();
    email_regex.is_match(email)
}
