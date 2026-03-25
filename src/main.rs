use packet_browser::{
    blocklist::start_blocklist_manager,
    browser::{BrowserInstance, InputKind},
    commands::{parse_command, Command},
    config::Config,
    display::{format_acknowledgment_prompt, format_help, format_inputs_section, format_links_summary, format_page_footer, format_welcome, paginate},
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
    // Health check mode: attempt a TCP connect to the service port and exit 0/1.
    // Used by the Docker HEALTHCHECK so no shell or netcat is needed in the image.
    if std::env::args().any(|a| a == "--healthcheck") {
        let port = std::env::var("LISTEN_PORT")
            .ok()
            .and_then(|v| v.parse::<u16>().ok())
            .unwrap_or(63004);
        match TcpStream::connect(format!("127.0.0.1:{}", port)) {
            Ok(_) => std::process::exit(0),
            Err(_) => std::process::exit(1),
        }
    }

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
                let peer = stream.peer_addr().map(|a| a.to_string()).unwrap_or_else(|_| "unknown".to_string());
                eprintln!("[CONNECT] New connection from {}", peer);
                let config = Arc::clone(&config);
                thread::spawn(move || {
                    if let Err(e) = handle_connection(stream, config) {
                        eprintln!("[ERROR] Connection error from {}: {}", peer, e);
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
            eprintln!("[AUTH] Invalid callsign: {:?}", callsign);
            writeln!(stream, "Invalid callsign format.")?;
            return Ok(());
        }
    };
    eprintln!("[AUTH] Callsign validated: {}", callsign);

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

    eprintln!("[AUTH] {} agreed to terms", callsign);
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

    // Initialize browser (wrapped in Option for crash recovery)
    eprintln!("[BROWSER] Initializing for {}", callsign);
    let mut browser: Option<BrowserInstance> = match BrowserInstance::new(&callsign) {
        Ok(b) => { eprintln!("[BROWSER] Ready for {}", callsign); Some(b) }
        Err(e) => {
            eprintln!("[BROWSER] Failed to initialize: {}", e);
            writeln!(stream, "Failed to initialize browser: {}", e)?;
            return Ok(());
        }
    };

    // Load portal page
    eprintln!("[PORTAL] Loading {} for {}", config.portal_url, callsign);
    if let Err(e) = load_page(&mut session, &mut browser, &callsign, &config, &logger, &mut stream, &config.portal_url) {
        eprintln!("[PORTAL] Failed for {}: {}", callsign, e);
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

        let trimmed = input.trim();
        eprintln!("[CMD] {} sent: {:?}", callsign, trimmed);

        let command = parse_command(&input);

        match command {
            Command::Quit => {
                eprintln!("[CMD] {} disconnected", callsign);
                writeln!(stream, "Goodbye!")?;
                break;
            }
            Command::Help => {
                writeln!(stream, "\n{}\n", format_help(session.lines_per_page))?;
            }
            Command::Menu => {
                let url = config.portal_url.clone();
                if let Err(e) = load_page(&mut session, &mut browser, &callsign, &config, &logger, &mut stream, &url) {
                    writeln!(stream, "Error loading menu: {}", e)?;
                }
            }
            Command::Back => {
                if let Some(prev_url) = session.previous_url.clone() {
                    if let Err(e) = load_page(&mut session, &mut browser, &callsign, &config, &logger, &mut stream, &prev_url) {
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
                    if let Err(e) = load_page(&mut session, &mut browser, &callsign, &config, &logger, &mut stream, &url) {
                        writeln!(stream, "Error loading link: {}", e)?;
                    }
                } else {
                    writeln!(stream, "Invalid link number.")?;
                }
            }
            Command::NewUrl(url) => {
                if let Err(e) = load_page(&mut session, &mut browser, &callsign, &config, &logger, &mut stream, &url) {
                    writeln!(stream, "Error loading URL: {}", e)?;
                }
            }
            Command::Search(query) => {
                let encoded_query = urlencoding::encode(&query);
                let url = format!("https://en.wikipedia.org/wiki/Special:Search?search={}", encoded_query);
                if let Err(e) = load_page(&mut session, &mut browser, &callsign, &config, &logger, &mut stream, &url) {
                    writeln!(stream, "Error searching: {}", e)?;
                }
            }
            Command::FillInput(num, value) => {
                if let Some(url) = session.current_url.clone() {
                    if let Some(field) = session.inputs.iter().find(|f| f.index == num) {
                        let needs_value = !matches!(field.kind, InputKind::Checkbox { .. });
                        if needs_value && value.is_none() {
                            writeln!(stream, "Input {} requires a value. Use: I {} <value>", num, num)?;
                        } else {
                            writeln!(stream, "Submitting...")?;
                            stream.flush()?;
                            // Try interaction, restart browser on crash
                            let result = if let Some(ref b) = browser {
                                b.interact_with_input(&url, num, value.as_deref())
                            } else {
                                Err(packet_browser::browser::BrowserError::BrowserCrashed)
                            };
                            match result {
                                Ok(content) => {
                                    session.previous_url = Some(url);
                                    session.links = content.links;
                                    session.inputs = content.inputs;
                                    session.page_content = content.text;
                                    display_content(&mut session, &mut stream)?;
                                }
                                Err(e) => {
                                    // Check if this is a connection error (browser crashed)
                                    let err_str = e.to_string();
                                    if err_str.contains("connection is closed") || err_str.contains("BrowserCrashed") {
                                        eprintln!("[BROWSER] Chrome crashed, restarting for {}", callsign);
                                        writeln!(stream, "Browser restarting, please try again...")?;
                                        browser = BrowserInstance::new(&callsign).ok();
                                    } else {
                                        writeln!(stream, "Failed: {}", e)?;
                                    }
                                }
                            }
                        }
                    } else {
                        writeln!(stream, "No input {} on this page.", num)?;
                    }
                } else {
                    writeln!(stream, "No page loaded.")?;
                }
            }
            Command::Unknown(cmd) => {
                eprintln!("[CMD] {} unknown command: {:?}", callsign, cmd);
                writeln!(stream, "Unknown command: '{}'. Type H for help.", cmd)?;
            }
        }
    }

    eprintln!("[CONNECT] Session ended for {}", callsign);
    Ok(())
}

fn load_page(
    session: &mut Session,
    browser: &mut Option<BrowserInstance>,
    callsign: &str,
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

    // Fetch page (with crash recovery)
    writeln!(stream, "Loading...")?;
    stream.flush()?;

    let page_content = loop {
        let b = match browser.as_ref() {
            Some(b) => b,
            None => {
                // Browser not available, try to restart
                eprintln!("[BROWSER] No browser instance, creating for {}", callsign);
                *browser = BrowserInstance::new(callsign).ok();
                if browser.is_none() {
                    writeln!(stream, "Failed to start browser")?;
                    return Ok(());
                }
                continue;
            }
        };

        match b.fetch_page(url) {
            Ok(content) => break content,
            Err(e) => {
                let err_str = e.to_string();
                // Check if browser crashed (WebSocket closed)
                if err_str.contains("connection is closed") {
                    eprintln!("[BROWSER] Chrome crashed, restarting for {}", callsign);
                    writeln!(stream, "Browser restarting...")?;
                    stream.flush()?;
                    *browser = BrowserInstance::new(callsign).ok();
                    if browser.is_none() {
                        writeln!(stream, "Failed to restart browser")?;
                        return Ok(());
                    }
                    // Retry the fetch
                    continue;
                }
                // Other error, don't retry
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

    // VT52-style layout: 25 rows total
    // - Lines 1-21: Content (or fewer if page is short)
    // - Line 22: Links summary (if any)
    // - Line 23: Inputs summary (if any)
    // - Line 24: Help footer
    // - Line 25: Pagination prompt or command prompt

    // Calculate effective content lines (leave room for footer elements)
    let footer_lines = 3; // links + help + prompt
    let effective_lines = session.lines_per_page.saturating_sub(footer_lines).max(5);

    if session.full_page_mode {
        for line in &session.page_content {
            writeln!(stream, "{}", line)?;
        }
        // Show footer at end
        display_footer(session, stream)?;
    } else {
        let pages = paginate(&session.page_content, effective_lines);
        let total_pages = pages.len();

        for (page_num, page) in pages.iter().enumerate() {
            // Content
            for line in page {
                writeln!(stream, "{}", line)?;
            }

            // Always show footer elements after each page
            display_footer(session, stream)?;

            // Pagination prompt (not on last page)
            if page_num < total_pages - 1 {
                write!(stream, "[Page {}/{}] ENTER=more Q=stop: ", page_num + 1, total_pages)?;
                stream.flush()?;

                let mut reader = BufReader::new(stream.try_clone()?);
                let mut input = String::new();
                reader.read_line(&mut input)?;

                if input.trim().to_lowercase() == "q" {
                    break;
                }
                writeln!(stream)?; // Blank line before next page
            }
        }
    }

    Ok(())
}

/// Display the footer section: links summary, inputs, and help text
fn display_footer(session: &Session, stream: &mut TcpStream) -> std::io::Result<()> {
    // Links summary (show first 5 links)
    let links_line = format_links_summary(&session.links, 5);
    if !links_line.is_empty() {
        writeln!(stream, "{}", links_line)?;
    }

    // Full inputs section
    for line in format_inputs_section(&session.inputs) {
        writeln!(stream, "{}", line)?;
    }

    // Help footer - always visible
    writeln!(stream, "{}", format_page_footer())?;

    Ok(())
}
