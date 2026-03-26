#[derive(Debug, PartialEq, Eq)]
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
    FillInput(usize, Option<String>),
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

    // Fill/interact with input field: i<num> [value] or i <num> [value]
    // Accepts: I1 search term, I 1 search term, I1, I 1
    if input.starts_with("i") && input.len() > 1 {
        let rest = input[1..].trim_start(); // Skip 'i', then optional whitespace

        // Try to parse the number (with or without space after 'i')
        let (num_str, remainder) = if let Some(space_pos) = rest.find(' ') {
            (&rest[..space_pos], rest[space_pos + 1..].trim())
        } else {
            (rest, "")
        };

        // Extract number from start of num_str (handles "1" or "1abc" edge cases)
        let num_end = num_str.chars().take_while(|c| c.is_ascii_digit()).count();
        if num_end > 0 {
            if let Ok(num) = num_str[..num_end].parse::<usize>() {
                // Check if there's remaining text after the number
                let after_num = &num_str[num_end..];
                let value = if !after_num.is_empty() {
                    // e.g., "i1abc" -> num=1, value="abc"
                    Some(format!("{}{}", after_num, if remainder.is_empty() { "" } else { " " }).to_string() + remainder)
                } else if !remainder.is_empty() {
                    Some(remainder.to_string())
                } else {
                    None
                };
                return Command::FillInput(num, value.map(|v| v.trim().to_string()).filter(|v| !v.is_empty()));
            }
        }
    }

    // Load link by number
    if let Ok(num) = input.parse::<usize>() {
        return Command::LoadLink(num);
    }

    Command::Unknown(input)
}
