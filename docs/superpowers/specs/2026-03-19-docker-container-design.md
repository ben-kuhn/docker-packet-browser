# Docker Packet Browser Container Design Spec

**Date:** 2026-03-19
**Status:** Draft
**Author:** Design collaboration with KU0HN

## Overview

Package the PE1RRR packet radio browser into a secure, portable NixOS-based Docker container. The container provides a text-based web browsing interface for packet radio users connecting through BPQ, with comprehensive logging, content filtering, and security hardening.

## Goals

- Portable, reproducible container deployment
- Comprehensive activity logging for compliance
- Content filtering for amateur radio regulations
- Security hardening to prevent escape/abuse
- Easy integration with BPQ (host or containerized)

## Non-Goals

- Including BPQ inside this container
- GUI or graphical rendering

---

## 1. Container Architecture

**Base Image:** NixOS minimal

**Components installed via Nix:**
- `lynx` - HTML to text conversion
- `dumb-init` or similar - minimal init process (PID 1)
- `logrotate` - log rotation (optional feature)
- `packet-browser` - Rust binary (port of browse.sh logic)

**What's NOT included:**
- No bash, sh, zsh, or any shell
- No coreutils beyond absolute minimum
- No package managers
- No compilers or interpreters

**Rust binary (`packet-browser`):**
- Statically compiled, single binary
- Ports all browse.sh functionality
- Invokes Lynx for HTML-to-text conversion
- Handles user input, pagination, link navigation
- Manages logging, filtering, timeout

**Startup flow:**
1. Container starts with dumb-init as PID 1
2. dumb-init directly executes the browser script via exec
3. Script runs as non-root user `browser` (UID 1000)
4. When script exits, container exits

**Filesystem:**
- Root filesystem: read-only
- `/var/log/packet-browser`: Docker volume mount (host-accessible logs)
- `/etc/hosts`: Docker volume/bind mount (blocklist + admin overrides)
- `/tmp`: writable tmpfs (RAM disk, size-limited)

---

## 2. Security Model

**User isolation:**
- Browser runs as non-root user `browser` (UID 1000)
- No sudo, no setuid binaries

**Capability dropping:**
- Drop all Linux capabilities except minimal set needed for network

**No escape path:**
- No shell binaries in container
- If user kills/escapes browse script, they face a dead end
- Container exits when script exits (no orphan processes)

**URL restrictions (hardcoded):**
- Block protocols: `file://`, `ftp://`, `gopher://`, `mailto://`

**URL restrictions (configurable, defaults below):**
- Block hosts: `localhost`, `127.0.0.1`, `::1`
- Block ranges: `10.0.0.0/8`, `172.16.0.0/12`, `192.168.0.0/16`, `169.254.0.0/16`
- Operator can override to allow specific local services

**Session timeout:**
- Configurable idle timeout (default: 10 minutes)
- If browser script is killed or hangs, init detects and exits container
- Clean exit returns user to BPQ node
- Prevents orphan connections consuming resources

**Read-only root:**
- Container filesystem is read-only
- Prevents any persistent modifications

---

## 3. Logging System

**What gets logged:**
- Timestamp (ISO 8601 format)
- User callsign
- Requested URL
- Response status (success, blocked, error)
- Block reason (if filtered)

**Log format (structured JSON):**
```json
{"ts":"2026-03-19T14:32:01Z","call":"W1ABC","url":"https://example.com","status":"ok"}
{"ts":"2026-03-19T14:32:45Z","call":"W1ABC","url":"https://blocked.com","status":"blocked","reason":"dns_filter"}
```

**Storage:**
- Primary: `/var/log/packet-browser/access.log` (Docker volume, host-accessible)
- Optional: forward to external syslog (configurable)

**Log rotation (optional, in-container):**
- Logrotate included in container
- Enabled via env var (default: enabled)
- Policy: daily rotation, gzip compression, 30 day retention

**User acknowledgment flow:**
1. User connects, sees welcome banner
2. Banner states: "All activity is logged including your callsign"
3. User must type `AGREE` (or similar) to proceed
4. Agreement logged as first entry for session
5. If user doesn't agree, session ends cleanly

---

## 4. Content Filtering

### Layer 1: DNS-based filtering (external)

- Configurable DNS servers (default: OpenDNS Family Shield)
- Filtering DNS returns block pages for prohibited content
- Detect block pages via response signatures/patterns

### Layer 2: Local blocklist filtering (hosts file)

- Container fetches blocklists on startup and periodically
- Writes blocked domains to `/etc/hosts` (resolving to `127.0.0.2`)
- `/etc/hosts` mounted as volume/bind mount
- Admin can manually edit from host side if needed

**Refresh approach:**
1. Read current hosts file
2. Strip all lines between `# BLOCKLIST-MANAGED START` and `# BLOCKLIST-MANAGED END`
3. Fetch fresh blocklists
4. Append new managed block with markers
5. Write updated hosts file

**Hosts file structure:**
```
# System entries
127.0.0.1 localhost

# Admin custom entries (always preserved)
127.0.0.2 my-custom-block.com

# BLOCKLIST-MANAGED START
127.0.0.2 ads.example.com
127.0.0.2 tracker.example.com
# BLOCKLIST-MANAGED END
```

Admin entries outside the markers are never touched. Everything inside the markers gets replaced on each refresh.

---

## 5. Configuration

All configuration via environment variables:

| Variable | Default | Description |
|----------|---------|-------------|
| `LISTEN_PORT` | `63004` | Port the service listens on |
| `PORTAL_URL` | `http://matrix.ehvairport.com/~bpq/` | Default home page |
| `IDLE_TIMEOUT_MINUTES` | `10` | Session idle timeout |
| `DNS_SERVERS` | `208.67.222.123,208.67.220.123` | Filtering DNS (OpenDNS Family Shield) |
| `BLOCKED_RANGES` | `127.0.0.0/8,10.0.0.0/8,172.16.0.0/12,192.168.0.0/16,169.254.0.0/16` | SSRF prevention (configurable) |
| `BLOCKLIST_URLS` | *(default ad/tracking lists)* | URLs to fetch blocklists from |
| `BLOCKLIST_REFRESH_HOURS` | `24` | How often to refresh blocklists |
| `BLOCKLIST_ENABLED` | `true` | Enable/disable local blocklist |
| `LOG_ROTATE_ENABLED` | `true` | Enable/disable log rotation |
| `LOG_RETAIN_DAYS` | `30` | Log retention period |
| `SYSLOG_ENABLED` | `false` | Enable/disable syslog forwarding |
| `SYSLOG_HOST` | *(empty)* | Optional syslog server |
| `SYSLOG_PORT` | `514` | Syslog port |
| `LINES_PER_PAGE` | `15` | Default pagination |

**Volume mounts:**

| Mount | Purpose |
|-------|---------|
| `/var/log/packet-browser` | Logs (host-accessible) |
| `/etc/hosts` | Blocklist + admin overrides |

---

## 6. Network & Integration

**Docker Compose deployment (primary method):**

BPQ may be in same compose file or external.

**Scenario A - BPQ external (on host or elsewhere):**
```yaml
services:
  packet-browser:
    ports:
      - "127.0.0.1:63004:63004"
```

**Scenario B - BPQ in same compose file:**
```yaml
services:
  packet-browser:
    # No port expose needed, use internal network
  bpq:
    # Connects to packet-browser:63004
```

**Default port binding:**
- Binds to `127.0.0.1:63004` (loopback only)
- Not accessible from LAN by default
- Admin can change to `0.0.0.0:63004:63004` if needed

**BPQ Configuration (bpq32.cfg):**
```
PORT
 ...
 APPLICATION 1,WEB,C 3 HOST 0 S CONTAINERIP 63004
```

**Connection flow:**
1. User connects to BPQ node
2. User types `WEB` (or configured command)
3. BPQ opens TCP connection to container:63004
4. Container spawns browser session, receives callsign from BPQ
5. User interacts via text commands
6. On disconnect/timeout, container closes connection

**Callsign passing:**
- BPQ passes callsign at connection start
- Container validates callsign format before proceeding
- Invalid callsign = session rejected with error

---

## 7. Error Handling

**URL request errors:**
- Let Lynx display errors natively
- No interception or custom handling
- User sees what Lynx shows

**Session errors:**
- Invalid callsign: "Invalid callsign format. Disconnecting."
- Acknowledgment refused: "You must agree to proceed. Disconnecting."
- Idle timeout: "Session timed out due to inactivity."

**Container self-healing:**
- Hosts file missing managed section: add markers
- Blocklist fetch fails: retry, then use cached/bundled
- Permissions wrong: attempt to fix, log if unable
- Log errors but keep operating when possible

---

## 8. Build & Packaging

**NixOS container build:**
- Nix flake defines container image
- Declarative, reproducible builds
- All dependencies pinned via flake.lock

**Build artifacts:**
- Docker image (OCI format)
- Can push to registry or load locally

**Files to create:**
- `flake.nix` - Nix flake definition
- `flake.lock` - Pinned dependencies
- `docker-compose.yml` - Example deployment
- `src/` - Rust source code for packet-browser binary
- `Cargo.toml` - Rust package manifest
- `README.md` - Updated documentation
- `.github/workflows/build.yml` - CI/CD pipeline

**Local build command:**
```
nix build .#docker-image
docker load < result
```

**GitHub Container Registry:**
- Automated build on push/tag via GitHub Actions
- Publish to `ghcr.io/[owner]/packet-browser`
- Push to main: build and push `latest`
- Tag (e.g., `v1.0.0`): build and push version tag

**Users can:**
- Build locally with Nix
- Pull pre-built from `ghcr.io`

---

## Technical Constraints

- 80-column terminal maximum
- No arrow keys or tab (single character commands only)
- 300-1200 baud connections - every byte counts
- Numbered links for navigation (e.g., `[1]`, `[2]`)
- No ASCII art or image rendering

## Text Browser

Lynx is retained for HTML-to-text conversion:
- Proven in this application
- Lightweight
- Clean text output without ASCII art
- Compatible with existing numbered-link navigation paradigm

---

## Future Enhancements

**Headless browser rendering:**
- Add optional headless Chrome/Firefox backend for JavaScript-heavy sites
- Render page fully, then extract text for display
- Would improve compatibility with modern web applications
- Trade-off: heavier container, increased attack surface
