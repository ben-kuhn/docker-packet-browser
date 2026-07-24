use packet_browser_client::cache;
use packet_browser_client::config;
use packet_browser_client::proxy;
use packet_browser_client::state;
use packet_browser_client::transport;

use config::CliArgs;
use proxy::{AppContext, HostAllowlist};
use state::create_shared_state;
use std::sync::Arc;
use std::time::Duration;
use thiserror::Error;
use tokio::sync::broadcast;

#[derive(Error, Debug)]
pub enum ClientError {
    #[error("AGWPE error: {0}")]
    Agwpe(#[from] transport::agwpe::AgwpeError),
    #[error("Configuration error: {0}")]
    Config(#[from] config::ConfigError),
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

#[tokio::main]
async fn main() -> Result<(), ClientError> {
    let cli = CliArgs::parse();

    let log_level = match cli.verbosity {
        0 => tracing::Level::WARN,
        1 => tracing::Level::INFO,
        2 => tracing::Level::DEBUG,
        _ => tracing::Level::TRACE,
    };

    tracing_subscriber::fmt()
        .with_max_level(log_level)
        .init();

    let config = cli.resolve_config()?;
    let listen_addr = cli.listen_addr.clone();

    let shared_state = create_shared_state(config.clone());
    let (log_tx, _) = broadcast::channel::<state::DebugLogEntry>(256);

    // Build the initial transport based on the configured default. The active
    // transport can still be swapped at runtime from the web UI (see
    // /api/connect), so the choice here only affects the startup auto-connect.
    let default_kind = config.transport.default;
    let initial_transport: Box<dyn transport::Transport> = match default_kind {
        transport::TransportKind::Ax25 => {
            let mut t = transport::agwpe::AgwpeTransport::new();
            t.attach_state(shared_state.clone(), log_tx.clone());
            Box::new(t)
        }
        transport::TransportKind::VaraFm | transport::TransportKind::VaraHf => {
            Box::new(transport::vara::VaraTransport::new())
        }
    };
    let agwpe_manager = transport::TransportManager::spawn(initial_transport, shared_state.clone(), log_tx.clone(), config.connection.response_timeout_secs);

    // Auto-connect to the configured default modem on startup. Failures are
    // non-fatal: the user can still open the web UI and configure/connect the
    // modem from there.
    if config.my_callsign.is_empty() {
        tracing::info!("No callsign configured; skipping modem auto-connect (configure via the web UI)");
    } else {
        let modem_cfg = transport::TransportConfig {
            kind: default_kind,
            agwpe: transport::AgwpeParams {
                host: config.agwpe_host.clone(),
                port: config.agwpe_port,
            },
            vara: transport::VaraParams {
                cmd_host: config.vara.cmd_host.clone(),
                cmd_port: config.vara.cmd_port,
                data_host: config.vara.data_host.clone(),
                data_port: config.vara.data_port,
                mode: config.vara.mode,
                bandwidth: config.vara.bandwidth,
            },
            local_callsign: config.my_callsign.clone(),
        };
        let (label, endpoint) = match default_kind {
            transport::TransportKind::Ax25 => (
                "AGWPE",
                format!("{}:{}", config.agwpe_host, config.agwpe_port),
            ),
            transport::TransportKind::VaraFm | transport::TransportKind::VaraHf => (
                "VARA",
                format!("{}:{}", config.vara.cmd_host, config.vara.cmd_port),
            ),
        };
        tracing::info!("Attempting auto-connect to {} at {}", label, endpoint);
        match agwpe_manager.connect_modem(modem_cfg).await {
            Ok(_) => {
                tracing::info!("Successfully connected to {}", label);
                if matches!(default_kind, transport::TransportKind::Ax25) {
                    if let Err(e) = agwpe_manager.query_ports().await {
                        tracing::warn!("Connected to {} but failed to query ports: {}", label, e);
                    }
                }
            }
            Err(e) => {
                tracing::warn!("Could not auto-connect to {}: {}", label, e);
                tracing::warn!("Start your modem and (re)connect from the web UI, or edit the config.");
            }
        }
    }

    // Bind first so we can derive the actual listening IP for the Host
    // allowlist (matters when --listen-addr uses port 0 or the caller passes
    // a hostname that resolves to a specific interface).
    let listener = tokio::net::TcpListener::bind(&listen_addr).await?;
    let bound = listener.local_addr().ok();
    let listen_ip = bound
        .map(|a| a.ip())
        .unwrap_or(std::net::IpAddr::V4(std::net::Ipv4Addr::LOCALHOST));

    let host_allowlist = HostAllowlist::new(listen_ip, cli.allowed_hosts.clone());

    let cache_max_ttl = Duration::from_secs(config.cache.max_ttl_seconds);
    let cache = if config.cache.enabled {
        match config
            .cache
            .effective_dir()
            .map_err(|e| e.to_string())
            .and_then(|d| {
                crate::cache::Cache::open(&d, config.cache.max_bytes, cache_max_ttl)
                    .map_err(|e| e.to_string())
            }) {
            Ok(c) => Some(std::sync::Arc::new(c)),
            Err(e) => {
                tracing::warn!("cache disabled for this session: {}", e);
                None
            }
        }
    } else {
        None
    };

    let ctx = Arc::new(AppContext {
        state: shared_state,
        agwpe: tokio::sync::Mutex::new(agwpe_manager),
        log_tx,
        host_allowlist,
        cache,
        cache_max_ttl,
        config: config.clone(),
    });

    let app = proxy::create_router(ctx);

    print_startup_banner(&listen_addr, bound.as_ref());

    tracing::info!("Packet browser client starting");
    tracing::info!("Listening on http://{}", listen_addr);
    tracing::info!("AGWPE: {}:{}", config.agwpe_host, config.agwpe_port);
    tracing::info!("My callsign: {}", config.my_callsign);

    if !cli.dont_launch_browser {
        if let Some(url) = connect_url(bound.as_ref(), &listen_addr) {
            tokio::spawn(async move {
                if let Err(e) = launch_browser(&url) {
                    tracing::warn!("Failed to open browser at {}: {}", url, e);
                } else {
                    tracing::info!("Opened {} in default browser", url);
                }
            });
        }
    }

    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await?;

    Ok(())
}

/// Build a browser-usable URL for the connect page. A non-loopback wildcard
/// bind (0.0.0.0 / ::) is rewritten to loopback so the launched browser hits
/// the local process rather than trying to route to the wildcard address.
fn connect_url(bound: Option<&std::net::SocketAddr>, listen_addr: &str) -> Option<String> {
    let addr = bound.copied().or_else(|| listen_addr.parse().ok())?;
    let port = addr.port();
    let host = match addr.ip() {
        std::net::IpAddr::V4(v4) if v4.is_unspecified() => "127.0.0.1".to_string(),
        std::net::IpAddr::V6(v6) if v6.is_unspecified() => "[::1]".to_string(),
        std::net::IpAddr::V6(v6) => format!("[{}]", v6),
        std::net::IpAddr::V4(v4) => v4.to_string(),
    };
    Some(format!("http://{}:{}/connect", host, port))
}

fn launch_browser(url: &str) -> std::io::Result<()> {
    #[cfg(target_os = "linux")]
    let (cmd, args): (&str, &[&str]) = ("xdg-open", &[]);
    #[cfg(target_os = "macos")]
    let (cmd, args): (&str, &[&str]) = ("open", &[]);
    #[cfg(target_os = "windows")]
    let (cmd, args): (&str, &[&str]) = ("cmd", &["/C", "start", ""]);

    std::process::Command::new(cmd)
        .args(args)
        .arg(url)
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .spawn()
        .map(|_| ())
}

fn print_startup_banner(listen_addr: &str, bound: Option<&std::net::SocketAddr>) {
    // Goes through println! (not tracing) so it shows at any verbosity.
    let version = env!("CARGO_PKG_VERSION");
    let bar = "=".repeat(60);
    // Prefer the address we actually bound to (resolves :0 to a real port).
    let display = bound.map(|a| a.to_string()).unwrap_or_else(|| listen_addr.to_string());

    println!();
    println!("{}", bar);
    println!("  Packet Browser Client v{}", version);
    println!();
    println!("  Open http://{} in your browser", display);

    if let Some(addr) = bound {
        let ip = addr.ip();
        if !ip.is_loopback() {
            println!();
            println!("  WARNING: bound to {} (non-loopback address).", ip);
            println!("           Anyone who can reach this host on the network");
            println!("           can use this proxy and change its configuration.");
            println!("           Use --listen-addr 127.0.0.1:PORT to restrict it");
            println!("           to this machine only.");
        }
    }

    println!("{}", bar);
    println!();
}

async fn shutdown_signal() {
    tokio::signal::ctrl_c()
        .await
        .expect("Failed to install Ctrl+C handler");
    tracing::info!("Shutting down...");
}
