use configparser::ini::Ini;
use std::fs;
use std::path::PathBuf;
use thiserror::Error;
use crate::transport::{TransportKind, VaraBandwidth, VaraMode};

/// The two RF-mode-specific bandwidth defaults. VARA HF/Mercury typically
/// runs one of the Bw* variants; VARA FM uses V(N|W)arrow.
const DEFAULT_HF_BW: VaraBandwidth = VaraBandwidth::Bw500;
const DEFAULT_FM_BW: VaraBandwidth = VaraBandwidth::VWide;

#[derive(Error, Debug)]
pub enum ConfigError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Parse error: {0}")]
    Parse(String),
    #[error("Config directory not found")]
    NoConfigDir,
}

#[derive(Debug, Clone)]
pub struct ConnectionConfig {
    pub response_timeout_secs: u64,
    pub auto_reconnect: bool,
}

impl Default for ConnectionConfig {
    fn default() -> Self {
        Self {
            response_timeout_secs: 30,
            auto_reconnect: true,
        }
    }
}

#[derive(Debug, Clone)]
pub struct CacheSection {
    pub enabled: bool,
    pub max_bytes: u64,
    pub max_ttl_seconds: u64,
    pub dir: Option<PathBuf>,
}

impl Default for CacheSection {
    fn default() -> Self {
        Self {
            enabled: true,
            max_bytes: 209_715_200, // 200 MiB
            max_ttl_seconds: 86_400,
            dir: None,
        }
    }
}

impl CacheSection {
    pub fn effective_dir(&self) -> Result<PathBuf, ConfigError> {
        if let Some(d) = &self.dir {
            return Ok(d.clone());
        }
        let cache_root = dirs::cache_dir().ok_or(ConfigError::NoConfigDir)?;
        Ok(cache_root.join("packet-browser"))
    }
}

#[derive(Debug, Clone)]
pub struct TransportSection {
    pub default: TransportKind,
}

impl Default for TransportSection {
    fn default() -> Self {
        Self { default: TransportKind::Ax25 }
    }
}

/// One VARA modem endpoint — the network settings needed to talk to a single
/// running VARA HF/Mercury or VARA FM instance. `[vara_hf]` and `[vara_fm]`
/// each deserialize into one of these.
#[derive(Debug, Clone)]
pub struct VaraEndpoint {
    pub cmd_host: String,
    pub cmd_port: u16,
    pub data_host: String,
    pub data_port: u16,
    pub bandwidth: VaraBandwidth,
}

impl VaraEndpoint {
    fn default_hf() -> Self {
        Self {
            cmd_host: "127.0.0.1".to_string(),
            cmd_port: 8300,
            data_host: "127.0.0.1".to_string(),
            data_port: 8301,
            bandwidth: DEFAULT_HF_BW,
        }
    }

    fn default_fm() -> Self {
        Self {
            cmd_host: "127.0.0.1".to_string(),
            cmd_port: 8400,
            data_host: "127.0.0.1".to_string(),
            data_port: 8401,
            bandwidth: DEFAULT_FM_BW,
        }
    }
}

/// Holds both VARA endpoints. Operators commonly run VARA HF/Mercury and
/// VARA FM on the same host but on different TCP port pairs, so we keep them
/// entirely independent.
#[derive(Debug, Clone)]
pub struct VaraSection {
    pub hf: VaraEndpoint,
    pub fm: VaraEndpoint,
}

impl VaraSection {
    /// Return the endpoint that matches the requested VARA mode.
    pub fn for_mode(&self, mode: VaraMode) -> &VaraEndpoint {
        match mode {
            VaraMode::Hf => &self.hf,
            VaraMode::Fm => &self.fm,
        }
    }

    /// Same as [`for_mode`] but keyed off the transport kind. AX.25 has no
    /// VARA endpoint; callers must not invoke this for `TransportKind::Ax25`.
    pub fn for_transport(&self, kind: TransportKind) -> Option<(&VaraEndpoint, VaraMode)> {
        match kind {
            TransportKind::VaraHf => Some((&self.hf, VaraMode::Hf)),
            TransportKind::VaraFm => Some((&self.fm, VaraMode::Fm)),
            TransportKind::Ax25 => None,
        }
    }
}

impl Default for VaraSection {
    fn default() -> Self {
        Self {
            hf: VaraEndpoint::default_hf(),
            fm: VaraEndpoint::default_fm(),
        }
    }
}

fn save_vara_endpoint(ini: &mut Ini, section: &str, ep: &VaraEndpoint) {
    ini.set(section, "cmd_host", Some(ep.cmd_host.clone()));
    ini.set(section, "cmd_port", Some(ep.cmd_port.to_string()));
    ini.set(section, "data_host", Some(ep.data_host.clone()));
    ini.set(section, "data_port", Some(ep.data_port.to_string()));
    ini.set(section, "bandwidth", Some(ep.bandwidth.to_string()));
}

fn load_vara_endpoint(ini: &Ini, section: &str, defaults: VaraEndpoint) -> VaraEndpoint {
    VaraEndpoint {
        cmd_host: ini.get(section, "cmd_host").unwrap_or(defaults.cmd_host),
        cmd_port: ini
            .get(section, "cmd_port")
            .and_then(|v| v.parse().ok())
            .unwrap_or(defaults.cmd_port),
        data_host: ini.get(section, "data_host").unwrap_or(defaults.data_host),
        data_port: ini
            .get(section, "data_port")
            .and_then(|v| v.parse().ok())
            .unwrap_or(defaults.data_port),
        bandwidth: ini
            .get(section, "bandwidth")
            .map(|v| parse_vara_bandwidth(&v))
            .unwrap_or(defaults.bandwidth),
    }
}

fn parse_vara_bandwidth(s: &str) -> VaraBandwidth {
    match s {
        "vnarrow" => VaraBandwidth::VNarrow,
        "vwide" => VaraBandwidth::VWide,
        "bw250" => VaraBandwidth::Bw250,
        "bw500" => VaraBandwidth::Bw500,
        "bw2300" => VaraBandwidth::Bw2300,
        "bw2750" => VaraBandwidth::Bw2750,
        _ => VaraBandwidth::VWide,
    }
}

#[derive(Debug, Clone)]
pub struct FileConfig {
    pub agwpe_host: String,
    pub agwpe_port: u16,
    pub my_callsign: String,
    pub target_callsign: String,
    pub bpq_command: String,
    pub skip_bpq_app: bool,
    pub cache: CacheSection,
    pub connection: ConnectionConfig,
    pub transport: TransportSection,
    pub vara: VaraSection,
}

impl Default for FileConfig {
    fn default() -> Self {
        Self {
            agwpe_host: "127.0.0.1".to_string(),
            agwpe_port: 8000,
            my_callsign: String::new(),
            target_callsign: String::new(),
            bpq_command: "WEB".to_string(),
            skip_bpq_app: false,
            cache: CacheSection::default(),
            connection: ConnectionConfig::default(),
            transport: TransportSection::default(),
            vara: VaraSection::default(),
        }
    }
}

impl FileConfig {
    pub fn default_path() -> Result<PathBuf, ConfigError> {
        let config_dir = dirs::config_dir().ok_or(ConfigError::NoConfigDir)?;
        Ok(config_dir.join("packet-browser").join("config.ini"))
    }

    pub fn load(path: &PathBuf) -> Result<Self, ConfigError> {
        if !path.exists() {
            return Ok(Self::default());
        }

        let mut ini = Ini::new();
        ini.load(path).map_err(|e| ConfigError::Parse(e))?;

        let agwpe_host = ini
            .get("server", "agwpe_host")
            .unwrap_or_else(|| "127.0.0.1".to_string());

        let agwpe_port = ini
            .get("server", "agwpe_port")
            .and_then(|v| v.parse().ok())
            .unwrap_or(8000);

        let my_callsign = ini
            .get("session", "my_callsign")
            .unwrap_or_default();

        let target_callsign = ini
            .get("session", "target_callsign")
            .unwrap_or_default();

        let bpq_command = ini
            .get("session", "bpq_command")
            .unwrap_or_else(|| "WEB".to_string());

        let skip_bpq_app = ini
            .get("session", "skip_bpq_app")
            .map(|v| v.to_lowercase() == "true")
            .unwrap_or(false);

        let cache_enabled = ini
            .get("cache", "enabled")
            .map(|v| matches!(v.to_lowercase().as_str(), "true" | "1" | "yes" | "on"))
            .unwrap_or(true);
        let cache_max_bytes = ini
            .get("cache", "max_bytes")
            .and_then(|v| v.parse().ok())
            .unwrap_or(209_715_200);
        let cache_max_ttl_seconds = ini
            .get("cache", "max_ttl_seconds")
            .and_then(|v| v.parse().ok())
            .unwrap_or(86_400);
        let cache_dir = ini
            .get("cache", "dir")
            .filter(|s| !s.trim().is_empty())
            .map(PathBuf::from);

        let response_timeout_secs = ini
            .get("connection", "response_timeout_secs")
            .and_then(|v| v.parse().ok())
            .unwrap_or(30);
        let auto_reconnect = ini
            .get("connection", "auto_reconnect")
            .map(|v| matches!(v.to_lowercase().as_str(), "true" | "1" | "yes" | "on"))
            .unwrap_or(true);

        let transport_default = ini
            .get("transport", "default")
            .and_then(|v| v.parse::<TransportKind>().ok())
            .unwrap_or(TransportKind::Ax25);

        let vara = VaraSection {
            hf: load_vara_endpoint(&ini, "vara_hf", VaraEndpoint::default_hf()),
            fm: load_vara_endpoint(&ini, "vara_fm", VaraEndpoint::default_fm()),
        };

        let transport = TransportSection { default: transport_default };

        Ok(Self {
            agwpe_host,
            agwpe_port,
            my_callsign,
            target_callsign,
            bpq_command,
            skip_bpq_app,
            cache: CacheSection {
                enabled: cache_enabled,
                max_bytes: cache_max_bytes,
                max_ttl_seconds: cache_max_ttl_seconds,
                dir: cache_dir,
            },
            connection: ConnectionConfig {
                response_timeout_secs,
                auto_reconnect,
            },
            transport,
            vara,
        })
    }

    pub fn save(&self, path: &PathBuf) -> Result<(), ConfigError> {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }

        let mut ini = Ini::new();

        ini.set("server", "agwpe_host", Some(self.agwpe_host.clone()));
        ini.set("server", "agwpe_port", Some(self.agwpe_port.to_string()));
        ini.set("session", "my_callsign", Some(self.my_callsign.clone()));
        ini.set("session", "target_callsign", Some(self.target_callsign.clone()));
        ini.set("session", "bpq_command", Some(self.bpq_command.clone()));
        ini.set("session", "skip_bpq_app", Some(self.skip_bpq_app.to_string()));

        ini.set("cache", "enabled", Some(self.cache.enabled.to_string()));
        ini.set("cache", "max_bytes", Some(self.cache.max_bytes.to_string()));
        ini.set("cache", "max_ttl_seconds", Some(self.cache.max_ttl_seconds.to_string()));
        if let Some(d) = &self.cache.dir {
            ini.set("cache", "dir", Some(d.to_string_lossy().into_owned()));
        }

        ini.set("connection", "response_timeout_secs", Some(self.connection.response_timeout_secs.to_string()));
        ini.set("connection", "auto_reconnect", Some(self.connection.auto_reconnect.to_string()));

        ini.set("transport", "default", Some(self.transport.default.to_string()));

        save_vara_endpoint(&mut ini, "vara_hf", &self.vara.hf);
        save_vara_endpoint(&mut ini, "vara_fm", &self.vara.fm);

        ini.write(path).map_err(|e| ConfigError::Parse(e.to_string()))?;
        Ok(())
    }

    pub fn update_target(&mut self, target: &str) {
        self.target_callsign = target.to_string();
    }
}

#[derive(Debug, Clone)]
pub struct CliArgs {
    pub config_path: Option<PathBuf>,
    pub agwpe_host: Option<String>,
    pub agwpe_port: Option<u16>,
    pub listen_addr: String,
    pub bpq_command: Option<String>,
    pub verbosity: u8,
    pub allowed_hosts: Vec<String>,
    pub dont_launch_browser: bool,
}

impl CliArgs {
    pub fn parse() -> Self {
        use clap::Parser;

        #[derive(Parser)]
        #[command(name = "packet-browser-client")]
        #[command(about = "Packet radio web browser client")]
        #[command(version)]
        struct Args {
            #[arg(short, long, help = "Configuration file (INI format)")]
            config: Option<PathBuf>,

            #[arg(long, help = "AGWPE host (default: 127.0.0.1)")]
            agwpe_host: Option<String>,

            #[arg(long, help = "AGWPE port (default: 8000)")]
            agwpe_port: Option<u16>,

            #[arg(long, default_value = "127.0.0.1:8088", help = "Web proxy listen address")]
            listen_addr: String,

            #[arg(long, default_value = "WEB", help = "BPQ APPLICATION command")]
            bpq_command: String,

            #[arg(short, long, action = clap::ArgAction::Count, help = "Verbosity level (-v, -vv, -vvv)")]
            verbose: u8,

            #[arg(
                long,
                value_delimiter = ',',
                help = "Extra hostnames to accept in the Host header (comma-separated). Useful for mDNS names like 'raspberrypi.local' when binding to a LAN interface. Loopback and LAN IP literals are already accepted based on --listen-addr."
            )]
            allowed_hosts: Vec<String>,

            #[arg(long, help = "Don't open the default web browser to the connect page on startup")]
            dont_launch_browser: bool,
        }

        let args = Args::parse();

        Self {
            config_path: args.config,
            agwpe_host: args.agwpe_host,
            agwpe_port: args.agwpe_port,
            listen_addr: args.listen_addr,
            bpq_command: Some(args.bpq_command),
            verbosity: args.verbose,
            allowed_hosts: args
                .allowed_hosts
                .into_iter()
                .map(|s| s.trim().to_ascii_lowercase())
                .filter(|s| !s.is_empty())
                .collect(),
            dont_launch_browser: args.dont_launch_browser,
        }
    }

    pub fn resolve_config(&self) -> Result<FileConfig, ConfigError> {
        let path = match self.config_path.clone() {
            Some(p) => p,
            None => FileConfig::default_path()?,
        };

        let mut config = FileConfig::load(&path)?;

        if let Some(host) = &self.agwpe_host {
            config.agwpe_host = host.clone();
        }

        if let Some(port) = self.agwpe_port {
            config.agwpe_port = port;
        }

        if let Some(cmd) = &self.bpq_command {
            config.bpq_command = cmd.clone();
        }

        Ok(config)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_config_default() {
        let config = FileConfig::default();
        assert_eq!(config.agwpe_host, "127.0.0.1");
        assert_eq!(config.agwpe_port, 8000);
        assert_eq!(config.my_callsign, "");
        assert_eq!(config.target_callsign, "");
        assert_eq!(config.bpq_command, "WEB");
    }

    #[test]
    fn test_config_save_load_roundtrip() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("config.ini");

        let config = FileConfig {
            agwpe_host: "192.168.1.100".to_string(),
            agwpe_port: 9000,
            my_callsign: "N0CALL".to_string(),
            target_callsign: "NODE1".to_string(),
            bpq_command: "BROWSE".to_string(),
            skip_bpq_app: false,
            cache: CacheSection::default(),
            connection: ConnectionConfig::default(),
            transport: TransportSection::default(),
            vara: VaraSection::default(),
        };

        config.save(&path).unwrap();
        let loaded = FileConfig::load(&path).unwrap();

        assert_eq!(loaded.agwpe_host, "192.168.1.100");
        assert_eq!(loaded.agwpe_port, 9000);
        assert_eq!(loaded.my_callsign, "N0CALL");
        assert_eq!(loaded.target_callsign, "NODE1");
        assert_eq!(loaded.bpq_command, "BROWSE");
    }

    #[test]
    fn test_config_load_missing_file() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("nonexistent.ini");

        let config = FileConfig::load(&path).unwrap();
        assert_eq!(config.agwpe_host, "127.0.0.1");
        assert_eq!(config.agwpe_port, 8000);
    }

    #[test]
    fn test_config_update_target() {
        let mut config = FileConfig::default();
        assert_eq!(config.target_callsign, "");

        config.update_target("NEWNODE");
        assert_eq!(config.target_callsign, "NEWNODE");
    }

    #[test]
    fn test_cli_args_override() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("config.ini");

        let mut config = FileConfig::default();
        config.agwpe_host = "10.0.0.1".to_string();
        config.agwpe_port = 7000;
        config.save(&path).unwrap();

        let cli = CliArgs {
            config_path: Some(path.clone()),
            agwpe_host: Some("192.168.1.1".to_string()),
            agwpe_port: None,
            listen_addr: "127.0.0.1:8080".to_string(),
            bpq_command: Some("WEB".to_string()),
            verbosity: 0,
            allowed_hosts: vec![],
            dont_launch_browser: true,
        };

        let resolved = cli.resolve_config().unwrap();
        assert_eq!(resolved.agwpe_host, "192.168.1.1");
        assert_eq!(resolved.agwpe_port, 7000);
    }

    #[test]
    fn cache_defaults_are_applied_when_section_absent() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("config.ini");
        let cfg = FileConfig::default();
        cfg.save(&path).unwrap();
        let loaded = FileConfig::load(&path).unwrap();
        assert!(loaded.cache.enabled);
        assert_eq!(loaded.cache.max_bytes, 209_715_200);
        assert_eq!(loaded.cache.max_ttl_seconds, 86_400);
        assert!(loaded.cache.dir.is_none());
    }

    #[test]
    fn cache_section_roundtrip() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("config.ini");
        let cfg = FileConfig {
            cache: CacheSection {
                enabled: false,
                max_bytes: 42,
                max_ttl_seconds: 7,
                dir: Some(std::path::PathBuf::from("/tmp/pb-cache")),
            },
            ..FileConfig::default()
        };
        cfg.save(&path).unwrap();
        let loaded = FileConfig::load(&path).unwrap();
        assert!(!loaded.cache.enabled);
        assert_eq!(loaded.cache.max_bytes, 42);
        assert_eq!(loaded.cache.max_ttl_seconds, 7);
        assert_eq!(
            loaded.cache.dir.as_deref().map(|p| p.to_string_lossy().into_owned()),
            Some("/tmp/pb-cache".to_string())
        );
    }

    #[test]
    fn test_connection_config_defaults() {
        let cfg = FileConfig::default();
        assert_eq!(cfg.connection.response_timeout_secs, 30);
        assert!(cfg.connection.auto_reconnect);
    }

    #[test]
    fn test_connection_config_overrides() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("config.ini");

        let mut ini = Ini::new();
        ini.set("server", "agwpe_host", Some("127.0.0.1".to_string()));
        ini.set("server", "agwpe_port", Some("8000".to_string()));
        ini.set("session", "my_callsign", Some("W1TEST".to_string()));
        ini.set("session", "target_callsign", Some("N0CALL".to_string()));
        ini.set("session", "bpq_command", Some("WEB".to_string()));
        ini.set("session", "skip_bpq_app", Some("false".to_string()));
        ini.set("connection", "response_timeout_secs", Some("15".to_string()));
        ini.set("connection", "auto_reconnect", Some("false".to_string()));
        ini.write(&path).unwrap();

        let loaded = FileConfig::load(&path).unwrap();
        assert_eq!(loaded.connection.response_timeout_secs, 15);
        assert!(!loaded.connection.auto_reconnect);
    }

    #[test]
    fn loads_transport_and_split_vara_sections() {
        let dir = tempdir().unwrap();
        let p = dir.path().join("t.ini");
        std::fs::write(
            &p,
            r#"
[transport]
default = vara_fm

[vara_hf]
cmd_host = 10.0.0.5
cmd_port = 8300
data_host = 10.0.0.5
data_port = 8301
bandwidth = bw2300

[vara_fm]
cmd_host = 10.0.0.6
cmd_port = 8400
data_host = 10.0.0.6
data_port = 8401
bandwidth = vwide
"#,
        )
        .unwrap();

        let cfg = FileConfig::load(&p).unwrap();

        assert_eq!(cfg.transport.default, crate::transport::TransportKind::VaraFm);
        assert_eq!(cfg.vara.hf.cmd_host, "10.0.0.5");
        assert_eq!(cfg.vara.hf.cmd_port, 8300);
        assert_eq!(cfg.vara.hf.bandwidth, crate::transport::VaraBandwidth::Bw2300);
        assert_eq!(cfg.vara.fm.cmd_host, "10.0.0.6");
        assert_eq!(cfg.vara.fm.cmd_port, 8400);
        assert_eq!(cfg.vara.fm.data_port, 8401);
        assert_eq!(cfg.vara.fm.bandwidth, crate::transport::VaraBandwidth::VWide);
    }

    #[test]
    fn missing_transport_section_defaults_to_ax25_with_default_vara_endpoints() {
        let dir = tempdir().unwrap();
        let p = dir.path().join("t.ini");
        std::fs::write(&p, "").unwrap();
        let cfg = FileConfig::load(&p).unwrap();
        assert_eq!(cfg.transport.default, crate::transport::TransportKind::Ax25);
        assert_eq!(cfg.vara.hf.cmd_port, 8300);
        assert_eq!(cfg.vara.fm.cmd_port, 8400);
    }

    #[test]
    fn test_config_save_load_roundtrip_preserves_split_vara() {
        use crate::transport::{TransportKind, VaraBandwidth};

        let dir = tempdir().unwrap();
        let path = dir.path().join("config.ini");

        let cfg = FileConfig {
            agwpe_host: "10.0.0.1".to_string(),
            agwpe_port: 9999,
            my_callsign: "W1TEST".to_string(),
            target_callsign: "N0CALL-9".to_string(),
            bpq_command: "WEB".to_string(),
            skip_bpq_app: false,
            cache: CacheSection::default(),
            connection: ConnectionConfig::default(),
            transport: TransportSection { default: TransportKind::VaraFm },
            vara: VaraSection {
                hf: VaraEndpoint {
                    cmd_host: "10.1.2.3".to_string(),
                    cmd_port: 8305,
                    data_host: "10.1.2.3".to_string(),
                    data_port: 8306,
                    bandwidth: VaraBandwidth::Bw2300,
                },
                fm: VaraEndpoint {
                    cmd_host: "10.4.5.6".to_string(),
                    cmd_port: 8410,
                    data_host: "10.4.5.6".to_string(),
                    data_port: 8411,
                    bandwidth: VaraBandwidth::VNarrow,
                },
            },
        };

        cfg.save(&path).unwrap();
        let loaded = FileConfig::load(&path).unwrap();

        assert_eq!(loaded.transport.default, TransportKind::VaraFm);
        assert_eq!(loaded.vara.hf.cmd_host, "10.1.2.3");
        assert_eq!(loaded.vara.hf.cmd_port, 8305);
        assert_eq!(loaded.vara.hf.data_port, 8306);
        assert_eq!(loaded.vara.hf.bandwidth, VaraBandwidth::Bw2300);
        assert_eq!(loaded.vara.fm.cmd_host, "10.4.5.6");
        assert_eq!(loaded.vara.fm.cmd_port, 8410);
        assert_eq!(loaded.vara.fm.data_port, 8411);
        assert_eq!(loaded.vara.fm.bandwidth, VaraBandwidth::VNarrow);
    }
}
