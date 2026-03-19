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
