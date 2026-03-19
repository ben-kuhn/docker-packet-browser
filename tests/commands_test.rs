use packet_browser::commands::{parse_command, Command};

#[test]
fn test_quit_commands() {
    assert!(matches!(parse_command("q"), Command::Quit));
    assert!(matches!(parse_command("Q"), Command::Quit));
    assert!(matches!(parse_command("b"), Command::Quit));
    assert!(matches!(parse_command("0"), Command::Quit));
}

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

#[test]
fn test_load_link() {
    assert_eq!(parse_command("1"), Command::LoadLink(1));
    assert_eq!(parse_command("42"), Command::LoadLink(42));
    assert_eq!(parse_command("999"), Command::LoadLink(999));
}

#[test]
fn test_new_url() {
    assert_eq!(parse_command("n https://example.com"), Command::NewUrl("https://example.com".to_string()));
}

#[test]
fn test_search() {
    assert_eq!(parse_command("s rust programming"), Command::Search("rust programming".to_string()));
}

#[test]
fn test_page_size() {
    assert_eq!(parse_command("op"), Command::SetPageSize(0));
    assert_eq!(parse_command("op 10"), Command::SetPageSize(10));
    assert_eq!(parse_command("op 50"), Command::SetPageSize(50));
}

#[test]
fn test_unknown_commands() {
    assert!(matches!(parse_command("xyz"), Command::Unknown(_)));
    assert!(matches!(parse_command("invalid"), Command::Unknown(_)));
}

#[test]
fn test_whitespace_handling() {
    assert!(matches!(parse_command("  q  "), Command::Quit));
    assert!(matches!(parse_command("  m  "), Command::Menu));
    assert_eq!(parse_command("n  https://example.com  "), Command::NewUrl("https://example.com".to_string()));
}
