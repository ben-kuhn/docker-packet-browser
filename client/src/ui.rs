/// Escape a string for safe insertion into HTML element text or attribute values
/// (double-quoted). Covers the OWASP "HTML body" + "HTML attribute" contexts.
pub fn h(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            '\'' => out.push_str("&#39;"),
            _ => out.push(c),
        }
    }
    out
}

/// Make a JSON string safe to inline inside a `<script>` block by re-encoding
/// the few characters that could close the script tag or break JS parsing.
/// Each replacement is still valid JSON, so the runtime value is unchanged.
fn json_for_script(s: &str) -> String {
    s.replace('<', "\\u003c")
        .replace('>', "\\u003e")
        .replace('&', "\\u0026")
        .replace('\u{2028}', "\\u2028")
        .replace('\u{2029}', "\\u2029")
}

pub const CSS: &str = r#"
* { box-sizing: border-box; margin: 0; padding: 0; }
body {
    background: #0c1222;
    color: #f1f5f9;
    font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', sans-serif;
    line-height: 1.6;
    padding: 1em;
    max-width: 900px;
    margin: 0 auto;
}
h1, h2, h3 { margin: 0.5em 0; color: #22d3ee; }
a { color: #22d3ee; }
input, select, button, textarea {
    background: #0f172a;
    color: #f1f5f9;
    border: 1px solid #1e293b;
    padding: 0.5em 0.75em;
    border-radius: 4px;
    font-size: 1em;
    font-family: inherit;
}
input:focus, select:focus, textarea:focus {
    outline: none;
    border-color: #22d3ee;
}
button {
    background: #1e293b;
    cursor: pointer;
    transition: background 0.2s;
}
button:hover { background: #334155; }
button:disabled { opacity: 0.5; cursor: not-allowed; }
button.primary { background: #16a34a; }
button.primary:hover { background: #22c55e; }
button.danger { background: #dc2626; }
button.danger:hover { background: #ef4444; }
.form-group {
    margin-bottom: 1em;
}
.form-group label {
    display: block;
    margin-bottom: 0.25em;
    color: #22d3ee;
    font-size: 0.9em;
}
.form-group input, .form-group select {
    width: 100%;
}
.form-group input[type="checkbox"] {
    width: auto;
    margin: 0.25em 0 0;
}
.modal-backdrop {
    position: fixed;
    inset: 0;
    background: rgba(12, 18, 34, 0.75);
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 1000;
    padding: 1em;
}
.modal {
    background: #0f172a;
    border: 1px solid #1e293b;
    border-radius: 8px;
    padding: 1.25em 1.5em;
    max-width: 640px;
    width: 100%;
    max-height: 90vh;
    overflow-y: auto;
    box-shadow: 0 10px 30px rgba(0, 0, 0, 0.5);
}
.consent-disclaimer {
    background: #0c1222;
    border: 1px solid #1e293b;
    border-radius: 4px;
    padding: 0.75em 1em;
    color: #f1f5f9;
    /* Prose, not code: inherit the page body font instead of forcing a
       monospace stack whose named fonts (SF Mono, Consolas) don't exist on
       Linux and fall back to the ugly default `monospace` glyphs. */
    font-family: inherit;
    font-size: 0.95em;
    line-height: 1.5;
    white-space: pre-wrap;
    word-break: break-word;
    margin: 0.75em 0;
}
.status-badge {
    display: inline-block;
    padding: 0.25em 0.75em;
    border-radius: 12px;
    font-size: 0.85em;
    font-weight: bold;
}
.status-disconnected { background: #450a0a; color: #ef4444; }
.status-modem-connected { background: #052e16; color: #22c55e; }
.status-connecting { background: #422006; color: #fbbf24; }
.status-reconnecting { background: #422006; color: #fbbf24; }
.status-connected { background: #052e16; color: #22c55e; }
.status-error { background: #450a0a; color: #ef4444; }
.btn-row {
    display: flex;
    gap: 0.5em;
    margin-top: 1em;
}
.card {
    background: #0f172a;
    border: 1px solid #1e293b;
    border-radius: 8px;
    padding: 1em;
    margin-bottom: 1em;
}
.debug-log {
    /* System monospace fallback chain: try the OS's own before dropping to
       the CSS generic, which on Linux without SF Mono/Fira Code/Consolas
       renders as an unstyled bitmap. */
    font-family: ui-monospace, 'Cascadia Mono', 'JetBrains Mono', 'DejaVu Sans Mono', 'Liberation Mono', Menlo, Consolas, monospace;
    font-size: 0.8em;
    background: #020617;
    border: 1px solid #1e293b;
    border-radius: 4px;
    padding: 0.5em;
    height: 300px;
    overflow-y: auto;
    white-space: pre-wrap;
    word-break: break-all;
}
.debug-log .log-entry { margin-bottom: 2px; }
.debug-log .log-info { color: #22d3ee; }
.debug-log .log-debug { color: #64748b; }
.debug-log .log-trace { color: #475569; }
.debug-log .log-tx { color: #fbbf24; }
.debug-log .log-rx { color: #22c55e; }
.debug-log .log-error { color: #ef4444; }
.debug-log .log-state { color: #a78bfa; }
.log-controls {
    display: flex;
    gap: 0.5em;
    margin-bottom: 0.5em;
    align-items: center;
}
.log-controls label { font-size: 0.85em; color: #22d3ee; }
.log-controls select { padding: 0.25em 0.5em; font-size: 0.85em; }
nav {
    display: flex;
    gap: 1em;
    margin-bottom: 1em;
    padding-bottom: 0.5em;
    border-bottom: 1px solid #1e293b;
}
nav a {
    text-decoration: none;
    padding: 0.25em 0.5em;
    border-radius: 4px;
}
nav a:hover { background: #1e293b; }
nav a.active { background: #1e293b; color: #f1f5f9; }
.msg { padding: 0.5em 0.75em; border-radius: 4px; margin-bottom: 1em; }
.msg-success { background: #052e16; color: #22c55e; border: 1px solid #22c55e; }
.msg-error { background: #450a0a; color: #ef4444; border: 1px solid #ef4444; }
.browse-bar {
    display: flex;
    gap: 0.5em;
    margin-bottom: 1em;
}
.browse-bar input { flex: 1; }
"#;

pub fn connect_page(
    my_callsign: &str,
    target_callsign: &str,
    connection_state: &str,
    connection_state_class: &str,
    ports_json: &str,
    transport_default: crate::transport::TransportKind,
    vara: &crate::config::VaraSection,
) -> String {
    use crate::transport::{TransportKind, VaraBandwidth};

    let ax25_hidden = if transport_default == TransportKind::Ax25 { "" } else { " hidden" };
    let vara_hidden = if transport_default != TransportKind::Ax25 { "" } else { " hidden" };

    let ax25_selected    = if transport_default == TransportKind::Ax25    { " selected" } else { "" };
    let vara_fm_selected = if transport_default == TransportKind::VaraFm  { " selected" } else { "" };
    let vara_hf_selected = if transport_default == TransportKind::VaraHf  { " selected" } else { "" };

    // Initial form values: use FM endpoint by default (matches the dropdown
    // default when nothing else is set); the JS swaps to HF on selection.
    let initial_ep = match transport_default {
        TransportKind::VaraHf => &vara.hf,
        _ => &vara.fm,
    };
    let vara_cmd_host  = h(&initial_ep.cmd_host);
    let vara_cmd_port  = initial_ep.cmd_port;
    let vara_data_host = h(&initial_ep.data_host);
    let vara_data_port = initial_ep.data_port;

    let vara_bw_vnarrow_sel = if initial_ep.bandwidth == VaraBandwidth::VNarrow { " selected" } else { "" };
    let vara_bw_vwide_sel   = if initial_ep.bandwidth == VaraBandwidth::VWide   { " selected" } else { "" };
    let vara_bw_250_sel     = if initial_ep.bandwidth == VaraBandwidth::Bw250   { " selected" } else { "" };
    let vara_bw_500_sel     = if initial_ep.bandwidth == VaraBandwidth::Bw500   { " selected" } else { "" };
    let vara_bw_2300_sel    = if initial_ep.bandwidth == VaraBandwidth::Bw2300  { " selected" } else { "" };
    let vara_bw_2750_sel    = if initial_ep.bandwidth == VaraBandwidth::Bw2750  { " selected" } else { "" };

    // Serialize both endpoints for the JS swap-on-mode-change handler.
    let endpoint_json = |ep: &crate::config::VaraEndpoint| {
        serde_json::json!({
            "cmd_host": ep.cmd_host,
            "cmd_port": ep.cmd_port,
            "data_host": ep.data_host,
            "data_port": ep.data_port,
            "bandwidth": ep.bandwidth.to_string(),
        })
    };
    let vara_endpoints_json = json_for_script(
        &serde_json::json!({
            "vara_hf": endpoint_json(&vara.hf),
            "vara_fm": endpoint_json(&vara.fm),
        })
        .to_string(),
    );

    format!(
        r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="utf-8">
    <meta name="viewport" content="width=device-width, initial-scale=1">
    <meta http-equiv="Content-Security-Policy" content="default-src 'self'; script-src 'unsafe-inline'; style-src 'unsafe-inline'; connect-src 'self'; form-action 'self'; frame-ancestors 'none'; base-uri 'none'">
    <title>Packet Browser - Connect</title>
    <style>{css}</style>
</head>
<body>
    <nav>
        <a href="/connect" class="active">Connect</a>
        <a href="/browse">Browse</a>
        <a href="/configuration">Configuration</a>
    </nav>

    <h1>Packet Browser</h1>

    <div class="card">
        <h2>Connection</h2>
        <p>Status: <span id="status-badge" class="status-badge {state_class}">{state}</span></p>

        <div class="form-group">
            <label for="my-call">My Callsign</label>
            <input type="text" id="my-call" value="{my_call}" placeholder="N0CALL" autocomplete="off">
        </div>

        <div class="form-group">
            <label for="target-call">Target Callsign</label>
            <input type="text" id="target-call" value="{target_call}" placeholder="NODE1" autocomplete="off">
        </div>

        <div class="form-group">
            <label for="transport">Transport</label>
            <select id="transport" onchange="onTransportChange()">
                <option value="ax25"{ax25_selected}>AX.25 (AGWPE)</option>
                <option value="vara_fm"{vara_fm_selected}>VARA FM</option>
                <option value="vara_hf"{vara_hf_selected}>VARA HF / Mercury</option>
            </select>
        </div>

        <div id="ax25-fields"{ax25_hidden}>
            <div class="form-group">
                <label for="port-select">AGWPE Port</label>
                <select id="port-select">
                    <option value="">-- query AGWPE for ports --</option>
                </select>
            </div>
        </div>

        <div id="vara-fields"{vara_hidden}>
            <div class="form-group">
                <label for="vara-cmd-host">VARA Command Host</label>
                <input type="text" id="vara-cmd-host" value="{vara_cmd_host}" placeholder="127.0.0.1" autocomplete="off">
            </div>
            <div class="form-group">
                <label for="vara-cmd-port">VARA Command Port</label>
                <input type="number" id="vara-cmd-port" value="{vara_cmd_port}" min="1" max="65535">
            </div>
            <div class="form-group">
                <label for="vara-data-host">VARA Data Host</label>
                <input type="text" id="vara-data-host" value="{vara_data_host}" placeholder="127.0.0.1" autocomplete="off">
            </div>
            <div class="form-group">
                <label for="vara-data-port">VARA Data Port</label>
                <input type="number" id="vara-data-port" value="{vara_data_port}" min="1" max="65535">
            </div>
            <div class="form-group">
                <label for="vara-bandwidth">Bandwidth</label>
                <select id="vara-bandwidth">
                    <option value="vnarrow"{vara_bw_vnarrow_sel}>VNarrow</option>
                    <option value="vwide"{vara_bw_vwide_sel}>VWide</option>
                    <option value="bw250"{vara_bw_250_sel}>BW250</option>
                    <option value="bw500"{vara_bw_500_sel}>BW500</option>
                    <option value="bw2300"{vara_bw_2300_sel}>BW2300</option>
                    <option value="bw2750"{vara_bw_2750_sel}>BW2750</option>
                </select>
            </div>
        </div>

        <div class="btn-row">
            <button id="btn-modem" onclick="connectModem()">Connect to Modem</button>
            <button id="btn-connect" class="primary" onclick="connectNode()" disabled>Connect to Node</button>
            <button id="btn-disconnect" class="danger" onclick="disconnectNode()" disabled>Disconnect</button>
        </div>
    </div>

    <div id="msg-area"></div>

    <div id="consent-modal" class="modal-backdrop" style="display:none" role="dialog" aria-modal="true" aria-labelledby="consent-modal-title">
        <div class="modal">
            <h2 id="consent-modal-title">Confirm connection</h2>
            <p>The remote station is asking you to acknowledge the following notice before continuing:</p>
            <pre id="consent-disclaimer" class="consent-disclaimer"></pre>
            <p>Only agree if you accept the notice above.</p>
            <div class="btn-row">
                <button class="primary" onclick="submitConsent(true)">I Agree</button>
                <button class="danger" onclick="submitConsent(false)">Decline &amp; Disconnect</button>
            </div>
        </div>
    </div>

    <div class="card">
        <h2>Debug Log</h2>
        <div class="log-controls">
            <label for="log-filter">Level:</label>
            <select id="log-filter" onchange="filterLogs()">
                <option value="all">All</option>
                <option value="info">Info</option>
                <option value="debug">Debug</option>
                <option value="trace">Trace</option>
            </select>
            <button onclick="clearLogs()">Clear</button>
        </div>
        <div id="debug-log" class="debug-log"></div>
    </div>

    <div class="card">
        <h2>Server</h2>
        <p><small>Cleanly disconnect the modem and stop the local proxy. You'll need to relaunch the client to browse again.</small></p>
        <div class="btn-row">
            <button class="danger" onclick="shutdownServer()">Shutdown Server</button>
        </div>
    </div>

    <script>
        let ports = {ports_json};
        let logEntries = [];
        let eventSource = null;

        function initPorts() {{
            const sel = document.getElementById('port-select');
            sel.innerHTML = '';
            if (ports.length === 0) {{
                sel.innerHTML = '<option value="">-- no ports found --</option>';
                return;
            }}
            ports.forEach(p => {{
                const opt = document.createElement('option');
                opt.value = p.port_num;
                opt.textContent = p.port_num + ': ' + p.description;
                sel.appendChild(opt);
            }});
        }}

        function updateUI(state) {{
            const badge = document.getElementById('status-badge');
            badge.textContent = state;
            badge.className = 'status-badge status-' + state.toLowerCase().replace(/[^a-z]/g, '-');

            const btnModem = document.getElementById('btn-modem');
            const btnConnect = document.getElementById('btn-connect');
            const btnDisconnect = document.getElementById('btn-disconnect');

            const busy = (state === 'Connecting' || state === 'Awaiting consent');
            btnModem.disabled = (state === 'Modem Connected' || busy || state === 'Connected');
            btnConnect.disabled = (state !== 'Modem Connected');
            btnDisconnect.disabled = (state !== 'Connected' && !busy);

            if (state === 'Awaiting consent') {{
                openConsentModal();
            }} else {{
                closeConsentModal();
            }}
        }}

        let consentOpen = false;
        async function openConsentModal() {{
            if (consentOpen) return;
            consentOpen = true;
            try {{
                const resp = await fetch('/api/consent');
                const data = await resp.json();
                if (!data.awaiting) {{ consentOpen = false; return; }}
                document.getElementById('consent-disclaimer').textContent =
                    data.disclaimer || '(no disclaimer text provided)';
                document.getElementById('consent-modal').style.display = 'flex';
            }} catch (e) {{
                consentOpen = false;
                showMsg('Could not fetch consent prompt: ' + e.message, true);
            }}
        }}

        function closeConsentModal() {{
            document.getElementById('consent-modal').style.display = 'none';
            consentOpen = false;
        }}

        async function submitConsent(accepted) {{
            closeConsentModal();
            try {{
                const resp = await fetch('/api/consent', {{
                    method: 'POST',
                    headers: {{ 'Content-Type': 'application/json' }},
                    body: JSON.stringify({{ accepted: accepted }})
                }});
                const data = await resp.json();
                if (!data.ok) {{
                    showMsg(data.error || 'Consent submission failed', true);
                    return;
                }}
                if (!accepted) {{
                    showMsg('Declined. Disconnecting.');
                    // Give the background handshake a moment to unwind, then
                    // force-tear-down so we don't leave a half-open session.
                    setTimeout(() => {{ ax25Disconnect(); }}, 250);
                }}
            }} catch (e) {{
                showMsg('Error: ' + e.message, true);
            }}
        }}

        function showMsg(text, isError) {{
            const area = document.getElementById('msg-area');
            area.innerHTML = '<div class="msg ' + (isError ? 'msg-error' : 'msg-success') + '">' + text + '</div>';
            setTimeout(() => area.innerHTML = '', 5000);
        }}

        async function connectModem() {{
            const btn = document.getElementById('btn-modem');
            btn.disabled = true;
            btn.textContent = 'Connecting...';
            try {{
                const resp = await fetch('/api/agwpe-status', {{ method: 'POST' }});
                const data = await resp.json();
                if (data.ok) {{
                    ports = data.ports || [];
                    initPorts();
                    updateUI(data.state);
                    showMsg('Connected to modem');
                }} else {{
                    updateUI(data.state || 'Error');
                    showMsg(data.error || 'Failed to connect to modem', true);
                }}
            }} catch (e) {{
                showMsg('Error: ' + e.message, true);
                updateUI('Error');
            }}
            btn.textContent = 'Connect to Modem';
        }}

        async function connectNode() {{
            const target = document.getElementById('target-call').value.trim();
            const portNum = document.getElementById('port-select').value;
            if (!target) {{ showMsg('Enter a target callsign', true); return; }}
            if (portNum === '') {{ showMsg('Select a modem port first', true); return; }}

            const btn = document.getElementById('btn-connect');
            btn.disabled = true;
            btn.textContent = 'Connecting...';
            updateUI('Connecting');
            try {{
                const resp = await fetch('/api/connect', {{
                    method: 'POST',
                    headers: {{ 'Content-Type': 'application/json' }},
                    body: JSON.stringify({{ target_callsign: target, port_num: parseInt(portNum) }})
                }});
                const data = await resp.json();
                if (data.ok) {{
                    updateUI('Connected');
                    showMsg('Connected to ' + target + '. Opening browser…');
                    // Send the user straight to the browse UI so there's an
                    // obvious next step; the connect page has no other job
                    // once the link is up.
                    window.location.href = '/browse';
                    return;
                }} else {{
                    updateUI(data.state || 'Error');
                    showMsg(data.error || 'Connection failed', true);
                }}
            }} catch (e) {{
                showMsg('Error: ' + e.message, true);
                updateUI('Error');
            }}
            btn.textContent = 'Connect to Node';
            btn.disabled = false;
        }}

        async function disconnectNode() {{
            try {{
                const resp = await fetch('/api/disconnect', {{ method: 'POST' }});
                const data = await resp.json();
                updateUI('Disconnected');
                showMsg('Disconnected');
            }} catch (e) {{
                showMsg('Error: ' + e.message, true);
            }}
        }}

        async function shutdownServer() {{
            if (!confirm('Shut down the packet-browser client? You will need to relaunch it to browse again.')) return;
            try {{
                await fetch('/api/shutdown', {{ method: 'POST' }});
                document.body.innerHTML =
                    '<div style="max-width:600px;margin:6em auto;padding:2em;text-align:center;font-family:sans-serif;">' +
                    '<h1>Server shutting down</h1>' +
                    '<p>You can close this tab.</p>' +
                    '</div>';
            }} catch (e) {{
                showMsg('Shutdown request failed: ' + e.message, true);
            }}
        }}

        function addLogEntry(entry) {{
            logEntries.push(entry);
            if (logEntries.length > 1000) logEntries.shift();
            renderLogs();
        }}

        function renderLogs() {{
            const log = document.getElementById('debug-log');
            const filter = document.getElementById('log-filter').value;
            let html = '';
            for (const e of logEntries) {{
                if (filter !== 'all' && e.level.toLowerCase() !== filter) continue;
                const dir = e.direction ? ('[' + e.direction + '] ') : '';
                const cls = 'log-entry log-' + e.level.toLowerCase()
                    + (e.direction ? ' log-' + e.direction.toLowerCase() : '')
                    + (e.category === 'STATE' ? ' log-state' : '')
                    + (e.category === 'ERROR' ? ' log-error' : '');
                const ts = e.timestamp ? e.timestamp.substring(11, 23) : '';
                html += '<div class="' + cls + '">' + ts + ' ' + dir + e.category + ': ' + escapeHtml(e.message) + '</div>';
            }}
            log.innerHTML = html;
            log.scrollTop = log.scrollHeight;
        }}

        function filterLogs() {{ renderLogs(); }}

        function clearLogs() {{
            logEntries = [];
            renderLogs();
        }}

        function escapeHtml(s) {{
            const d = document.createElement('div');
            d.textContent = s;
            return d.innerHTML;
        }}

        const VARA_ENDPOINTS = {vara_endpoints_json};

        function onTransportChange() {{
            const t = document.getElementById('transport').value;
            const ax25Fields = document.getElementById('ax25-fields');
            const varaFields = document.getElementById('vara-fields');
            if (t === 'ax25') {{
                ax25Fields.removeAttribute('hidden');
                varaFields.setAttribute('hidden', '');
            }} else {{
                ax25Fields.setAttribute('hidden', '');
                varaFields.removeAttribute('hidden');
                // Swap host/port/bandwidth to the endpoint that matches the
                // chosen VARA mode ([vara_hf] vs [vara_fm] in the config).
                const ep = VARA_ENDPOINTS[t];
                if (ep) {{
                    document.getElementById('vara-cmd-host').value  = ep.cmd_host;
                    document.getElementById('vara-cmd-port').value  = ep.cmd_port;
                    document.getElementById('vara-data-host').value = ep.data_host;
                    document.getElementById('vara-data-port').value = ep.data_port;
                    document.getElementById('vara-bandwidth').value = ep.bandwidth;
                }}
            }}
        }}

        function connectSSE() {{
            if (eventSource) eventSource.close();
            eventSource = new EventSource('/events');
            eventSource.onmessage = function(event) {{
                try {{
                    const entry = JSON.parse(event.data);
                    addLogEntry(entry);
                    // State transitions arrive as STATE-category log lines of
                    // the form "State changed to: <name>". Parse them so the
                    // UI can react to the AwaitingConsent → Connected flip
                    // even while /api/connect is still awaiting server-side.
                    if (entry.category === 'STATE') {{
                        const m = /^State changed to:\s*(.+)$/.exec(entry.message);
                        if (m) updateUI(m[1].trim());
                    }}
                }} catch (e) {{}}
            }};
            eventSource.onerror = function() {{
                setTimeout(connectSSE, 3000);
            }};
        }}

        initPorts();
        updateUI('{state}');
        connectSSE();

        fetch('/api/agwpe-status').then(r => r.json()).then(data => {{
            if (data.ports) {{
                ports = data.ports;
                initPorts();
            }}
            if (data.state) updateUI(data.state);
        }}).catch(() => {{}});
    </script>
</body>
</html>"#,
        css = CSS,
        state = h(connection_state),
        state_class = h(connection_state_class),
        my_call = h(my_callsign),
        target_call = h(target_callsign),
        ports_json = json_for_script(ports_json),
        ax25_selected = ax25_selected,
        vara_fm_selected = vara_fm_selected,
        vara_hf_selected = vara_hf_selected,
        ax25_hidden = ax25_hidden,
        vara_hidden = vara_hidden,
        vara_cmd_host = vara_cmd_host,
        vara_cmd_port = vara_cmd_port,
        vara_data_host = vara_data_host,
        vara_data_port = vara_data_port,
        vara_bw_vnarrow_sel = vara_bw_vnarrow_sel,
        vara_bw_vwide_sel = vara_bw_vwide_sel,
        vara_bw_250_sel = vara_bw_250_sel,
        vara_bw_500_sel = vara_bw_500_sel,
        vara_bw_2300_sel = vara_bw_2300_sel,
        vara_bw_2750_sel = vara_bw_2750_sel,
        vara_endpoints_json = vara_endpoints_json,
    )
}

pub fn configuration_page(
    agwpe_host: &str,
    agwpe_port: u16,
    my_callsign: &str,
    target_callsign: &str,
    bpq_command: &str,
    skip_bpq_app: bool,
    vara: &crate::config::VaraSection,
) -> String {
    use crate::transport::VaraBandwidth;

    let hf_cmd_host  = h(&vara.hf.cmd_host);
    let hf_cmd_port  = vara.hf.cmd_port;
    let hf_data_host = h(&vara.hf.data_host);
    let hf_data_port = vara.hf.data_port;
    let hf_bw_250_sel  = if vara.hf.bandwidth == VaraBandwidth::Bw250  { " selected" } else { "" };
    let hf_bw_500_sel  = if vara.hf.bandwidth == VaraBandwidth::Bw500  { " selected" } else { "" };
    let hf_bw_2300_sel = if vara.hf.bandwidth == VaraBandwidth::Bw2300 { " selected" } else { "" };
    let hf_bw_2750_sel = if vara.hf.bandwidth == VaraBandwidth::Bw2750 { " selected" } else { "" };

    let fm_cmd_host  = h(&vara.fm.cmd_host);
    let fm_cmd_port  = vara.fm.cmd_port;
    let fm_data_host = h(&vara.fm.data_host);
    let fm_data_port = vara.fm.data_port;
    let fm_bw_vnarrow_sel = if vara.fm.bandwidth == VaraBandwidth::VNarrow { " selected" } else { "" };
    let fm_bw_vwide_sel   = if vara.fm.bandwidth == VaraBandwidth::VWide   { " selected" } else { "" };

    format!(
        r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="utf-8">
    <meta name="viewport" content="width=device-width, initial-scale=1">
    <meta http-equiv="Content-Security-Policy" content="default-src 'self'; script-src 'unsafe-inline'; style-src 'unsafe-inline'; connect-src 'self'; form-action 'self'; frame-ancestors 'none'; base-uri 'none'">
    <title>Packet Browser - Configuration</title>
    <style>{css}</style>
</head>
<body>
    <nav>
        <a href="/connect">Connect</a>
        <a href="/browse">Browse</a>
        <a href="/configuration" class="active">Configuration</a>
    </nav>

    <h1>Configuration</h1>

    <div id="msg-area"></div>

    <div class="card">
        <h2>AGWPE Settings</h2>

        <div class="form-group">
            <label for="agwpe-host">AGWPE Host</label>
            <input type="text" id="agwpe-host" value="{host}" placeholder="127.0.0.1">
        </div>

        <div class="form-group">
            <label for="agwpe-port">AGWPE Port</label>
            <input type="number" id="agwpe-port" value="{port}" placeholder="8000">
        </div>
    </div>

    <div class="card">
        <h2>Session Settings</h2>

        <div class="form-group">
            <label for="my-callsign">My Callsign</label>
            <input type="text" id="my-callsign" value="{my_callsign}" placeholder="N0CALL">
            <small>Your amateur radio callsign</small>
        </div>

        <div class="form-group">
            <label for="target-callsign">Target Callsign</label>
            <input type="text" id="target-callsign" value="{target_callsign}" placeholder="NODE1">
            <small>The BPQ node or station to connect to</small>
        </div>

        <div class="form-group">
            <label for="skip-bpq-app">Skip BPQ Application Command</label>
            <input type="checkbox" id="skip-bpq-app" {skip_checked} onchange="updateBpqCommandVisibility()">
            <small>Enable if connecting directly to a node SSID that doesn't require an application command</small>
        </div>

        <div class="form-group" id="bpq-command-group">
            <label for="bpq-command">BPQ Application Command</label>
            <input type="text" id="bpq-command" value="{bpq_command}" placeholder="WEB">
            <small>The application command sent after connecting (e.g., WEB, BBS)</small>
        </div>

        <div class="btn-row">
            <button class="primary" onclick="saveConfig()">Save Configuration</button>
            <button onclick="testAgwpe()">Test AGWPE Connection</button>
        </div>
    </div>

    <div class="card">
        <h2>VARA HF / Mercury Settings</h2>
        <p><small>Endpoint for a VARA HF or Mercury modem. Common default: 8300 / 8301.</small></p>

        <div class="form-group">
            <label for="vara-hf-cmd-host">Command Host</label>
            <input type="text" id="vara-hf-cmd-host" value="{hf_cmd_host}" placeholder="127.0.0.1" autocomplete="off">
        </div>

        <div class="form-group">
            <label for="vara-hf-cmd-port">Command Port</label>
            <input type="number" id="vara-hf-cmd-port" value="{hf_cmd_port}" min="1" max="65535">
        </div>

        <div class="form-group">
            <label for="vara-hf-data-host">Data Host</label>
            <input type="text" id="vara-hf-data-host" value="{hf_data_host}" placeholder="127.0.0.1" autocomplete="off">
        </div>

        <div class="form-group">
            <label for="vara-hf-data-port">Data Port</label>
            <input type="number" id="vara-hf-data-port" value="{hf_data_port}" min="1" max="65535">
        </div>

        <div class="form-group">
            <label for="vara-hf-bandwidth">Bandwidth</label>
            <select id="vara-hf-bandwidth">
                <option value="bw250"{hf_bw_250_sel}>BW250</option>
                <option value="bw500"{hf_bw_500_sel}>BW500</option>
                <option value="bw2300"{hf_bw_2300_sel}>BW2300</option>
                <option value="bw2750"{hf_bw_2750_sel}>BW2750</option>
            </select>
        </div>

        <div class="btn-row">
            <button onclick="testVara('hf')">Test VARA / Mercury Connection</button>
        </div>
    </div>

    <div class="card">
        <h2>VARA FM Settings</h2>
        <p><small>Endpoint for a VARA FM modem. Common default: 8400 / 8401 (may differ from HF/Mercury).</small></p>

        <div class="form-group">
            <label for="vara-fm-cmd-host">Command Host</label>
            <input type="text" id="vara-fm-cmd-host" value="{fm_cmd_host}" placeholder="127.0.0.1" autocomplete="off">
        </div>

        <div class="form-group">
            <label for="vara-fm-cmd-port">Command Port</label>
            <input type="number" id="vara-fm-cmd-port" value="{fm_cmd_port}" min="1" max="65535">
        </div>

        <div class="form-group">
            <label for="vara-fm-data-host">Data Host</label>
            <input type="text" id="vara-fm-data-host" value="{fm_data_host}" placeholder="127.0.0.1" autocomplete="off">
        </div>

        <div class="form-group">
            <label for="vara-fm-data-port">Data Port</label>
            <input type="number" id="vara-fm-data-port" value="{fm_data_port}" min="1" max="65535">
        </div>

        <div class="form-group">
            <label for="vara-fm-bandwidth">Bandwidth</label>
            <select id="vara-fm-bandwidth">
                <option value="vnarrow"{fm_bw_vnarrow_sel}>VNarrow</option>
                <option value="vwide"{fm_bw_vwide_sel}>VWide</option>
            </select>
        </div>

        <div class="btn-row">
            <button onclick="testVara('fm')">Test VARA FM Connection</button>
        </div>
    </div>

    <div class="card">
        <h2>Server</h2>
        <p><small>Cleanly disconnect the modem and stop the local proxy. You'll need to relaunch the client to browse again.</small></p>
        <div class="btn-row">
            <button class="danger" onclick="shutdownServer()">Shutdown Server</button>
        </div>
    </div>

    <script>
        function showMsg(text, isError) {{
            const area = document.getElementById('msg-area');
            area.innerHTML = '<div class="msg ' + (isError ? 'msg-error' : 'msg-success') + '">' + text + '</div>';
            setTimeout(() => area.innerHTML = '', 5000);
        }}

        function updateBpqCommandVisibility() {{
            const skipped = document.getElementById('skip-bpq-app').checked;
            document.getElementById('bpq-command-group').style.display = skipped ? 'none' : '';
        }}

        function fillEndpoint(prefix, ep) {{
            if (!ep) return;
            document.getElementById(prefix + '-cmd-host').value  = ep.cmd_host  || '127.0.0.1';
            document.getElementById(prefix + '-cmd-port').value  = ep.cmd_port  || '';
            document.getElementById(prefix + '-data-host').value = ep.data_host || '127.0.0.1';
            document.getElementById(prefix + '-data-port').value = ep.data_port || '';
            if (ep.bandwidth) document.getElementById(prefix + '-bandwidth').value = ep.bandwidth;
        }}

        function readEndpoint(prefix) {{
            return {{
                cmd_host:  document.getElementById(prefix + '-cmd-host').value.trim(),
                cmd_port:  parseInt(document.getElementById(prefix + '-cmd-port').value),
                data_host: document.getElementById(prefix + '-data-host').value.trim(),
                data_port: parseInt(document.getElementById(prefix + '-data-port').value),
                bandwidth: document.getElementById(prefix + '-bandwidth').value,
            }};
        }}

        function validateEndpoint(ep, label) {{
            if (!ep.cmd_host)  {{ showMsg(label + ' Command Host is required', true); return false; }}
            if (!ep.cmd_port || ep.cmd_port < 1 || ep.cmd_port > 65535) {{ showMsg('Invalid ' + label + ' command port', true); return false; }}
            if (!ep.data_host) {{ showMsg(label + ' Data Host is required', true); return false; }}
            if (!ep.data_port || ep.data_port < 1 || ep.data_port > 65535) {{ showMsg('Invalid ' + label + ' data port', true); return false; }}
            return true;
        }}

        async function loadConfig() {{
            try {{
                const resp = await fetch('/api/config');
                const data = await resp.json();
                document.getElementById('agwpe-host').value = data.agwpe_host || '127.0.0.1';
                document.getElementById('agwpe-port').value = data.agwpe_port || 8000;
                document.getElementById('my-callsign').value = data.my_callsign || '';
                document.getElementById('target-callsign').value = data.target_callsign || '';
                document.getElementById('bpq-command').value = data.bpq_command || 'WEB';
                document.getElementById('skip-bpq-app').checked = data.skip_bpq_app || false;
                fillEndpoint('vara-hf', data.vara_hf);
                fillEndpoint('vara-fm', data.vara_fm);
                updateBpqCommandVisibility();
            }} catch (e) {{
                showMsg('Failed to load config: ' + e.message, true);
            }}
        }}

        async function saveConfig() {{
            const host = document.getElementById('agwpe-host').value.trim();
            const port = parseInt(document.getElementById('agwpe-port').value);
            const myCallsign = document.getElementById('my-callsign').value.trim();
            const targetCallsign = document.getElementById('target-callsign').value.trim();
            const bpqCommand = document.getElementById('bpq-command').value.trim();
            const skipBpqApp = document.getElementById('skip-bpq-app').checked;

            const hfEp = readEndpoint('vara-hf');
            const fmEp = readEndpoint('vara-fm');

            if (!host) {{ showMsg('AGWPE Host is required', true); return; }}
            if (!port || port < 1 || port > 65535) {{ showMsg('Invalid AGWPE port', true); return; }}
            if (!myCallsign) {{ showMsg('My Callsign is required', true); return; }}
            if (!validateEndpoint(hfEp, 'VARA HF / Mercury')) return;
            if (!validateEndpoint(fmEp, 'VARA FM')) return;

            try {{
                const resp = await fetch('/api/config', {{
                    method: 'POST',
                    headers: {{ 'Content-Type': 'application/json' }},
                    body: JSON.stringify({{
                        agwpe_host: host,
                        agwpe_port: port,
                        my_callsign: myCallsign,
                        target_callsign: targetCallsign,
                        bpq_command: bpqCommand,
                        skip_bpq_app: skipBpqApp,
                        vara_hf: hfEp,
                        vara_fm: fmEp,
                    }})
                }});
                const data = await resp.json();
                if (data.ok) {{
                    showMsg('Configuration saved successfully');
                }} else {{
                    showMsg(data.error || 'Failed to save', true);
                }}
            }} catch (e) {{
                showMsg('Error: ' + e.message, true);
            }}
        }}

        async function testAgwpe() {{
            try {{
                const resp = await fetch('/api/agwpe-status', {{ method: 'POST' }});
                const data = await resp.json();
                if (data.ok) {{
                    showMsg('AGWPE reachable. ' + (data.ports || []).length + ' port(s) found.');
                }} else {{
                    showMsg(data.error || 'AGWPE unreachable', true);
                }}
            }} catch (e) {{
                showMsg('Error: ' + e.message, true);
            }}
        }}

        async function shutdownServer() {{
            if (!confirm('Shut down the packet-browser client? You will need to relaunch it to browse again.')) return;
            try {{
                await fetch('/api/shutdown', {{ method: 'POST' }});
                document.body.innerHTML =
                    '<div style="max-width:600px;margin:6em auto;padding:2em;text-align:center;font-family:sans-serif;">' +
                    '<h1>Server shutting down</h1>' +
                    '<p>You can close this tab.</p>' +
                    '</div>';
            }} catch (e) {{
                showMsg('Shutdown request failed: ' + e.message, true);
            }}
        }}

        async function testVara(mode) {{
            const label = mode === 'fm' ? 'VARA FM' : 'VARA / Mercury';
            try {{
                const resp = await fetch('/api/vara-test', {{
                    method: 'POST',
                    headers: {{ 'Content-Type': 'application/json' }},
                    body: JSON.stringify({{ mode: mode }})
                }});
                const data = await resp.json();
                if (data.ok) {{
                    showMsg(label + ' modem reachable.');
                }} else {{
                    showMsg(data.error || (label + ' unreachable'), true);
                }}
            }} catch (e) {{
                showMsg('Error: ' + e.message, true);
            }}
        }}

        loadConfig();
    </script>
</body>
</html>"#,
        css = CSS,
        host = h(agwpe_host),
        port = agwpe_port,
        my_callsign = h(my_callsign),
        target_callsign = h(target_callsign),
        bpq_command = h(bpq_command),
        skip_checked = if skip_bpq_app { "checked" } else { "" },
        hf_cmd_host = hf_cmd_host,
        hf_cmd_port = hf_cmd_port,
        hf_data_host = hf_data_host,
        hf_data_port = hf_data_port,
        hf_bw_250_sel = hf_bw_250_sel,
        hf_bw_500_sel = hf_bw_500_sel,
        hf_bw_2300_sel = hf_bw_2300_sel,
        hf_bw_2750_sel = hf_bw_2750_sel,
        fm_cmd_host = fm_cmd_host,
        fm_cmd_port = fm_cmd_port,
        fm_data_host = fm_data_host,
        fm_data_port = fm_data_port,
        fm_bw_vnarrow_sel = fm_bw_vnarrow_sel,
        fm_bw_vwide_sel = fm_bw_vwide_sel,
    )
}

pub fn error_page(message: &str) -> String {
    format!(
        r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="utf-8">
    <meta name="viewport" content="width=device-width, initial-scale=1">
    <title>Packet Browser - Error</title>
    <style>{css}</style>
</head>
<body>
    <nav>
        <a href="/connect">Connect</a>
        <a href="/browse">Browse</a>
        <a href="/configuration">Configuration</a>
    </nav>

    <h1>Error</h1>
    <div class="card">
        <p class="msg msg-error">{message}</p>
        <p><a href="/connect">Return to Connect page</a></p>
    </div>
</body>
</html>"#,
        css = CSS,
        message = h(message),
    )
}

pub fn render_session_error_page(message: &str, show_reconnect_link: bool) -> String {
    let reconnect_link = if show_reconnect_link {
        r#"<p><a href="/connect">Reconnect</a></p>"#
    } else {
        ""
    };
    format!(
        r#"<!DOCTYPE html>
<html lang="en"><head><meta charset="utf-8"><title>Session error</title><style>{css}</style></head>
<body style="font-family: sans-serif; max-width: 600px; margin: 4em auto; padding: 1em;">
<h1>Session error</h1>
<p>{message}</p>
{reconnect_link}
</body></html>"#,
        css = CSS,
        message = h(message),
        reconnect_link = reconnect_link,
    )
}

pub fn browse_page(html_content: &str, url: &str) -> String {
    let escaped_url = h(url);

    // Style rules here are scoped to `.browse-header` so the client's chrome
    // stays consistent while the fetched content below renders under browser
    // defaults + whatever inline CSS the author's <style>/style="" blocks
    // supplied. Global body/a/h1/input rules from the shared CSS const are
    // deliberately not included — they'd cascade into browse-content and
    // repaint every page in the client's palette, which is exactly what the
    // reader was complaining about.
    format!(
        r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="utf-8">
    <meta name="viewport" content="width=device-width, initial-scale=1">
    <meta http-equiv="Content-Security-Policy" content="default-src 'none'; style-src 'unsafe-inline'; img-src data:; form-action 'self'; base-uri 'none'; frame-ancestors 'none'">
    <title>Packet Browser</title>
    <style>
    .browse-header, .browse-header * {{
        box-sizing: border-box;
        margin: 0;
        padding: 0;
        font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', sans-serif;
    }}
    .browse-header {{
        position: fixed;
        top: 0;
        left: 0;
        right: 0;
        z-index: 2147483647;
        background: #0f172a;
        color: #f1f5f9;
        border-bottom: 1px solid #1e293b;
        padding: 0.5em 1em;
        display: flex;
        gap: 0.5em;
        align-items: center;
    }}
    .browse-header a {{
        color: #22d3ee;
        font-size: 0.85em;
        white-space: nowrap;
        text-decoration: none;
    }}
    .browse-header a:hover {{ text-decoration: underline; }}
    .browse-header input {{
        background: #0c1222;
        color: #f1f5f9;
        border: 1px solid #1e293b;
        border-radius: 4px;
        padding: 0.4em 0.6em;
        font-size: 0.9em;
        flex: 1;
    }}
    .browse-header input:focus {{ outline: none; border-color: #22d3ee; }}
    .browse-header button {{
        background: #16a34a;
        color: #f1f5f9;
        border: 1px solid #1e293b;
        border-radius: 4px;
        padding: 0.4em 0.9em;
        font-size: 0.9em;
        cursor: pointer;
    }}
    .browse-header button:hover {{ background: #22c55e; }}
    /* Leave room for the fixed header so fetched content isn't hidden under
       it. margin-top lives on browse-content (not body) so we don't fight
       with body-level rules from the fetched CSS. */
    .browse-content {{ margin-top: 3.25em; }}
    </style>
</head>
<body>
    <div class="browse-header">
        <a href="/connect">Connect</a>
        <a href="/configuration">Config</a>
        <a href="/cache">Cache</a>
        <a href="/browse?url={url}&amp;nocache=1" title="Bypass cache and refetch">Reload</a>
        <form action="/browse" method="GET" style="display:flex;gap:0.5em;flex:1;margin:0">
            <input type="text" name="url" value="{url}" placeholder="Enter a URL, e.g. https://example.com" autocomplete="off" autofocus>
            <button type="submit">Go</button>
        </form>
    </div>
    <div class="browse-content">
        {content}
    </div>
</body>
</html>"#,
        url = escaped_url,
        content = html_content,
    )
}

pub struct CachePageRow {
    pub url: String,
    pub size_bytes: u64,
    pub fetched_at_iso: String,
    pub last_used_iso: String,
    pub ttl_remaining_secs: i64,
    pub etag: String,
}

pub fn cache_page(rows: &[CachePageRow], total_bytes: u64, cap_bytes: u64) -> String {
    let mut body = String::new();
    body.push_str(&format!(
        "<p>{} entries — {} / {} bytes used.</p>",
        rows.len(),
        total_bytes,
        cap_bytes,
    ));
    body.push_str(r#"<form method="POST" action="/api/cache/clear" style="margin-bottom:1em"><button class="danger" type="submit">Clear all</button></form>"#);
    body.push_str(r#"<table style="width:100%;border-collapse:collapse">"#);
    body.push_str(r#"<thead><tr><th style="text-align:left">URL</th><th>Size</th><th>Fetched</th><th>Last used</th><th>TTL left</th><th></th></tr></thead><tbody>"#);
    for row in rows {
        body.push_str(&format!(
            r#"<tr>
                <td style="max-width:40em;overflow:hidden;text-overflow:ellipsis">{url}</td>
                <td>{size}</td>
                <td>{fetched}</td>
                <td>{used}</td>
                <td>{ttl}s</td>
                <td>
                    <form method="POST" action="/api/cache/delete" style="margin:0">
                        <input type="hidden" name="url" value="{url_attr}">
                        <button class="danger" type="submit">Delete</button>
                    </form>
                </td>
            </tr>"#,
            url = h(&row.url),
            size = row.size_bytes,
            fetched = h(&row.fetched_at_iso),
            used = h(&row.last_used_iso),
            ttl = row.ttl_remaining_secs,
            url_attr = h(&row.url),
        ));
    }
    body.push_str("</tbody></table>");

    format!(
        r#"<!DOCTYPE html>
<html lang="en"><head><meta charset="utf-8"><title>Cache</title><style>{css}</style></head>
<body><h1>Cache</h1><p><a href="/browse">Back to browse</a></p>{body}</body></html>"#,
        css = CSS,
        body = body,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn connect_page_renders_transport_dropdown_with_defaults() {
        use crate::config::{VaraEndpoint, VaraSection};
        use crate::transport::{TransportKind, VaraBandwidth};

        let vara = VaraSection {
            hf: VaraEndpoint {
                cmd_host: "10.0.0.5".into(),
                cmd_port: 8300,
                data_host: "10.0.0.5".into(),
                data_port: 8301,
                bandwidth: VaraBandwidth::Bw500,
            },
            fm: VaraEndpoint {
                cmd_host: "10.0.0.6".into(),
                cmd_port: 8400,
                data_host: "10.0.0.6".into(),
                data_port: 8401,
                bandwidth: VaraBandwidth::VWide,
            },
        };

        let html = connect_page(
            "W1TEST",
            "N0CALL-8",
            "Modem Connected",
            "status-modem-connected",
            "[]",
            TransportKind::VaraFm,
            &vara,
        );

        assert!(html.contains("<select id=\"transport\""));
        assert!(html.contains("value=\"ax25\""));
        assert!(html.contains("value=\"vara_fm\" selected"));
        assert!(html.contains("VARA HF / Mercury"));
        assert!(html.contains("id=\"vara-cmd-host\""));
        // Initial values come from the FM endpoint because transport_default = VaraFm.
        assert!(html.contains("value=\"10.0.0.6\""));
        // JS swap table contains both endpoints.
        assert!(html.contains("VARA_ENDPOINTS"));
        assert!(html.contains("\"vara_hf\""));
        assert!(html.contains("\"vara_fm\""));
    }

    #[test]
    fn test_session_error_page_shows_message_and_link() {
        let html = render_session_error_page("test message", true);
        assert!(html.contains("test message"));
        assert!(html.contains("href=\"/connect\""));

        let html_no_link = render_session_error_page("no link message", false);
        assert!(html_no_link.contains("no link message"));
        assert!(!html_no_link.contains("href=\"/connect\""));
    }

    #[test]
    fn test_session_error_page_escapes_html() {
        let html = render_session_error_page("bad <script>alert(1)</script> input", false);
        assert!(!html.contains("<script>"));
        assert!(html.contains("&lt;script&gt;"));
    }
}
