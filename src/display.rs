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

    let mut lines = vec!["--- Inputs ---".to_string()];
    for field in inputs {
        let detail = match &field.kind {
            InputKind::Text => "(text)".to_string(),
            InputKind::Select { options } => {
                let opts = options.iter().enumerate()
                    .map(|(i, o)| format!("{}={}", i + 1, o))
                    .collect::<Vec<_>>()
                    .join(" ");
                format!("(select) {}", opts)
            }
            InputKind::Radio { options } => {
                let opts = options.iter().enumerate()
                    .map(|(i, o)| format!("{}={}", i + 1, o))
                    .collect::<Vec<_>>()
                    .join(" ");
                format!("(radio) {}", opts)
            }
            InputKind::Checkbox { checked } => {
                format!("(checkbox) {}", if *checked { "ON" } else { "OFF" })
            }
        };
        lines.push(format!("[I{}] {} {}", field.index, field.label, detail));
    }
    lines
}

pub fn format_page_footer() -> &'static str {
    "--- H=Help N=URL S=Search I<n>=Input P=Back M=Menu Q=Quit ---"
}
