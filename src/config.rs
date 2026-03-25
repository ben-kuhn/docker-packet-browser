use std::env;

#[derive(Debug, Clone)]
pub struct Config {
    pub listen_port: u16,                    // default: 63004
    pub portal_url: String,                  // default: "http://matrix.ehvairport.com/~bpq/"
    pub idle_timeout_minutes: u64,           // default: 10
    pub blocked_ranges: Vec<String>,         // default: ["127.0.0.0/8", "10.0.0.0/8", "172.16.0.0/12", "192.168.0.0/16", "169.254.0.0/16"]
    pub blocklist_urls: Vec<String>,         // default: empty
    pub blocklist_refresh_hours: u64,        // default: 24
    pub blocklist_enabled: bool,             // default: true
    pub log_rotate_enabled: bool,            // default: true
    pub log_retain_days: u32,                // default: 30
    pub syslog_enabled: bool,                // default: false
    pub syslog_host: Option<String>,         // default: None
    pub syslog_port: u16,                    // default: 514
    pub lines_per_page: usize,               // default: 22 (VT52: 25 rows - footer)
    pub debug_mode: bool,                    // default: false
}

impl Config {
    pub fn from_env() -> Self {
        Config {
            listen_port: parse_env_u16("LISTEN_PORT", 63004),
            portal_url: env::var("PORTAL_URL")
                .unwrap_or_else(|_| "http://matrix.ehvairport.com/~bpq/".to_string()),
            idle_timeout_minutes: parse_env_u64("IDLE_TIMEOUT_MINUTES", 10),
            blocked_ranges: parse_env_vec(
                "BLOCKED_RANGES",
                vec![
                    "127.0.0.0/8".to_string(),
                    "10.0.0.0/8".to_string(),
                    "172.16.0.0/12".to_string(),
                    "192.168.0.0/16".to_string(),
                    "169.254.0.0/16".to_string(),
                ],
            ),
            blocklist_urls: parse_env_vec("BLOCKLIST_URLS", vec![]),
            blocklist_refresh_hours: parse_env_u64("BLOCKLIST_REFRESH_HOURS", 24),
            blocklist_enabled: parse_env_bool("BLOCKLIST_ENABLED", true),
            log_rotate_enabled: parse_env_bool("LOG_ROTATE_ENABLED", true),
            log_retain_days: parse_env_u32("LOG_RETAIN_DAYS", 30),
            syslog_enabled: parse_env_bool("SYSLOG_ENABLED", false),
            syslog_host: env::var("SYSLOG_HOST").ok(),
            syslog_port: parse_env_u16("SYSLOG_PORT", 514),
            lines_per_page: parse_env_usize("LINES_PER_PAGE", 22),
            debug_mode: parse_env_bool("DEBUG_MODE", false),
        }
    }
}

// Helper functions for parsing environment variables

fn parse_env_u16(key: &str, default: u16) -> u16 {
    env::var(key)
        .ok()
        .and_then(|v| v.parse::<u16>().ok())
        .unwrap_or(default)
}

fn parse_env_u32(key: &str, default: u32) -> u32 {
    env::var(key)
        .ok()
        .and_then(|v| v.parse::<u32>().ok())
        .unwrap_or(default)
}

fn parse_env_u64(key: &str, default: u64) -> u64 {
    env::var(key)
        .ok()
        .and_then(|v| v.parse::<u64>().ok())
        .unwrap_or(default)
}

fn parse_env_usize(key: &str, default: usize) -> usize {
    env::var(key)
        .ok()
        .and_then(|v| v.parse::<usize>().ok())
        .unwrap_or(default)
}

fn parse_env_bool(key: &str, default: bool) -> bool {
    env::var(key)
        .ok()
        .and_then(|v| match v.to_lowercase().as_str() {
            "true" | "1" | "yes" | "on" => Some(true),
            "false" | "0" | "no" | "off" => Some(false),
            _ => None,
        })
        .unwrap_or(default)
}

fn parse_env_vec(key: &str, default: Vec<String>) -> Vec<String> {
    env::var(key)
        .ok()
        .map(|v| {
            v.split(',')
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect()
        })
        .unwrap_or(default)
}
