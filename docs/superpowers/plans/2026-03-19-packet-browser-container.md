# Packet Browser Container Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Build a secure, containerized packet radio web browser in Rust with headless Chromium.

**Architecture:** Rust binary handles TCP connections, session management, and user commands. Headless Chromium renders pages. Text extracted from DOM and displayed with numbered links. NixOS container for deployment.

**Tech Stack:** Rust, tokio (async), headless_chrome crate, NixOS, Docker

**Spec:** `docs/superpowers/specs/2026-03-19-docker-container-design.md`

---

## File Structure

```
packet-browser/
├── Cargo.toml                 - Rust package manifest
├── src/
│   ├── main.rs                - Entry point, TCP listener setup
│   ├── lib.rs                 - Library exports for testing
│   ├── config.rs              - Environment variable parsing
│   ├── session.rs             - Session state, callsign handling
│   ├── browser.rs             - Chromium launch and page fetching
│   ├── render.rs              - DOM text extraction, link parsing
│   ├── commands.rs            - User command parsing and dispatch
│   ├── display.rs             - Pagination and output formatting
│   ├── logger.rs              - JSON structured logging
│   └── filter.rs              - URL validation, SSRF prevention
├── tests/
│   ├── config_test.rs         - Config parsing tests
│   ├── session_test.rs        - Session/callsign tests
│   ├── render_test.rs         - Text extraction tests
│   ├── commands_test.rs       - Command parsing tests
│   ├── display_test.rs        - Pagination tests
│   ├── filter_test.rs         - URL filtering tests
│   └── integration_test.rs    - End-to-end tests
├── flake.nix                  - Nix flake for container build
├── docker-compose.yml         - Deployment configuration
├── .github/workflows/
│   └── build.yml              - CI/CD pipeline
└── README.md                  - Updated documentation
```

---

## Phase 1: Project Setup & Configuration

### Task 1: Initialize Rust Project

**Files:**
- Create: `Cargo.toml`
- Create: `src/main.rs`
- Create: `src/lib.rs`

- [ ] **Step 1: Create Cargo.toml**

```toml
[package]
name = "packet-browser"
version = "0.1.0"
edition = "2021"
description = "Secure packet radio web browser for BPQ"
license = "GPL-3.0"

[dependencies]
tokio = { version = "1", features = ["full"] }
headless_chrome = "1"
serde = { version = "1", features = ["derive"] }
serde_json = "1"
chrono = { version = "0.4", features = ["serde"] }
regex = "1"
thiserror = "1"
tracing = "0.1"
tracing-subscriber = "0.3"

[dev-dependencies]
tempfile = "3"
```

- [ ] **Step 2: Create minimal main.rs**

```rust
fn main() {
    println!("packet-browser starting...");
}
```

- [ ] **Step 3: Create lib.rs**

```rust
pub mod config;
```

- [ ] **Step 4: Create empty config module**

Create `src/config.rs`:
```rust
// Configuration module - to be implemented
```

- [ ] **Step 5: Verify project builds**

Run: `cargo build`
Expected: Build succeeds

- [ ] **Step 6: Commit**

```bash
git add Cargo.toml src/
git commit -m "feat: initialize Rust project structure"
```

---

### Task 2: Configuration Module

**Files:**
- Create: `src/config.rs`
- Create: `tests/config_test.rs`

- [ ] **Step 1: Write failing test for config defaults**

Create `tests/config_test.rs`:
```rust
use packet_browser::config::Config;

#[test]
fn test_config_defaults() {
    // Clear any env vars that might interfere
    std::env::remove_var("LISTEN_PORT");
    std::env::remove_var("PORTAL_URL");

    let config = Config::from_env();

    assert_eq!(config.listen_port, 63004);
    assert_eq!(config.portal_url, "http://matrix.ehvairport.com/~bpq/");
    assert_eq!(config.idle_timeout_minutes, 10);
    assert_eq!(config.lines_per_page, 15);
    assert!(!config.debug_mode);
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test test_config_defaults`
Expected: FAIL - module not found or struct not defined

- [ ] **Step 3: Implement Config struct with defaults**

Update `src/config.rs`:
```rust
use std::env;

#[derive(Debug, Clone)]
pub struct Config {
    pub listen_port: u16,
    pub portal_url: String,
    pub idle_timeout_minutes: u64,
    pub dns_servers: Vec<String>,
    pub blocked_ranges: Vec<String>,
    pub blocklist_urls: Vec<String>,
    pub blocklist_refresh_hours: u64,
    pub blocklist_enabled: bool,
    pub log_rotate_enabled: bool,
    pub log_retain_days: u32,
    pub syslog_enabled: bool,
    pub syslog_host: Option<String>,
    pub syslog_port: u16,
    pub lines_per_page: usize,
    pub debug_mode: bool,
}

impl Config {
    pub fn from_env() -> Self {
        Self {
            listen_port: env::var("LISTEN_PORT")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(63004),
            portal_url: env::var("PORTAL_URL")
                .unwrap_or_else(|_| "http://matrix.ehvairport.com/~bpq/".to_string()),
            idle_timeout_minutes: env::var("IDLE_TIMEOUT_MINUTES")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(10),
            dns_servers: env::var("DNS_SERVERS")
                .unwrap_or_else(|_| "208.67.222.123,208.67.220.123".to_string())
                .split(',')
                .map(|s| s.trim().to_string())
                .collect(),
            blocked_ranges: env::var("BLOCKED_RANGES")
                .unwrap_or_else(|_| "127.0.0.0/8,10.0.0.0/8,172.16.0.0/12,192.168.0.0/16,169.254.0.0/16".to_string())
                .split(',')
                .map(|s| s.trim().to_string())
                .collect(),
            blocklist_urls: env::var("BLOCKLIST_URLS")
                .unwrap_or_else(|_| "".to_string())
                .split(',')
                .filter(|s| !s.is_empty())
                .map(|s| s.trim().to_string())
                .collect(),
            blocklist_refresh_hours: env::var("BLOCKLIST_REFRESH_HOURS")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(24),
            blocklist_enabled: env::var("BLOCKLIST_ENABLED")
                .map(|v| v.to_lowercase() != "false")
                .unwrap_or(true),
            log_rotate_enabled: env::var("LOG_ROTATE_ENABLED")
                .map(|v| v.to_lowercase() != "false")
                .unwrap_or(true),
            log_retain_days: env::var("LOG_RETAIN_DAYS")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(30),
            syslog_enabled: env::var("SYSLOG_ENABLED")
                .map(|v| v.to_lowercase() == "true")
                .unwrap_or(false),
            syslog_host: env::var("SYSLOG_HOST").ok(),
            syslog_port: env::var("SYSLOG_PORT")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(514),
            lines_per_page: env::var("LINES_PER_PAGE")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(15),
            debug_mode: env::var("DEBUG_MODE")
                .map(|v| v.to_lowercase() == "true")
                .unwrap_or(false),
        }
    }
}
```

- [ ] **Step 4: Update lib.rs to export config**

Update `src/lib.rs`:
```rust
pub mod config;
```

- [ ] **Step 5: Run test to verify it passes**

Run: `cargo test test_config_defaults`
Expected: PASS

- [ ] **Step 6: Write test for env var override**

Add to `tests/config_test.rs`:
```rust
#[test]
fn test_config_env_override() {
    std::env::set_var("LISTEN_PORT", "12345");
    std::env::set_var("DEBUG_MODE", "true");

    let config = Config::from_env();

    assert_eq!(config.listen_port, 12345);
    assert!(config.debug_mode);

    // Clean up
    std::env::remove_var("LISTEN_PORT");
    std::env::remove_var("DEBUG_MODE");
}
```

- [ ] **Step 7: Run test to verify it passes**

Run: `cargo test test_config_env_override`
Expected: PASS

- [ ] **Step 8: Commit**

```bash
git add src/config.rs src/lib.rs tests/config_test.rs
git commit -m "feat: add configuration module with env var support"
```

---

## Phase 2: Session Management

### Task 3: Callsign Validation

**Files:**
- Create: `src/session.rs`
- Create: `tests/session_test.rs`

- [ ] **Step 1: Write failing test for valid callsign**

Create `tests/session_test.rs`:
```rust
use packet_browser::session::validate_callsign;

#[test]
fn test_valid_callsigns() {
    assert!(validate_callsign("W1ABC").is_ok());
    assert!(validate_callsign("VE3XYZ").is_ok());
    assert!(validate_callsign("KU0HN").is_ok());
    assert!(validate_callsign("G4ABC").is_ok());
    assert!(validate_callsign("JA1ABC").is_ok());
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test test_valid_callsigns`
Expected: FAIL - function not found

- [ ] **Step 3: Implement callsign validation**

Create `src/session.rs`:
```rust
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
```

- [ ] **Step 4: Update lib.rs**

```rust
pub mod config;
pub mod session;
```

- [ ] **Step 5: Run test to verify it passes**

Run: `cargo test test_valid_callsigns`
Expected: PASS

- [ ] **Step 6: Write test for invalid callsigns**

Add to `tests/session_test.rs`:
```rust
#[test]
fn test_invalid_callsigns() {
    assert!(validate_callsign("").is_err());
    assert!(validate_callsign("123").is_err());
    assert!(validate_callsign("ABCDEF").is_err());
    assert!(validate_callsign("W").is_err());
}
```

- [ ] **Step 7: Run test to verify it passes**

Run: `cargo test test_invalid_callsigns`
Expected: PASS

- [ ] **Step 8: Write test for SSID stripping**

Add to `tests/session_test.rs`:
```rust
#[test]
fn test_callsign_ssid_stripped() {
    assert_eq!(validate_callsign("W1ABC-1").unwrap(), "W1ABC");
    assert_eq!(validate_callsign("KU0HN-15").unwrap(), "KU0HN");
}
```

- [ ] **Step 9: Run test to verify it passes**

Run: `cargo test test_callsign_ssid_stripped`
Expected: PASS

- [ ] **Step 10: Commit**

```bash
git add src/session.rs src/lib.rs tests/session_test.rs
git commit -m "feat: add callsign validation with SSID stripping"
```

---

### Task 4: Session State

**Files:**
- Modify: `src/session.rs`
- Modify: `tests/session_test.rs`

- [ ] **Step 1: Write failing test for session state**

Add to `tests/session_test.rs`:
```rust
use packet_browser::session::Session;
use std::time::Instant;

#[test]
fn test_session_creation() {
    let session = Session::new("W1ABC".to_string());

    assert_eq!(session.callsign, "W1ABC");
    assert!(!session.acknowledged);
    assert!(session.current_url.is_none());
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test test_session_creation`
Expected: FAIL - Session struct not found

- [ ] **Step 3: Implement Session struct**

Add to `src/session.rs`:
```rust
use std::time::Instant;

pub struct Session {
    pub callsign: String,
    pub acknowledged: bool,
    pub current_url: Option<String>,
    pub previous_url: Option<String>,
    pub links: Vec<(usize, String)>,
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
            page_content: Vec::new(),
            lines_per_page: 15,
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
```

- [ ] **Step 4: Run test to verify it passes**

Run: `cargo test test_session_creation`
Expected: PASS

- [ ] **Step 5: Write test for timeout**

Add to `tests/session_test.rs`:
```rust
#[test]
fn test_session_timeout() {
    let session = Session::new("W1ABC".to_string());

    // Should not be timed out immediately
    assert!(!session.is_timed_out(10));
}
```

- [ ] **Step 6: Run test to verify it passes**

Run: `cargo test test_session_timeout`
Expected: PASS

- [ ] **Step 7: Commit**

```bash
git add src/session.rs tests/session_test.rs
git commit -m "feat: add session state management with timeout"
```

---

## Phase 3: URL Filtering

### Task 5: Protocol Filtering

**Files:**
- Create: `src/filter.rs`
- Create: `tests/filter_test.rs`

- [ ] **Step 1: Write failing test for blocked protocols**

Create `tests/filter_test.rs`:
```rust
use packet_browser::filter::{validate_url, UrlError};

#[test]
fn test_blocked_protocols() {
    assert!(matches!(validate_url("file:///etc/passwd", &[]), Err(UrlError::BlockedProtocol(_))));
    assert!(matches!(validate_url("ftp://example.com", &[]), Err(UrlError::BlockedProtocol(_))));
    assert!(matches!(validate_url("gopher://example.com", &[]), Err(UrlError::BlockedProtocol(_))));
    assert!(matches!(validate_url("mailto:test@example.com", &[]), Err(UrlError::BlockedProtocol(_))));
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test test_blocked_protocols`
Expected: FAIL - module not found

- [ ] **Step 3: Implement protocol filtering**

Create `src/filter.rs`:
```rust
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
```

- [ ] **Step 4: Update lib.rs**

```rust
pub mod config;
pub mod session;
pub mod filter;
```

- [ ] **Step 5: Run test to verify it passes**

Run: `cargo test test_blocked_protocols`
Expected: PASS

- [ ] **Step 6: Write test for allowed protocols**

Add to `tests/filter_test.rs`:
```rust
#[test]
fn test_allowed_protocols() {
    assert!(validate_url("http://example.com", &[]).is_ok());
    assert!(validate_url("https://example.com", &[]).is_ok());
    assert!(validate_url("HTTP://EXAMPLE.COM", &[]).is_ok());
}
```

- [ ] **Step 7: Run test to verify it passes**

Run: `cargo test test_allowed_protocols`
Expected: PASS

- [ ] **Step 8: Commit**

```bash
git add src/filter.rs src/lib.rs tests/filter_test.rs
git commit -m "feat: add URL protocol filtering"
```

---

### Task 6: SSRF Prevention

**Files:**
- Modify: `src/filter.rs`
- Modify: `tests/filter_test.rs`

- [ ] **Step 1: Write failing test for localhost blocking**

Add to `tests/filter_test.rs`:
```rust
#[test]
fn test_blocked_localhost() {
    let blocked = vec!["127.0.0.0/8".to_string()];

    assert!(matches!(
        validate_url("http://127.0.0.1/admin", &blocked),
        Err(UrlError::BlockedHost(_))
    ));
    assert!(matches!(
        validate_url("http://localhost/admin", &blocked),
        Err(UrlError::BlockedHost(_))
    ));
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test test_blocked_localhost`
Expected: FAIL - localhost not blocked yet

- [ ] **Step 3: Implement SSRF prevention**

Update `src/filter.rs`:
```rust
use std::net::{IpAddr, Ipv4Addr};
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
const BLOCKED_HOSTNAMES: &[&str] = &["localhost"];

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

    // Extract host from URL
    let host = extract_host(&url_lower)?;

    // Check blocked hostnames
    for blocked in BLOCKED_HOSTNAMES {
        if host == *blocked {
            return Err(UrlError::BlockedHost(host));
        }
    }

    // Check if host is an IP address in blocked ranges
    if let Ok(ip) = host.parse::<IpAddr>() {
        if is_ip_in_blocked_ranges(&ip, blocked_ranges) {
            return Err(UrlError::BlockedHost(host));
        }
    }

    Ok(())
}

fn extract_host(url: &str) -> Result<String, UrlError> {
    let without_proto = url
        .strip_prefix("https://")
        .or_else(|| url.strip_prefix("http://"))
        .ok_or(UrlError::InvalidUrl)?;

    let host_part = without_proto.split('/').next().unwrap_or("");
    let host = host_part.split(':').next().unwrap_or("");

    if host.is_empty() {
        return Err(UrlError::InvalidUrl);
    }

    Ok(host.to_string())
}

fn is_ip_in_blocked_ranges(ip: &IpAddr, blocked_ranges: &[String]) -> bool {
    match ip {
        IpAddr::V4(ipv4) => {
            for range in blocked_ranges {
                if let Some((network, prefix)) = range.split_once('/') {
                    if let (Ok(net_ip), Ok(prefix_len)) = (network.parse::<Ipv4Addr>(), prefix.parse::<u8>()) {
                        if is_ipv4_in_cidr(ipv4, &net_ip, prefix_len) {
                            return true;
                        }
                    }
                }
            }
            false
        }
        IpAddr::V6(_) => {
            // Block IPv6 loopback
            ip.is_loopback()
        }
    }
}

fn is_ipv4_in_cidr(ip: &Ipv4Addr, network: &Ipv4Addr, prefix_len: u8) -> bool {
    let ip_bits = u32::from_be_bytes(ip.octets());
    let net_bits = u32::from_be_bytes(network.octets());
    let mask = if prefix_len == 0 { 0 } else { !0u32 << (32 - prefix_len) };

    (ip_bits & mask) == (net_bits & mask)
}
```

- [ ] **Step 4: Run test to verify it passes**

Run: `cargo test test_blocked_localhost`
Expected: PASS

- [ ] **Step 5: Write test for private ranges**

Add to `tests/filter_test.rs`:
```rust
#[test]
fn test_blocked_private_ranges() {
    let blocked = vec![
        "10.0.0.0/8".to_string(),
        "172.16.0.0/12".to_string(),
        "192.168.0.0/16".to_string(),
    ];

    assert!(validate_url("http://10.0.0.1/", &blocked).is_err());
    assert!(validate_url("http://172.16.0.1/", &blocked).is_err());
    assert!(validate_url("http://192.168.1.1/", &blocked).is_err());

    // Public IPs should be allowed
    assert!(validate_url("http://8.8.8.8/", &blocked).is_ok());
}
```

- [ ] **Step 6: Run test to verify it passes**

Run: `cargo test test_blocked_private_ranges`
Expected: PASS

- [ ] **Step 7: Commit**

```bash
git add src/filter.rs tests/filter_test.rs
git commit -m "feat: add SSRF prevention for private IP ranges"
```

---

## Phase 4: Command Parsing

### Task 7: Command Parser

**Files:**
- Create: `src/commands.rs`
- Create: `tests/commands_test.rs`

- [ ] **Step 1: Write failing test for quit command**

Create `tests/commands_test.rs`:
```rust
use packet_browser::commands::{parse_command, Command};

#[test]
fn test_quit_commands() {
    assert!(matches!(parse_command("q"), Command::Quit));
    assert!(matches!(parse_command("Q"), Command::Quit));
    assert!(matches!(parse_command("b"), Command::Quit));
    assert!(matches!(parse_command("0"), Command::Quit));
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test test_quit_commands`
Expected: FAIL - module not found

- [ ] **Step 3: Implement command parser**

Create `src/commands.rs`:
```rust
#[derive(Debug, PartialEq)]
pub enum Command {
    Quit,
    Help,
    Menu,
    Back,
    List,
    Redisplay,
    FullPageToggle,
    SetPageSize(usize),
    LoadLink(usize),
    NewUrl(String),
    Search(String),
    Unknown(String),
}

pub fn parse_command(input: &str) -> Command {
    let input = input.trim().to_lowercase();

    // Quit commands
    if input == "q" || input == "b" || input == "0" {
        return Command::Quit;
    }

    // Help
    if input == "h" || input == "?" {
        return Command::Help;
    }

    // Menu
    if input == "m" {
        return Command::Menu;
    }

    // Back/Previous
    if input == "p" {
        return Command::Back;
    }

    // List links
    if input == "l" {
        return Command::List;
    }

    // Redisplay
    if input == "r" {
        return Command::Redisplay;
    }

    // Full page toggle
    if input == "f" {
        return Command::FullPageToggle;
    }

    // Set page size: op <num>
    if input.starts_with("op ") {
        if let Ok(size) = input[3..].trim().parse::<usize>() {
            if size >= 1 && size <= 99 {
                return Command::SetPageSize(size);
            }
        }
        return Command::Unknown(input);
    }

    // Just "op" shows current
    if input == "op" {
        return Command::SetPageSize(0); // 0 means show current
    }

    // New URL: n <url>
    if input.starts_with("n ") {
        let url = input[2..].trim().to_string();
        return Command::NewUrl(url);
    }

    // Search: s <query>
    if input.starts_with("s ") {
        let query = input[2..].trim().to_string();
        return Command::Search(query);
    }

    // Load link by number
    if let Ok(num) = input.parse::<usize>() {
        return Command::LoadLink(num);
    }

    Command::Unknown(input)
}
```

- [ ] **Step 4: Update lib.rs**

```rust
pub mod config;
pub mod session;
pub mod filter;
pub mod commands;
```

- [ ] **Step 5: Run test to verify it passes**

Run: `cargo test test_quit_commands`
Expected: PASS

- [ ] **Step 6: Write test for navigation commands**

Add to `tests/commands_test.rs`:
```rust
#[test]
fn test_navigation_commands() {
    assert!(matches!(parse_command("m"), Command::Menu));
    assert!(matches!(parse_command("p"), Command::Back));
    assert!(matches!(parse_command("l"), Command::List));
    assert!(matches!(parse_command("r"), Command::Redisplay));
    assert!(matches!(parse_command("f"), Command::FullPageToggle));
    assert!(matches!(parse_command("h"), Command::Help));
    assert!(matches!(parse_command("?"), Command::Help));
}
```

- [ ] **Step 7: Run test to verify it passes**

Run: `cargo test test_navigation_commands`
Expected: PASS

- [ ] **Step 8: Write test for link loading**

Add to `tests/commands_test.rs`:
```rust
#[test]
fn test_load_link() {
    assert_eq!(parse_command("1"), Command::LoadLink(1));
    assert_eq!(parse_command("42"), Command::LoadLink(42));
    assert_eq!(parse_command("999"), Command::LoadLink(999));
}
```

- [ ] **Step 9: Run test to verify it passes**

Run: `cargo test test_load_link`
Expected: PASS

- [ ] **Step 10: Write test for URL and search**

Add to `tests/commands_test.rs`:
```rust
#[test]
fn test_new_url() {
    assert_eq!(
        parse_command("n https://example.com"),
        Command::NewUrl("https://example.com".to_string())
    );
}

#[test]
fn test_search() {
    assert_eq!(
        parse_command("s rust programming"),
        Command::Search("rust programming".to_string())
    );
}
```

- [ ] **Step 11: Run tests to verify they pass**

Run: `cargo test test_new_url test_search`
Expected: PASS

- [ ] **Step 12: Commit**

```bash
git add src/commands.rs src/lib.rs tests/commands_test.rs
git commit -m "feat: add command parser for user input"
```

---

## Phase 5: Display & Pagination

### Task 8: Pagination Logic

**Files:**
- Create: `src/display.rs`
- Create: `tests/display_test.rs`

- [ ] **Step 1: Write failing test for pagination**

Create `tests/display_test.rs`:
```rust
use packet_browser::display::paginate;

#[test]
fn test_paginate_short_content() {
    let lines: Vec<String> = vec!["line1".to_string(), "line2".to_string()];
    let pages = paginate(&lines, 15);

    assert_eq!(pages.len(), 1);
    assert_eq!(pages[0].len(), 2);
}

#[test]
fn test_paginate_long_content() {
    let lines: Vec<String> = (0..30).map(|i| format!("line{}", i)).collect();
    let pages = paginate(&lines, 10);

    assert_eq!(pages.len(), 3);
    assert_eq!(pages[0].len(), 10);
    assert_eq!(pages[1].len(), 10);
    assert_eq!(pages[2].len(), 10);
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test test_paginate`
Expected: FAIL - module not found

- [ ] **Step 3: Implement pagination**

Create `src/display.rs`:
```rust
pub fn paginate(lines: &[String], lines_per_page: usize) -> Vec<Vec<String>> {
    if lines_per_page == 0 {
        return vec![lines.to_vec()];
    }

    lines
        .chunks(lines_per_page)
        .map(|chunk| chunk.to_vec())
        .collect()
}

pub fn format_help(lines_per_page: usize) -> String {
    format!(
        r#"Navigate pages using the number highlighted between [ ]
To view a particular page, enter just the page number.

If the page is longer than {} lines, you
will be prompted with the choice to continue or not.

Commands:
F - Toggle between Formatted Full Page and Paged
H - This text
L - List hyperlinks associated with the numbers
N <url> - Open <url>
M - Main Menu
OP <1-99> - Set Lines Per Page. OP<enter> shows.
P - Previous page (back)
R - Redisplay current page
S <text> - Search Wikipedia for <text>
Q/B - Quit/Bye"#,
        lines_per_page
    )
}

pub fn format_welcome(callsign: &str, version: &str) -> String {
    format!(
        "Hi {}, WWW V{}\nPage navigation numbers are highlighted with [ ]",
        callsign, version
    )
}

pub fn format_acknowledgment_prompt() -> &'static str {
    "All activity is logged including your callsign.\nType AGREE to proceed: "
}
```

- [ ] **Step 4: Update lib.rs**

```rust
pub mod config;
pub mod session;
pub mod filter;
pub mod commands;
pub mod display;
```

- [ ] **Step 5: Run tests to verify they pass**

Run: `cargo test test_paginate`
Expected: PASS

- [ ] **Step 6: Commit**

```bash
git add src/display.rs src/lib.rs tests/display_test.rs
git commit -m "feat: add pagination and display formatting"
```

---

## Phase 6: Logging

### Task 9: JSON Structured Logging

**Files:**
- Create: `src/logger.rs`
- Create: `tests/logger_test.rs`

- [ ] **Step 1: Write failing test for log entry**

Create `tests/logger_test.rs`:
```rust
use packet_browser::logger::{LogEntry, LogStatus};

#[test]
fn test_log_entry_serialization() {
    let entry = LogEntry::new(
        "W1ABC".to_string(),
        "https://example.com".to_string(),
        LogStatus::Ok,
        None,
    );

    let json = entry.to_json();
    assert!(json.contains("\"call\":\"W1ABC\""));
    assert!(json.contains("\"url\":\"https://example.com\""));
    assert!(json.contains("\"status\":\"ok\""));
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test test_log_entry_serialization`
Expected: FAIL - module not found

- [ ] **Step 3: Implement logger**

Create `src/logger.rs`:
```rust
use chrono::{DateTime, Utc};
use serde::{Serialize, Deserialize};
use std::fs::{File, OpenOptions};
use std::io::Write;
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum LogStatus {
    Ok,
    Blocked,
    Error,
    Agreed,
}

#[derive(Debug, Serialize)]
pub struct LogEntry {
    pub ts: DateTime<Utc>,
    pub call: String,
    pub url: String,
    pub status: LogStatus,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
}

impl LogEntry {
    pub fn new(call: String, url: String, status: LogStatus, reason: Option<String>) -> Self {
        Self {
            ts: Utc::now(),
            call,
            url,
            status,
            reason,
        }
    }

    pub fn to_json(&self) -> String {
        serde_json::to_string(self).unwrap_or_else(|_| "{}".to_string())
    }
}

pub struct Logger {
    log_path: String,
}

impl Logger {
    pub fn new(log_path: &str) -> Self {
        Self {
            log_path: log_path.to_string(),
        }
    }

    pub fn log(&self, entry: &LogEntry) -> std::io::Result<()> {
        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.log_path)?;

        writeln!(file, "{}", entry.to_json())?;
        Ok(())
    }
}
```

- [ ] **Step 4: Update lib.rs**

```rust
pub mod config;
pub mod session;
pub mod filter;
pub mod commands;
pub mod display;
pub mod logger;
```

- [ ] **Step 5: Run test to verify it passes**

Run: `cargo test test_log_entry_serialization`
Expected: PASS

- [ ] **Step 6: Write test for blocked status**

Add to `tests/logger_test.rs`:
```rust
#[test]
fn test_log_entry_with_reason() {
    let entry = LogEntry::new(
        "W1ABC".to_string(),
        "https://blocked.com".to_string(),
        LogStatus::Blocked,
        Some("dns_filter".to_string()),
    );

    let json = entry.to_json();
    assert!(json.contains("\"status\":\"blocked\""));
    assert!(json.contains("\"reason\":\"dns_filter\""));
}
```

- [ ] **Step 7: Run test to verify it passes**

Run: `cargo test test_log_entry_with_reason`
Expected: PASS

- [ ] **Step 8: Commit**

```bash
git add src/logger.rs src/lib.rs tests/logger_test.rs
git commit -m "feat: add JSON structured logging"
```

---

## Phase 7: Browser Integration

### Task 10: Chromium Wrapper

**Files:**
- Create: `src/browser.rs`

- [ ] **Step 1: Create browser module structure**

Create `src/browser.rs`:
```rust
use headless_chrome::{Browser, LaunchOptions};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum BrowserError {
    #[error("Failed to launch browser: {0}")]
    LaunchFailed(String),
    #[error("Failed to navigate: {0}")]
    NavigationFailed(String),
    #[error("Failed to extract content: {0}")]
    ExtractionFailed(String),
}

pub struct PageContent {
    pub text: Vec<String>,
    pub links: Vec<(usize, String)>, // (index, url)
}

pub struct BrowserInstance {
    browser: Browser,
}

impl BrowserInstance {
    pub fn new() -> Result<Self, BrowserError> {
        let launch_options = LaunchOptions::default_builder()
            .headless(true)
            .sandbox(true)
            .build()
            .map_err(|e| BrowserError::LaunchFailed(e.to_string()))?;

        let browser = Browser::new(launch_options)
            .map_err(|e| BrowserError::LaunchFailed(e.to_string()))?;

        Ok(Self { browser })
    }

    pub fn fetch_page(&self, url: &str) -> Result<PageContent, BrowserError> {
        let tab = self.browser.new_tab()
            .map_err(|e| BrowserError::LaunchFailed(e.to_string()))?;

        tab.navigate_to(url)
            .map_err(|e| BrowserError::NavigationFailed(e.to_string()))?;

        tab.wait_until_navigated()
            .map_err(|e| BrowserError::NavigationFailed(e.to_string()))?;

        // Extract text content
        let text_result = tab.evaluate(
            r#"document.body.innerText"#,
            false
        ).map_err(|e| BrowserError::ExtractionFailed(e.to_string()))?;

        let text = text_result.value
            .and_then(|v| v.as_str().map(|s| s.to_string()))
            .unwrap_or_default();

        let text_lines: Vec<String> = text.lines().map(|l| l.to_string()).collect();

        // Extract links
        let links_result = tab.evaluate(
            r#"
            Array.from(document.querySelectorAll('a[href]'))
                .map(a => a.href)
                .filter(href => href.startsWith('http'))
            "#,
            false
        ).map_err(|e| BrowserError::ExtractionFailed(e.to_string()))?;

        let links: Vec<(usize, String)> = links_result.value
            .and_then(|v| v.as_array().cloned())
            .unwrap_or_default()
            .into_iter()
            .filter_map(|v| v.as_str().map(|s| s.to_string()))
            .enumerate()
            .map(|(i, url)| (i + 1, url))
            .collect();

        Ok(PageContent { text: text_lines, links })
    }
}
```

- [ ] **Step 2: Update lib.rs**

```rust
pub mod config;
pub mod session;
pub mod filter;
pub mod commands;
pub mod display;
pub mod logger;
pub mod browser;
```

- [ ] **Step 3: Verify it compiles**

Run: `cargo build`
Expected: Build succeeds

- [ ] **Step 4: Commit**

```bash
git add src/browser.rs src/lib.rs
git commit -m "feat: add headless Chromium browser wrapper"
```

---

## Phase 8: TCP Server & Main Loop

### Task 11: TCP Server

**Files:**
- Modify: `src/main.rs`

- [ ] **Step 1: Implement TCP server with session handling**

Update `src/main.rs`:
```rust
use packet_browser::{
    config::Config,
    session::{Session, validate_callsign},
    filter::validate_url,
    commands::{parse_command, Command},
    display::{paginate, format_help, format_welcome, format_acknowledgment_prompt},
    logger::{Logger, LogEntry, LogStatus},
    browser::BrowserInstance,
};
use std::io::{BufRead, BufReader, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::Arc;

const VERSION: &str = env!("CARGO_PKG_VERSION");

fn main() {
    let config = Arc::new(Config::from_env());

    let addr = format!("0.0.0.0:{}", config.listen_port);
    let listener = TcpListener::bind(&addr).expect("Failed to bind");

    println!("packet-browser listening on {}", addr);

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                let config = Arc::clone(&config);
                std::thread::spawn(move || {
                    if let Err(e) = handle_connection(stream, &config) {
                        eprintln!("Connection error: {}", e);
                    }
                });
            }
            Err(e) => eprintln!("Accept error: {}", e),
        }
    }
}

fn handle_connection(mut stream: TcpStream, config: &Config) -> std::io::Result<()> {
    let mut reader = BufReader::new(stream.try_clone()?);

    // Get callsign
    let callsign = if config.debug_mode {
        writeln!(stream, "Please enter your callsign:")?;
        let mut line = String::new();
        reader.read_line(&mut line)?;
        line.trim().to_string()
    } else {
        let mut line = String::new();
        reader.read_line(&mut line)?;
        line.trim().to_string()
    };

    // Validate callsign
    let callsign = match validate_callsign(&callsign) {
        Ok(c) => c,
        Err(_) => {
            writeln!(stream, "Invalid callsign format. Disconnecting.")?;
            return Ok(());
        }
    };

    let mut session = Session::new(callsign.clone());
    session.lines_per_page = config.lines_per_page;

    // Acknowledgment
    write!(stream, "{}", format_acknowledgment_prompt())?;
    stream.flush()?;

    let mut line = String::new();
    reader.read_line(&mut line)?;

    if line.trim().to_lowercase() != "agree" {
        writeln!(stream, "You must agree to proceed. Disconnecting.")?;
        return Ok(());
    }

    session.acknowledge();

    // Log agreement
    let logger = Logger::new("/var/log/packet-browser/access.log");
    let _ = logger.log(&LogEntry::new(
        callsign.clone(),
        "SESSION_START".to_string(),
        LogStatus::Agreed,
        None,
    ));

    // Welcome message
    writeln!(stream, "{}", format_welcome(&callsign, VERSION))?;

    // Initialize browser
    let browser = match BrowserInstance::new() {
        Ok(b) => b,
        Err(e) => {
            writeln!(stream, "Failed to initialize browser: {}", e)?;
            return Ok(());
        }
    };

    // Load portal page
    if let Err(e) = fetch_and_display(&mut stream, &browser, &config.portal_url, &mut session, config, &logger) {
        writeln!(stream, "Error loading page: {}", e)?;
    }

    // Main loop
    loop {
        if session.is_timed_out(config.idle_timeout_minutes) {
            writeln!(stream, "Session timed out due to inactivity.")?;
            break;
        }

        write!(stream, "[H] for Help --> ")?;
        stream.flush()?;

        let mut input = String::new();
        if reader.read_line(&mut input)? == 0 {
            break; // EOF
        }

        session.touch();

        match parse_command(&input) {
            Command::Quit => {
                writeln!(stream, "Exiting... Bye!")?;
                break;
            }
            Command::Help => {
                writeln!(stream, "{}", format_help(session.lines_per_page))?;
            }
            Command::Menu => {
                if let Err(e) = fetch_and_display(&mut stream, &browser, &config.portal_url, &mut session, config, &logger) {
                    writeln!(stream, "Error: {}", e)?;
                }
            }
            Command::Back => {
                if let Some(ref url) = session.previous_url.clone() {
                    if let Err(e) = fetch_and_display(&mut stream, &browser, url, &mut session, config, &logger) {
                        writeln!(stream, "Error: {}", e)?;
                    }
                } else {
                    writeln!(stream, "Error: No previous page")?;
                }
            }
            Command::List => {
                for (idx, url) in &session.links {
                    writeln!(stream, "{} = {}", idx, url)?;
                }
            }
            Command::Redisplay => {
                display_content(&mut stream, &session)?;
            }
            Command::FullPageToggle => {
                session.full_page_mode = !session.full_page_mode;
                writeln!(stream, "{} Mode Set",
                    if session.full_page_mode { "Full Page" } else { "Paged" })?;
            }
            Command::SetPageSize(0) => {
                writeln!(stream, "Lines Per Page: {}", session.lines_per_page)?;
            }
            Command::SetPageSize(size) => {
                session.lines_per_page = size;
                writeln!(stream, "Paging is {} lines per page", size)?;
            }
            Command::LoadLink(num) => {
                if let Some((_, url)) = session.links.iter().find(|(i, _)| *i == num) {
                    let url = url.clone();
                    if let Err(e) = fetch_and_display(&mut stream, &browser, &url, &mut session, config, &logger) {
                        writeln!(stream, "Error: {}", e)?;
                    }
                } else {
                    writeln!(stream, "Error: Link {} not found", num)?;
                }
            }
            Command::NewUrl(url) => {
                let url = if !url.starts_with("http://") && !url.starts_with("https://") {
                    format!("http://{}", url)
                } else {
                    url
                };
                if let Err(e) = fetch_and_display(&mut stream, &browser, &url, &mut session, config, &logger) {
                    writeln!(stream, "Error: {}", e)?;
                }
            }
            Command::Search(query) => {
                let url = format!(
                    "https://en.m.wikipedia.org/w/index.php?title=Special:Search&ns0=1&search={}",
                    urlencoding::encode(&query)
                );
                writeln!(stream, "Wikipedia Search. Note: All queries are logged")?;
                writeln!(stream, "Processing: {}. Please wait.", query)?;
                if let Err(e) = fetch_and_display(&mut stream, &browser, &url, &mut session, config, &logger) {
                    writeln!(stream, "Error: {}", e)?;
                }
            }
            Command::Unknown(cmd) => {
                writeln!(stream, "Error: Unknown command '{}'. Try H for Help", cmd)?;
            }
        }
    }

    Ok(())
}

fn fetch_and_display(
    stream: &mut TcpStream,
    browser: &BrowserInstance,
    url: &str,
    session: &mut Session,
    config: &Config,
    logger: &Logger,
) -> Result<(), String> {
    // Validate URL
    validate_url(url, &config.blocked_ranges)
        .map_err(|e| e.to_string())?;

    writeln!(stream, "Wait...").map_err(|e| e.to_string())?;

    // Log the request
    let _ = logger.log(&LogEntry::new(
        session.callsign.clone(),
        url.to_string(),
        LogStatus::Ok,
        None,
    ));

    // Fetch page
    let content = browser.fetch_page(url)
        .map_err(|e| e.to_string())?;

    // Update session state
    session.previous_url = session.current_url.take();
    session.current_url = Some(url.to_string());
    session.page_content = content.text;
    session.links = content.links;

    // Display
    display_content(stream, session).map_err(|e| e.to_string())?;

    Ok(())
}

fn display_content(stream: &mut TcpStream, session: &Session) -> std::io::Result<()> {
    if session.full_page_mode {
        for line in &session.page_content {
            writeln!(stream, "{}", line)?;
        }
    } else {
        let pages = paginate(&session.page_content, session.lines_per_page);
        for (i, page) in pages.iter().enumerate() {
            for line in page {
                writeln!(stream, "{}", line)?;
            }
            if i < pages.len() - 1 {
                writeln!(stream, "ENTER = continue, A = Abort [Page {}/{}]", i + 1, pages.len())?;
                // In full implementation, read input here
            }
        }
    }
    Ok(())
}
```

- [ ] **Step 2: Add urlencoding dependency**

Update `Cargo.toml` dependencies:
```toml
urlencoding = "2"
```

- [ ] **Step 3: Verify it compiles**

Run: `cargo build`
Expected: Build succeeds

- [ ] **Step 4: Commit**

```bash
git add src/main.rs Cargo.toml
git commit -m "feat: add TCP server with main session loop"
```

---

## Phase 9: Container Build

### Task 12: Nix Flake

**Files:**
- Create: `flake.nix`

- [ ] **Step 1: Create Nix flake**

Create `flake.nix`:
```nix
{
  description = "Packet Browser - Secure web browser for packet radio";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    rust-overlay.url = "github:oxalica/rust-overlay";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, rust-overlay, flake-utils }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        overlays = [ (import rust-overlay) ];
        pkgs = import nixpkgs { inherit system overlays; };

        rustToolchain = pkgs.rust-bin.stable.latest.default;

        packet-browser = pkgs.rustPlatform.buildRustPackage {
          pname = "packet-browser";
          version = "0.1.0";
          src = ./.;
          cargoLock.lockFile = ./Cargo.lock;

          nativeBuildInputs = [ pkgs.pkg-config ];
          buildInputs = [ pkgs.openssl ];
        };

        dockerImage = pkgs.dockerTools.buildImage {
          name = "packet-browser";
          tag = "latest";

          copyToRoot = pkgs.buildEnv {
            name = "image-root";
            paths = [
              packet-browser
              pkgs.chromium
              pkgs.dumb-init
              pkgs.logrotate
              pkgs.cacert
            ];
            pathsToLink = [ "/bin" "/etc" ];
          };

          config = {
            Cmd = [ "/bin/dumb-init" "/bin/packet-browser" ];
            ExposedPorts = { "63004/tcp" = {}; };
            Env = [
              "SSL_CERT_FILE=/etc/ssl/certs/ca-bundle.crt"
            ];
            User = "1000:1000";
          };

          runAsRoot = ''
            mkdir -p /var/log/packet-browser
            mkdir -p /tmp
            chown 1000:1000 /var/log/packet-browser
          '';
        };
      in
      {
        packages = {
          default = packet-browser;
          docker-image = dockerImage;
        };

        devShells.default = pkgs.mkShell {
          buildInputs = [
            rustToolchain
            pkgs.pkg-config
            pkgs.openssl
            pkgs.chromium
          ];
        };
      }
    );
}
```

- [ ] **Step 2: Generate Cargo.lock**

Run: `cargo generate-lockfile`

- [ ] **Step 3: Verify flake evaluates**

Run: `nix flake check`
Expected: No errors

- [ ] **Step 4: Commit**

```bash
git add flake.nix Cargo.lock
git commit -m "feat: add Nix flake for container build"
```

---

### Task 13: Docker Compose

**Files:**
- Create: `docker-compose.yml`

- [ ] **Step 1: Create docker-compose.yml**

Create `docker-compose.yml`:
```yaml
version: '3.8'

services:
  packet-browser:
    image: packet-browser:latest
    # Or use: ghcr.io/ben-kuhn/packet-browser:latest

    ports:
      # Bind to loopback only by default (security)
      - "127.0.0.1:63004:63004"

    volumes:
      # Logs - accessible from host
      - ./logs:/var/log/packet-browser
      # Hosts file for blocklist management
      - ./hosts:/etc/hosts

    environment:
      # Service configuration
      - LISTEN_PORT=63004
      - PORTAL_URL=http://matrix.ehvairport.com/~bpq/
      - IDLE_TIMEOUT_MINUTES=10
      - LINES_PER_PAGE=15

      # DNS filtering (OpenDNS Family Shield)
      - DNS_SERVERS=208.67.222.123,208.67.220.123

      # SSRF prevention - blocked IP ranges
      # Remove ranges to allow access to local services
      - BLOCKED_RANGES=127.0.0.0/8,10.0.0.0/8,172.16.0.0/12,192.168.0.0/16,169.254.0.0/16

      # Blocklist settings
      - BLOCKLIST_ENABLED=true
      - BLOCKLIST_REFRESH_HOURS=24
      # - BLOCKLIST_URLS=https://example.com/blocklist.txt

      # Logging
      - LOG_ROTATE_ENABLED=true
      - LOG_RETAIN_DAYS=30
      - SYSLOG_ENABLED=false
      # - SYSLOG_HOST=syslog.example.com
      # - SYSLOG_PORT=514

      # Debug mode - for testing without BPQ
      - DEBUG_MODE=false

    # Security hardening
    read_only: true
    tmpfs:
      - /tmp:size=64M

    cap_drop:
      - ALL
    cap_add:
      - NET_RAW  # Required for DNS

    # Health check
    healthcheck:
      test: ["CMD", "nc", "-z", "localhost", "63004"]
      interval: 30s
      timeout: 5s
      retries: 3

    restart: unless-stopped
```

- [ ] **Step 2: Create empty logs directory**

Run: `mkdir -p logs && touch logs/.gitkeep`

- [ ] **Step 3: Create initial hosts file**

Create `hosts`:
```
127.0.0.1 localhost

# Admin custom entries (add your own above this line)

# BLOCKLIST-MANAGED START
# BLOCKLIST-MANAGED END
```

- [ ] **Step 4: Commit**

```bash
git add docker-compose.yml logs/.gitkeep hosts
git commit -m "feat: add Docker Compose deployment configuration"
```

---

## Phase 10: CI/CD

### Task 14: GitHub Actions

**Files:**
- Create: `.github/workflows/build.yml`

- [ ] **Step 1: Create GitHub Actions workflow**

Create `.github/workflows/build.yml`:
```yaml
name: Build and Publish

on:
  push:
    branches: [main]
    tags: ['v*']
  pull_request:
    branches: [main]

env:
  REGISTRY: ghcr.io
  IMAGE_NAME: ${{ github.repository }}

jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      - name: Install Rust
        uses: dtolnay/rust-action@stable

      - name: Run tests
        run: cargo test --all-features

  build:
    needs: test
    runs-on: ubuntu-latest
    permissions:
      contents: read
      packages: write

    steps:
      - uses: actions/checkout@v4

      - name: Install Nix
        uses: cachix/install-nix-action@v24
        with:
          nix_path: nixpkgs=channel:nixos-unstable

      - name: Build Docker image
        run: nix build .#docker-image

      - name: Load image
        run: docker load < result

      - name: Log in to Container Registry
        if: github.event_name != 'pull_request'
        uses: docker/login-action@v3
        with:
          registry: ${{ env.REGISTRY }}
          username: ${{ github.actor }}
          password: ${{ secrets.GITHUB_TOKEN }}

      - name: Extract metadata
        id: meta
        uses: docker/metadata-action@v5
        with:
          images: ${{ env.REGISTRY }}/${{ env.IMAGE_NAME }}
          tags: |
            type=ref,event=branch
            type=ref,event=pr
            type=semver,pattern={{version}}
            type=semver,pattern={{major}}.{{minor}}
            type=raw,value=latest,enable={{is_default_branch}}

      - name: Tag and push image
        if: github.event_name != 'pull_request'
        run: |
          for tag in ${{ steps.meta.outputs.tags }}; do
            docker tag packet-browser:latest $tag
            docker push $tag
          done
```

- [ ] **Step 2: Commit**

```bash
mkdir -p .github/workflows
git add .github/workflows/build.yml
git commit -m "feat: add GitHub Actions CI/CD pipeline"
```

---

## Phase 11: Documentation

### Task 15: Update README

**Files:**
- Modify: `README.md`

- [ ] **Step 1: Update README with new deployment instructions**

The README should be updated to document:
- Docker Compose deployment
- Environment variables
- Building from source with Nix
- BPQ configuration
- Debug mode for testing

- [ ] **Step 2: Commit**

```bash
git add README.md
git commit -m "docs: update README with Docker deployment instructions"
```

---

## Summary

This plan implements the packet browser container in 15 tasks across 11 phases:

1. **Project Setup** - Rust project initialization
2. **Configuration** - Environment variable handling
3. **Session Management** - Callsign validation and state
4. **URL Filtering** - Protocol and SSRF protection
5. **Command Parsing** - User input handling
6. **Display** - Pagination and formatting
7. **Logging** - JSON structured logs
8. **Browser Integration** - Headless Chromium
9. **TCP Server** - Main application loop
10. **Container Build** - Nix flake and Docker
11. **CI/CD** - GitHub Actions
12. **Documentation** - README updates

Each task follows TDD with explicit test-first steps.
