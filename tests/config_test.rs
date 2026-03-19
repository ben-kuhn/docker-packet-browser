use packet_browser::config::Config;

#[test]
fn test_config_defaults() {
    // Clear relevant env vars to ensure defaults are used
    let env_vars = vec![
        "LISTEN_PORT",
        "PORTAL_URL",
        "IDLE_TIMEOUT_MINUTES",
        "DNS_SERVERS",
        "BLOCKED_RANGES",
        "BLOCKLIST_URLS",
        "BLOCKLIST_REFRESH_HOURS",
        "BLOCKLIST_ENABLED",
        "LOG_ROTATE_ENABLED",
        "LOG_RETAIN_DAYS",
        "SYSLOG_ENABLED",
        "SYSLOG_HOST",
        "SYSLOG_PORT",
        "LINES_PER_PAGE",
        "DEBUG_MODE",
    ];

    for var in &env_vars {
        std::env::remove_var(var);
    }

    let config = Config::from_env();

    // Verify all defaults
    assert_eq!(config.listen_port, 63004);
    assert_eq!(config.portal_url, "http://matrix.ehvairport.com/~bpq/");
    assert_eq!(config.idle_timeout_minutes, 10);
    assert_eq!(
        config.dns_servers,
        vec!["208.67.222.123", "208.67.220.123"]
    );
    assert_eq!(
        config.blocked_ranges,
        vec![
            "127.0.0.0/8",
            "10.0.0.0/8",
            "172.16.0.0/12",
            "192.168.0.0/16",
            "169.254.0.0/16"
        ]
    );
    assert!(config.blocklist_urls.is_empty());
    assert_eq!(config.blocklist_refresh_hours, 24);
    assert!(config.blocklist_enabled);
    assert!(config.log_rotate_enabled);
    assert_eq!(config.log_retain_days, 30);
    assert!(!config.syslog_enabled);
    assert_eq!(config.syslog_host, None);
    assert_eq!(config.syslog_port, 514);
    assert_eq!(config.lines_per_page, 15);
    assert!(!config.debug_mode);
}

#[test]
fn test_config_env_override() {
    // Clear env vars first
    let env_vars = vec![
        "LISTEN_PORT",
        "PORTAL_URL",
        "IDLE_TIMEOUT_MINUTES",
        "DNS_SERVERS",
        "BLOCKED_RANGES",
        "BLOCKLIST_URLS",
        "BLOCKLIST_REFRESH_HOURS",
        "BLOCKLIST_ENABLED",
        "LOG_ROTATE_ENABLED",
        "LOG_RETAIN_DAYS",
        "SYSLOG_ENABLED",
        "SYSLOG_HOST",
        "SYSLOG_PORT",
        "LINES_PER_PAGE",
        "DEBUG_MODE",
    ];

    for var in &env_vars {
        std::env::remove_var(var);
    }

    // Set env vars with custom values
    std::env::set_var("LISTEN_PORT", "8080");
    std::env::set_var("PORTAL_URL", "http://custom.example.com/");
    std::env::set_var("IDLE_TIMEOUT_MINUTES", "30");
    std::env::set_var("DNS_SERVERS", "1.1.1.1, 8.8.8.8");
    std::env::set_var("BLOCKED_RANGES", "192.168.1.0/24, 10.0.0.0/8");
    std::env::set_var("BLOCKLIST_URLS", "http://example.com/list1, http://example.com/list2");
    std::env::set_var("BLOCKLIST_REFRESH_HOURS", "48");
    std::env::set_var("BLOCKLIST_ENABLED", "false");
    std::env::set_var("LOG_ROTATE_ENABLED", "false");
    std::env::set_var("LOG_RETAIN_DAYS", "60");
    std::env::set_var("SYSLOG_ENABLED", "true");
    std::env::set_var("SYSLOG_HOST", "localhost");
    std::env::set_var("SYSLOG_PORT", "1514");
    std::env::set_var("LINES_PER_PAGE", "25");
    std::env::set_var("DEBUG_MODE", "true");

    let config = Config::from_env();

    // Verify all overrides
    assert_eq!(config.listen_port, 8080);
    assert_eq!(config.portal_url, "http://custom.example.com/");
    assert_eq!(config.idle_timeout_minutes, 30);
    assert_eq!(config.dns_servers, vec!["1.1.1.1", "8.8.8.8"]);
    assert_eq!(
        config.blocked_ranges,
        vec!["192.168.1.0/24", "10.0.0.0/8"]
    );
    assert_eq!(
        config.blocklist_urls,
        vec!["http://example.com/list1", "http://example.com/list2"]
    );
    assert_eq!(config.blocklist_refresh_hours, 48);
    assert!(!config.blocklist_enabled);
    assert!(!config.log_rotate_enabled);
    assert_eq!(config.log_retain_days, 60);
    assert!(config.syslog_enabled);
    assert_eq!(config.syslog_host, Some("localhost".to_string()));
    assert_eq!(config.syslog_port, 1514);
    assert_eq!(config.lines_per_page, 25);
    assert!(config.debug_mode);

    // Clean up env vars
    for var in &env_vars {
        std::env::remove_var(var);
    }
}

#[test]
fn test_config_bool_parsing() {
    // Clear env vars
    std::env::remove_var("DEBUG_MODE");

    // Test various true values
    std::env::set_var("DEBUG_MODE", "true");
    let config = Config::from_env();
    assert!(config.debug_mode);

    std::env::set_var("DEBUG_MODE", "1");
    let config = Config::from_env();
    assert!(config.debug_mode);

    std::env::set_var("DEBUG_MODE", "yes");
    let config = Config::from_env();
    assert!(config.debug_mode);

    std::env::set_var("DEBUG_MODE", "on");
    let config = Config::from_env();
    assert!(config.debug_mode);

    // Test various false values
    std::env::set_var("DEBUG_MODE", "false");
    let config = Config::from_env();
    assert!(!config.debug_mode);

    std::env::set_var("DEBUG_MODE", "0");
    let config = Config::from_env();
    assert!(!config.debug_mode);

    std::env::set_var("DEBUG_MODE", "no");
    let config = Config::from_env();
    assert!(!config.debug_mode);

    std::env::set_var("DEBUG_MODE", "off");
    let config = Config::from_env();
    assert!(!config.debug_mode);

    // Clean up
    std::env::remove_var("DEBUG_MODE");
}

#[test]
fn test_config_invalid_env_values_use_defaults() {
    // Clear env vars
    std::env::remove_var("LISTEN_PORT");
    std::env::remove_var("IDLE_TIMEOUT_MINUTES");
    std::env::remove_var("SYSLOG_PORT");

    // Set invalid values
    std::env::set_var("LISTEN_PORT", "not_a_number");
    std::env::set_var("IDLE_TIMEOUT_MINUTES", "invalid");
    std::env::set_var("SYSLOG_PORT", "not_a_port");

    let config = Config::from_env();

    // Should use defaults when parsing fails
    assert_eq!(config.listen_port, 63004);
    assert_eq!(config.idle_timeout_minutes, 10);
    assert_eq!(config.syslog_port, 514);

    // Clean up
    std::env::remove_var("LISTEN_PORT");
    std::env::remove_var("IDLE_TIMEOUT_MINUTES");
    std::env::remove_var("SYSLOG_PORT");
}
