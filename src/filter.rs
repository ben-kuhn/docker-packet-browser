use thiserror::Error;

#[derive(Error, Debug)]
pub enum UrlError {
    #[error("Blocked protocol: {0}")]
    BlockedProtocol(String),
    #[error("Blocked host: {0}")]
    BlockedHost(String),
    #[error("Invalid URL")]
    InvalidUrl,
}

const BLOCKED_PROTOCOLS: &[&str] = &["file:", "ftp:", "gopher:", "mailto:"];

pub fn validate_url(url: &str, blocked_ranges: &[String]) -> Result<(), UrlError> {
    let url_lower = url.to_lowercase();

    // Check blocked protocols
    for proto in BLOCKED_PROTOCOLS {
        if url_lower.starts_with(proto) {
            return Err(UrlError::BlockedProtocol(proto.to_string()));
        }
    }

    // Ensure http or https
    if !url_lower.starts_with("http://") && !url_lower.starts_with("https://") {
        return Err(UrlError::InvalidUrl);
    }

    Ok(())
}
