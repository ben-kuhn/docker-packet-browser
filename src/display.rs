use crate::browser::{InputField, InputKind};

pub fn paginate(lines: &[String], lines_per_page: usize) -> Vec<Vec<String>> {
    if lines_per_page == 0 {
        return vec![lines.to_vec()];
    }
    lines.chunks(lines_per_page).map(|chunk| chunk.to_vec()).collect()
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
I <n> [value] - Interact with input field n:
    Text/search: I 1 my search query  (fills and submits)
    Select/radio: I 2 3               (picks option 3 and submits)
    Checkbox: I 3                     (toggles and submits)
L - List hyperlinks associated with the numbers
N <url> - Open URL (e.g. N https://example.com)
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
        "Hi {}, WWW V{}\nPage navigation numbers are highlighted with [ ]\nType H for help. N <url> to open a URL. S <text> to search.",
        callsign, version
    )
}

pub fn format_acknowledgment_prompt() -> &'static str {
    "All activity is logged including your callsign.\nType AGREE to proceed: "
}

pub fn format_inputs_section(inputs: &[InputField]) -> Vec<String> {
    if inputs.is_empty() {
        return vec![];
    }

    let mut lines = vec!["--- Inputs: I<n> <value> to fill, I<n> to toggle ---".to_string()];
    for field in inputs {
        // Make label more user-friendly - if it's a short cryptic name, show type hint
        let label = if field.label.is_empty() || field.label.len() <= 2 {
            match &field.kind {
                InputKind::Text => "text input".to_string(),
                InputKind::Select { .. } => "dropdown".to_string(),
                InputKind::Radio { .. } => "choice".to_string(),
                InputKind::Checkbox { .. } => "checkbox".to_string(),
            }
        } else {
            field.label.clone()
        };

        let detail = match &field.kind {
            InputKind::Text => "-> I{} your text here".to_string(),
            InputKind::Select { options } => {
                let opts: Vec<String> = options.iter().take(4).enumerate()
                    .map(|(i, o)| format!("{}={}", i + 1, truncate_str(o, 15)))
                    .collect();
                let more = if options.len() > 4 { "..." } else { "" };
                format!("-> I{{}} <1-{}> {}{}", options.len(), opts.join(" "), more)
            }
            InputKind::Radio { options } => {
                let opts: Vec<String> = options.iter().take(4).enumerate()
                    .map(|(i, o)| format!("{}={}", i + 1, truncate_str(o, 15)))
                    .collect();
                let more = if options.len() > 4 { "..." } else { "" };
                format!("-> I{{}} <1-{}> {}{}", options.len(), opts.join(" "), more)
            }
            InputKind::Checkbox { checked } => {
                format!("[{}] -> I{{}} to toggle", if *checked { "X" } else { " " })
            }
        };
        let detail = detail.replace("{}", &field.index.to_string());
        lines.push(format!("[I{}] {}: {}", field.index, label, detail));
    }
    lines
}

fn truncate_str(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else {
        format!("{}...", &s[..max_len.saturating_sub(3)])
    }
}

pub fn format_page_footer() -> &'static str {
    "--- H=Help N=URL S=Search I<n>=Input P=Back M=Menu Q=Quit ---"
}

/// Format a summary of available links for display below content.
/// Shows first few links that fit, with numbers for easy navigation.
pub fn format_links_summary(links: &[(usize, String)], max_links: usize) -> String {
    if links.is_empty() {
        return String::new();
    }

    let display_links: Vec<String> = links.iter()
        .take(max_links)
        .map(|(idx, url)| {
            // Truncate long URLs to fit in 80 columns
            let truncated = if url.len() > 30 {
                format!("{}...", &url[..27])
            } else {
                url.clone()
            };
            format!("[{}]{}", idx, truncated)
        })
        .collect();

    let summary = display_links.join(" ");
    let more = if links.len() > max_links {
        format!(" +{} more (L=list)", links.len() - max_links)
    } else {
        String::new()
    };

    format!("{}{}", summary, more)
}
