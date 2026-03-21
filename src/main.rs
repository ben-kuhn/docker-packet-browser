use packet_browser::{
    blocklist::start_blocklist_manager,
    browser::BrowserInstance,
    commands::{parse_command, Command},
    config::Config,
    display::{format_acknowledgment_prompt, format_help, format_page_footer, format_welcome, paginate},
    filter::validate_url,
    logger::{LogEntry, LogStatus, Logger},
    session::{validate_callsign, Session},
};
use std::io::{BufRead, BufReader, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::Arc;
use std::thread;

const VERSION: &str = "0.1.0";

fn main() {
    let config = Arc::new(Config::from_env());

    println!("Starting packet-browser v{}", VERSION);
    println!("Listening on port {}", config.listen_port);

    if config.blocklist_enabled && !config.blocklist_urls.is_empty() {
        start_blocklist_manager(config.blocklist_urls.clone(), config.blocklist_refresh_hours);
    }

    let listener = TcpListener::bind(format!("0.0.0.0:{}", config.listen_port))
        .expect("Failed to bind to port");

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                let config = Arc::clone(&config);
                thread::spawn(move || {
                    if let Err(e) = handle_connection(stream, config) {
                        eprintln!("Connection error: {}", e);
                    }
                });
            }
            Err(e) => {
                eprintln!("Failed to accept connection: {}", e);
            }
        }
    }
}

fn handle_connection(mut stream: TcpStream, config: Arc<Config>) -> std::io::Result<()> {
    let mut reader = BufReader::new(stream.try_clone()?);

    // Read callsign
    let callsign = if config.debug_mode {
        write!(stream, "Enter callsign: ")?;
        stream.flush()?;
        let mut input = String::new();
        reader.read_line(&mut input)?;
        input.trim().to_string()
    } else {
        // In production, read from BPQ
        let mut input = String::new();
        reader.read_line(&mut input)?;
        input.trim().to_string()
    };

    // Validate callsign
    let callsign = match validate_callsign(&callsign) {
        Ok(call) => call,
        Err(_) => {
            writeln!(stream, "Invalid callsign format.")?;
            return Ok(());
        }
    };

    // Create session
    let mut session = Session::new(callsign.clone());

    // Show acknowledgment prompt
    write!(stream, "{}", format_acknowledgment_prompt())?;
    stream.flush()?;

    let mut input = String::new();
    reader.read_line(&mut input)?;

    if input.trim().to_uppercase() != "AGREE" {
        writeln!(stream, "Acknowledgment required. Goodbye.")?;
        return Ok(());
    }

    session.acknowledge();

    // Log agreement
    let logger = Logger::new("/var/log/packet-browser/access.log");
    let log_entry = LogEntry::new(
        callsign.clone(),
        "AGREED".to_string(),
        LogStatus::Agreed,
        None,
    );
    let _ = logger.log(&log_entry);

    // Show welcome message
    writeln!(stream, "\n{}\n", format_welcome(&callsign, VERSION))?;

    // Initialize browser
    let browser = match BrowserInstance::new() {
        Ok(b) => b,
        Err(e) => {
            writeln!(stream, "Failed to initialize browser: {}", e)?;
            return Ok(());
        }
    };

    // Load portal page
    if let Err(e) = load_page(&mut session, &browser, &config, &logger, &mut stream, &config.portal_url) {
        writeln!(stream, "Failed to load portal: {}", e)?;
    }

    // Main command loop
    loop {
        // Check timeout
        if session.is_timed_out(config.idle_timeout_minutes) {
            writeln!(stream, "\nSession timed out due to inactivity.")?;
            break;
        }

        write!(stream, "\nCommand: ")?;
        stream.flush()?;

        let mut input = String::new();
        if reader.read_line(&mut input).is_err() {
            break;
        }

        session.touch();

        let command = parse_command(&input);

        match command {
            Command::Quit => {
                writeln!(stream, "Goodbye!")?;
                break;
            }
            Command::Help => {
                writeln!(stream, "\n{}\n", format_help(session.lines_per_page))?;
            }
            Command::Menu => {
                let url = config.portal_url.clone();
                if let Err(e) = load_page(&mut session, &browser, &config, &logger, &mut stream, &url) {
                    writeln!(stream, "Error loading menu: {}", e)?;
                }
            }
            Command::Back => {
                if let Some(prev_url) = session.previous_url.clone() {
                    if let Err(e) = load_page(&mut session, &browser, &config, &logger, &mut stream, &prev_url) {
                        writeln!(stream, "Error loading previous page: {}", e)?;
                    }
                } else {
                    writeln!(stream, "No previous page.")?;
                }
            }
            Command::List => {
                writeln!(stream, "\nAvailable links:")?;
                for (idx, url) in &session.links {
                    writeln!(stream, "[{}] {}", idx, url)?;
                }
                if session.links.is_empty() {
                    writeln!(stream, "No links on this page.")?;
                }
            }
            Command::Redisplay => {
                display_content(&mut session, &mut stream)?;
            }
            Command::FullPageToggle => {
                session.full_page_mode = !session.full_page_mode;
                if session.full_page_mode {
                    writeln!(stream, "Full page mode enabled.")?;
                } else {
                    writeln!(stream, "Paged mode enabled.")?;
                }
                display_content(&mut session, &mut stream)?;
            }
            Command::SetPageSize(size) => {
                if size == 0 {
                    writeln!(stream, "Current page size: {} lines", session.lines_per_page)?;
                } else {
                    session.lines_per_page = size;
                    writeln!(stream, "Page size set to {} lines.", size)?;
                }
            }
            Command::LoadLink(num) => {
                if let Some((_, url)) = session.links.iter().find(|(idx, _)| *idx == num) {
                    let url = url.clone();
                    if let Err(e) = load_page(&mut session, &browser, &config, &logger, &mut stream, &url) {
                        writeln!(stream, "Error loading link: {}", e)?;
                    }
                } else {
                    writeln!(stream, "Invalid link number.")?;
                }
            }
            Command::NewUrl(url) => {
                if let Err(e) = load_page(&mut session, &browser, &config, &logger, &mut stream, &url) {
                    writeln!(stream, "Error loading URL: {}", e)?;
                }
            }
            Command::Search(query) => {
                let encoded_query = urlencoding::encode(&query);
                let url = format!("https://en.wikipedia.org/wiki/Special:Search?search={}", encoded_query);
                if let Err(e) = load_page(&mut session, &browser, &config, &logger, &mut stream, &url) {
                    writeln!(stream, "Error searching: {}", e)?;
                }
            }
            Command::FillInput(num, text) => {
                if let Some(url) = session.current_url.clone() {
                    writeln!(stream, "Submitting...")?;
                    stream.flush()?;
                    match browser.fill_and_submit(&url, num, &text) {
                        Ok(content) => {
                            session.previous_url = Some(url);
                            session.links = content.links;
                            session.inputs = content.inputs;
                            session.page_content = content.text;
                            display_content(&mut session, &mut stream)?;
                        }
                        Err(e) => {
                            writeln!(stream, "Failed to submit input: {}", e)?;
                        }
                    }
                } else {
                    writeln!(stream, "No page loaded.")?;
                }
            }
            Command::Unknown(cmd) => {
                writeln!(stream, "Unknown command: '{}'. Type H for help.", cmd)?;
            }
        }
    }

    Ok(())
}

fn load_page(
    session: &mut Session,
    browser: &BrowserInstance,
    config: &Config,
    logger: &Logger,
    stream: &mut TcpStream,
    url: &str,
) -> std::io::Result<()> {
    // Validate URL
    if let Err(e) = validate_url(url, &config.blocked_ranges) {
        writeln!(stream, "URL blocked: {}", e)?;
        let log_entry = LogEntry::new(
            session.callsign.clone(),
            url.to_string(),
            LogStatus::Blocked,
            Some(e.to_string()),
        );
        let _ = logger.log(&log_entry);
        return Ok(());
    }

    // Fetch page
    writeln!(stream, "Loading...")?;
    stream.flush()?;

    let page_content = match browser.fetch_page(url) {
        Ok(content) => content,
        Err(e) => {
            writeln!(stream, "Failed to load page: {}", e)?;
            let log_entry = LogEntry::new(
                session.callsign.clone(),
                url.to_string(),
                LogStatus::Error,
                Some(e.to_string()),
            );
            let _ = logger.log(&log_entry);
            return Ok(());
        }
    };

    // Log successful access
    let log_entry = LogEntry::new(
        session.callsign.clone(),
        url.to_string(),
        LogStatus::Ok,
        None,
    );
    let _ = logger.log(&log_entry);

    // Update session
    session.previous_url = session.current_url.clone();
    session.current_url = Some(url.to_string());
    session.links = page_content.links;
    session.inputs = page_content.inputs;
    session.page_content = page_content.text;

    // Display content
    display_content(session, stream)?;

    Ok(())
}

fn display_content(session: &mut Session, stream: &mut TcpStream) -> std::io::Result<()> {
    if session.page_content.is_empty() {
        writeln!(stream, "No content to display.")?;
        return Ok(());
    }

    if session.full_page_mode {
        for line in &session.page_content {
            writeln!(stream, "{}", line)?;
        }
    } else {
        let pages = paginate(&session.page_content, session.lines_per_page);

        for (page_num, page) in pages.iter().enumerate() {
            for line in page {
                writeln!(stream, "{}", line)?;
            }

            if page_num < pages.len() - 1 {
                write!(stream, "\nPress ENTER to continue, or Q to stop: ")?;
                stream.flush()?;

                let mut reader = BufReader::new(stream.try_clone()?);
                let mut input = String::new();
                reader.read_line(&mut input)?;

                if input.trim().to_lowercase() == "q" {
                    break;
                }
            }
        }
    }

    writeln!(stream, "\n{}", format_page_footer(&session.inputs))?;

    Ok(())
}
