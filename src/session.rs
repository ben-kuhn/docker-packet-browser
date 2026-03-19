use regex::Regex;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum SessionError {
    #[error("Invalid callsign format")]
    InvalidCallsign,
    #[error("Acknowledgment required")]
    AcknowledgmentRequired,
}

pub fn validate_callsign(callsign: &str) -> Result<String, SessionError> {
    // Strip SSID (e.g., W1ABC-1 -> W1ABC)
    let call = callsign.split('-').next().unwrap_or(callsign);

    // Callsign regex: 1-3 chars, digit, 0-3 chars, letter
    let re = Regex::new(r"^[a-zA-Z0-9]{1,3}[0-9][a-zA-Z0-9]{0,3}[a-zA-Z]$").unwrap();

    if re.is_match(call) {
        Ok(call.to_uppercase())
    } else {
        Err(SessionError::InvalidCallsign)
    }
}
