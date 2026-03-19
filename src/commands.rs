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
