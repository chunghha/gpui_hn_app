/// Extract the domain from a URL
pub fn extract_domain(url: &str) -> Option<String> {
    reqwest::Url::parse(url)
        .ok()
        .and_then(|u| u.host_str().map(|h| h.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_domain() {
        assert_eq!(
            extract_domain("https://github.com/zed-industries/zed"),
            Some("github.com".to_string())
        );
        assert_eq!(
            extract_domain("https://www.rust-lang.org/"),
            Some("www.rust-lang.org".to_string())
        );
        assert_eq!(
            extract_domain("https://news.ycombinator.com/item?id=123"),
            Some("news.ycombinator.com".to_string())
        );
        assert_eq!(extract_domain("not-a-url"), None);
    }
}
