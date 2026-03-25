use crate::browser::InputField;
use regex::Regex;
use thiserror::Error;
use std::time::Instant;

#[derive(Error, Debug)]
pub enum SessionError {
    #[error("Invalid callsign format")]
    InvalidCallsign,
    #[error("Acknowledgment required")]
    AcknowledgmentRequired,
}

pub struct Session {
    pub callsign: String,
    pub acknowledged: bool,
    pub current_url: Option<String>,
    pub previous_url: Option<String>,
    pub links: Vec<(usize, String)>,
    pub inputs: Vec<InputField>,
    pub page_content: Vec<String>,
    pub lines_per_page: usize,
    pub full_page_mode: bool,
    pub last_activity: Instant,
}

impl Session {
    pub fn new(callsign: String) -> Self {
        Self {
            callsign,
            acknowledged: false,
            current_url: None,
            previous_url: None,
            links: Vec::new(),
            inputs: Vec::new(),
            page_content: Vec::new(),
            lines_per_page: 22, // VT52: 25 rows - 3 for footer (links/inputs/help)
            full_page_mode: false,
            last_activity: Instant::now(),
        }
    }

    pub fn acknowledge(&mut self) {
        self.acknowledged = true;
        self.touch();
    }

    pub fn touch(&mut self) {
        self.last_activity = Instant::now();
    }

    pub fn is_timed_out(&self, timeout_minutes: u64) -> bool {
        self.last_activity.elapsed().as_secs() > timeout_minutes * 60
    }
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
