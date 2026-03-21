# Packet Browser

[![Build and Publish](https://github.com/ben-kuhn/docker-packet-browser/actions/workflows/build.yml/badge.svg)](https://github.com/ben-kuhn/docker-packet-browser/actions/workflows/build.yml)

A secure web browser for packet radio users connecting through BPQ. This containerized implementation provides text-based web browsing with comprehensive logging, content filtering, and security hardening suitable for amateur radio networks.

## Overview

This project modernizes the original PE1RRR packet radio browser (browse.sh) into a secure, containerized deployment. The browser uses headless Chromium to render modern web pages and presents them as paginated text suitable for low-bandwidth packet radio connections.

**Original Work:** Red Tuby PE1RRR (SK) - browse.sh
**License:** GNU General Public License v3

### Key Features

- Full JavaScript support via headless Chromium
- JSON structured logging with callsign tracking
- Multi-layer content filtering (DNS + hosts-based blocklist)
- SSRF prevention for network security
- Session timeout and idle disconnect
- Read-only container filesystem
- Capability-dropping security model
- Docker Compose deployment

## Quick Start

### Using Pre-built Image

Pull from GitHub Container Registry and run with Docker Compose:

```bash
# Create directory and required files
mkdir -p packet-browser/logs
touch packet-browser/hosts

# Create docker-compose.yml (see Configuration section below)
nano docker-compose.yml

# Pull and start
docker compose pull
docker compose up -d
```

### Building from Source

Prerequisites: Nix with flakes enabled

```bash
# Clone repository
git clone https://github.com/ben-kuhn/docker-packet-browser.git
cd docker-packet-browser

# Build Docker image with Nix
nix build .#docker-image

# Load into Docker
docker load < result

# Start with Docker Compose
docker compose up -d
```

## Configuration

### Docker Compose

Create `docker-compose.yml`:

```yaml
version: '3.8'

services:
  packet-browser:
    image: ghcr.io/ben-kuhn/packet-browser:latest
    # Or use: packet-browser:latest (if built locally)

    ports:
      # Bind to loopback only by default (security)
      - "127.0.0.1:63004:63004"

    volumes:
      # Logs - accessible from host
      - ./logs:/var/log/packet-browser
      # Hosts file for blocklist management
      # Note: ./hosts must exist as a file (not directory) before starting
      # Create it with: touch hosts
      - ./hosts:/etc/hosts

    environment:
      # Service configuration
      - LISTEN_PORT=63004
      - PORTAL_URL=http://matrix.ehvairport.com/~bpq/
      - IDLE_TIMEOUT_MINUTES=10
      - LINES_PER_PAGE=15

      # DNS filtering (OpenDNS Family Shield)
      - DNS_SERVERS=208.67.222.123,208.67.220.123

      # SSRF prevention - blocked IP ranges
      # Remove ranges to allow access to local services
      - BLOCKED_RANGES=127.0.0.0/8,10.0.0.0/8,172.16.0.0/12,192.168.0.0/16,169.254.0.0/16

      # Blocklist settings
      - BLOCKLIST_ENABLED=true
      - BLOCKLIST_REFRESH_HOURS=24
      - BLOCKLIST_URLS=https://cdn.jsdelivr.net/gh/hagezi/dns-blocklists@latest/hosts/ultimate.txt

      # Logging
      - LOG_ROTATE_ENABLED=true
      - LOG_RETAIN_DAYS=30
      - SYSLOG_ENABLED=false
      # - SYSLOG_HOST=syslog.example.com
      # - SYSLOG_PORT=514

      # Debug mode - for testing without BPQ
      - DEBUG_MODE=false

    # Security hardening
    read_only: true
    tmpfs:
      - /tmp:size=64M,mode=1777

    cap_drop:
      - ALL
    cap_add:
      - NET_RAW  # Required for DNS

    # Health check
    healthcheck:
      test: ["CMD", "nc", "-z", "localhost", "63004"]
      interval: 30s
      timeout: 5s
      retries: 3

    restart: unless-stopped
```

### Environment Variables

| Variable | Default | Description |
|----------|---------|-------------|
| `LISTEN_PORT` | `63004` | TCP port the service listens on |
| `PORTAL_URL` | `http://matrix.ehvairport.com/~bpq/` | Default home page shown on connect |
| `IDLE_TIMEOUT_MINUTES` | `10` | Session timeout for idle connections |
| `LINES_PER_PAGE` | `15` | Number of lines per page in pagination |
| `DNS_SERVERS` | `208.67.222.123,208.67.220.123` | DNS servers for filtering (OpenDNS Family Shield) |
| `BLOCKED_RANGES` | `127.0.0.0/8,10.0.0.0/8,...` | CIDR ranges blocked for SSRF prevention |
| `BLOCKLIST_ENABLED` | `true` | Enable/disable local hosts-based blocklist |
| `BLOCKLIST_REFRESH_HOURS` | `24` | How often to refresh blocklists from URLs |
| `BLOCKLIST_URLS` | *(empty)* | Comma-separated URLs of hosts-format blocklists (`0.0.0.0 domain.com`) |
| `LOG_ROTATE_ENABLED` | `true` | Enable automatic log rotation |
| `LOG_RETAIN_DAYS` | `30` | Number of days to retain rotated logs |
| `SYSLOG_ENABLED` | `false` | Forward logs to external syslog server |
| `SYSLOG_HOST` | *(empty)* | Syslog server hostname or IP |
| `SYSLOG_PORT` | `514` | Syslog server port |
| `DEBUG_MODE` | `false` | Enable debug mode for testing without BPQ |

## BPQ Integration

### BPQ Configuration

Add the following to your `bpq32.cfg`:

```
# Add port 63004 to CMDPORT list
# Position in list determines HOST number (0-indexed)
CMDPORT=63001,63002,63003,63004

# Define application
# Syntax: APPLICATION <num>,<command>,C <port> HOST <position> S
APPLICATION 4,WEB,C 10 HOST 3 S
```

**Explanation:**
- `CMDPORT=...` - List of TCP ports, comma-separated
- Port 63004 is at position 3 (0-indexed: 63001=0, 63002=1, 63003=2, 63004=3)
- `APPLICATION 4,WEB,C 10 HOST 3 S`
  - `4` - Application number (1-32, must be unique)
  - `WEB` - Command users type to access browser
  - `C 10` - Use BPQ port 10 (adjust to your telnet port)
  - `HOST 3` - Connect to CMDPORT position 3 (port 63004)
  - `S` - Return user to node menu on disconnect (Stay)

### Port Binding

By default, the container binds to `127.0.0.1:63004` (loopback only) for security.

**If BPQ is on the same host:**
```yaml
ports:
  - "127.0.0.1:63004:63004"  # Works for local BPQ
```

**If BPQ is on another host:**
```yaml
ports:
  - "0.0.0.0:63004:63004"  # WARNING: Accessible from network
```

**If BPQ is in the same Docker Compose:**
```yaml
# No port mapping needed - use internal DNS
services:
  bpq:
    # ... BPQ container config
    # Can connect to packet-browser:63004 directly

  packet-browser:
    # No ports: section needed
```

### Connection Flow

1. User connects to BPQ node via radio or telnet
2. User types `WEB` (or configured command)
3. BPQ opens TCP connection to container on port 63004
4. Container receives callsign from BPQ, shows welcome banner
5. User must type `AGREE` to acknowledge logging
6. User browses web via numbered link navigation
7. On exit or timeout, user returns to BPQ node menu

## User Commands

Once connected and agreed to logging, the browser loads the portal page and shows a `Command:` prompt. Type `H` at any time for in-session help.

| Command | Description |
|---------|-------------|
| `N <url>` | Open a URL (e.g. `N https://example.com`) — must include `https://` or `http://` |
| `S <text>` | Search Wikipedia for text |
| `<number>` | Follow a numbered link from the current page |
| `I <n> <text>` | Fill input field n with text and submit the form (e.g. `I 1 hello world`) |
| `L` | List all links on the current page with their numbers |
| `P` | Go back to the previous page |
| `M` | Return to the portal menu page |
| `R` | Redisplay the current page |
| `F` | Toggle between paged and full-page display |
| `OP <n>` | Set lines per page (1-99); `OP` alone shows current setting |
| `H` or `?` | Show help |
| `Q` or `B` | Quit / disconnect |

> **Note:** URLs must include the full scheme (`https://` or `http://`). Typing just `google.com` will not work — use `N https://google.com`.

Each page displays a footer banner listing available commands and any detected input fields (e.g. `[I1 Search]`). Use `I 1 <text>` to fill that field and submit.

## Debug Mode / Testing

To test the browser without BPQ, enable debug mode:

```yaml
environment:
  - DEBUG_MODE=true
```

Then connect via telnet:

```bash
telnet localhost 63004
```

In debug mode:
- You will be prompted to enter a callsign
- Callsign validation still applies (must match amateur radio format)
- All other functionality works identically
- Logging still occurs with the callsign you provide

This is useful for:
- Testing configuration changes
- Verifying blocklists
- Debugging connection issues
- Development and troubleshooting

## Building from Source

### Prerequisites

- Nix package manager with flakes enabled
- Docker or Podman

### Build Process

```bash
# Enable Nix flakes (if not already enabled)
mkdir -p ~/.config/nix
echo "experimental-features = nix-command flakes" >> ~/.config/nix/nix.conf

# Clone repository
git clone https://github.com/ben-kuhn/docker-packet-browser.git
cd docker-packet-browser

# Build the Docker image
nix build .#docker-image

# Load image into Docker
docker load < result

# Verify image is loaded
docker images | grep packet-browser

# Start with Docker Compose
docker compose up -d
```

### Development

Enter the Nix development shell to work on the Rust code:

```bash
nix develop

# Now you have Rust toolchain, Chromium, and dependencies
cargo build
cargo test
cargo run
```

### Building Binary Only

```bash
# Build just the Rust binary (no container)
nix build

# Binary is at ./result/bin/packet-browser
./result/bin/packet-browser
```

## Logging

### Access Logs

All user activity is logged in JSON format to `/var/log/packet-browser/access.log`:

```json
{"ts":"2026-03-19T14:32:01Z","call":"W1ABC","url":"https://example.com","status":"ok"}
{"ts":"2026-03-19T14:32:45Z","call":"W1ABC","url":"https://blocked.com","status":"blocked","reason":"dns_filter"}
```

**Log fields:**
- `ts` - ISO 8601 timestamp
- `call` - User callsign
- `url` - Requested URL
- `status` - Result: `ok`, `blocked`, `error`
- `reason` - Block/error reason (if applicable)

### Log Access

Logs are mounted to `./logs` on the host:

```bash
# View live logs
tail -f logs/access.log

# Parse with jq
cat logs/access.log | jq 'select(.call=="W1ABC")'

# Search for blocked requests
grep '"status":"blocked"' logs/access.log | jq .
```

### Log Rotation

Log rotation is enabled by default and runs daily:
- Logs rotated at midnight
- Compressed with gzip
- Retained for 30 days (configurable)
- Rotation controlled by `LOG_ROTATE_ENABLED` and `LOG_RETAIN_DAYS`

### Syslog Integration

Forward logs to external syslog server:

```yaml
environment:
  - SYSLOG_ENABLED=true
  - SYSLOG_HOST=syslog.example.com
  - SYSLOG_PORT=514
```

Logs are sent to both local file and syslog when enabled.

## Security

### Content Filtering

**Layer 1: DNS Filtering**
- Uses OpenDNS Family Shield by default (208.67.222.123, 208.67.220.123)
- Blocks adult content, phishing, malware sites
- Configurable via `DNS_SERVERS` environment variable
- Alternative: Cloudflare Family (1.1.1.3), Quad9 Family (9.9.9.11)

**Layer 2: Hosts-based Blocklist**
- Container fetches hosts-format blocklists from URLs on startup
- Blocklist URLs must use the hosts file format: `0.0.0.0 domain.com`
- Writes blocked domains to `/etc/hosts` (resolves to 0.0.0.0)
- Refreshes every 24 hours (configurable)
- Admin can manually add custom blocks via the hosts volume

### SSRF Prevention

By default, the following IP ranges are blocked to prevent Server-Side Request Forgery:
- `127.0.0.0/8` - Loopback
- `10.0.0.0/8` - Private network
- `172.16.0.0/12` - Private network
- `192.168.0.0/16` - Private network
- `169.254.0.0/16` - Link-local

To allow access to specific local services, remove ranges from `BLOCKED_RANGES`:

```yaml
environment:
  # Allow access to 192.168.x.x network
  - BLOCKED_RANGES=127.0.0.0/8,10.0.0.0/8,172.16.0.0/12,169.254.0.0/16
```

### Protocol Filtering

The following protocols are always blocked (hardcoded):
- `file://` - Local file access
- `ftp://` - FTP protocol
- `gopher://` - Gopher protocol
- `mailto://` - Email links

Only `http://` and `https://` are permitted.

### Container Hardening

- **Read-only root filesystem** - Prevents persistent modifications
- **No shell binaries** - No escape path for users
- **Capability dropping** - All capabilities dropped except NET_RAW (for DNS)
- **Non-root user** - Runs as UID 1000
- **tmpfs for /tmp** - Size-limited RAM disk (64MB, mode=1777 for non-root write access)
- **Network isolation** - Loopback binding by default

### Session Security

- **Idle timeout** - Default 10 minutes (configurable)
- **Callsign validation** - Amateur radio callsign format required
- **Logging acknowledgment** - Users must agree to logging before access
- **Clean disconnect** - Returns user to BPQ node menu on exit

## Managing Blocklists

### Automatic Updates

Blocklists refresh automatically every 24 hours (configurable via `BLOCKLIST_REFRESH_HOURS`).

### Manual Host Blocking

Edit the hosts file volume to add custom blocks:

```bash
# Edit hosts file
nano hosts

# Add custom blocks using hosts format (will be preserved on refresh)
0.0.0.0 unwanted-site.com
0.0.0.0 another-blocked.com

# Restart container to apply
docker compose restart
```

Custom entries outside the `# BLOCKLIST-MANAGED START/END` markers are preserved during automatic updates.

### Disabling Blocklist

```yaml
environment:
  - BLOCKLIST_ENABLED=false
```

## License

GNU General Public License v3 or later

Original browse.sh: Copyright 2019-2023 Red Tuby PE1RRR
Docker implementation: Copyright 2026 KU0HN

This program is free software: you can redistribute it and/or modify it under the terms of the GNU General Public License as published by the Free Software Foundation, either version 3 of the License, or (at your option) any later version.

This program is distributed in the hope that it will be useful, but WITHOUT ANY WARRANTY; without even the implied warranty of MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the GNU General Public License for more details.

You should have received a copy of the GNU General Public License along with this program. If not, see <http://www.gnu.org/licenses/>.

## Credits

- **Red Tuby PE1RRR** - Original browse.sh implementation (2019-2023)
- **KU0HN** - Docker containerization and Rust port (2026)

## Support

Issues and pull requests: https://github.com/ben-kuhn/docker-packet-browser

For BPQ-specific questions, consult the BPQ32 documentation.
