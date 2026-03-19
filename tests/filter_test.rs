use packet_browser::filter::{validate_url, UrlError};

#[test]
fn test_blocked_protocols() {
    assert!(matches!(validate_url("file:///etc/passwd", &[]), Err(UrlError::BlockedProtocol(_))));
    assert!(matches!(validate_url("ftp://example.com", &[]), Err(UrlError::BlockedProtocol(_))));
    assert!(matches!(validate_url("gopher://example.com", &[]), Err(UrlError::BlockedProtocol(_))));
    assert!(matches!(validate_url("mailto:test@example.com", &[]), Err(UrlError::BlockedProtocol(_))));
}

#[test]
fn test_allowed_protocols() {
    assert!(validate_url("http://example.com", &[]).is_ok());
    assert!(validate_url("https://example.com", &[]).is_ok());
    assert!(validate_url("HTTP://EXAMPLE.COM", &[]).is_ok());
}
