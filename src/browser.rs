use headless_chrome::{Browser, Tab};
use std::io::{BufRead, BufReader};
use std::process::{Child, Command, Stdio};
use std::sync::{mpsc, Arc};
use std::time::Duration;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum BrowserError {
    #[error("Failed to launch browser: {0}")]
    LaunchFailed(String),
    #[error("Failed to navigate: {0}")]
    NavigationFailed(String),
    #[error("Failed to extract content: {0}")]
    ExtractionFailed(String),
}

#[derive(Debug, Clone)]
pub enum InputKind {
    Text,
    Select { options: Vec<String> },
    Radio { options: Vec<String> },
    Checkbox { checked: bool },
}

#[derive(Debug, Clone)]
pub struct InputField {
    pub index: usize,
    pub label: String,
    pub kind: InputKind,
}

pub struct PageContent {
    pub text: Vec<String>,
    pub links: Vec<(usize, String)>,
    pub inputs: Vec<InputField>,
}

pub struct BrowserInstance {
    browser: Browser,
    _chrome: Child, // Keep the Chrome subprocess alive for this session
}

// Chrome flags required to run inside a Docker container with cap_drop: ALL.
// These are passed as CLI args when we spawn Chrome ourselves.
const CHROME_ARGS: &[&str] = &[
    "--headless",
    "--remote-debugging-port=0",   // Let Chrome pick a free port; outputs URL to stderr
    "--user-data-dir=/tmp/chrome", // Required: Chrome derives crashpad database path from this;
                                   // without it crashpad handler exits immediately crashing Chrome
    "--disable-dev-shm-usage",     // Docker limits /dev/shm to 64MB; forces Chrome to use /tmp
    "--disable-gpu",               // No GPU in container
    "--no-first-run",              // Skip first-run setup
    "--no-default-browser-check",
    "--disable-extensions",
    "--no-zygote",                 // Zygote process spawner requires fork capabilities unavailable under cap_drop: ALL
    "--disable-setuid-sandbox",
    "--no-sandbox",
    "--disable-crash-reporter",
    "--disable-breakpad",
];

impl BrowserInstance {
    pub fn new() -> Result<Self, BrowserError> {
        eprintln!("[BROWSER] Launching Chrome at /bin/chromium");

        let mut child = Command::new("/bin/chromium")
            .args(CHROME_ARGS)
            .env("BREAKPAD_DUMP_LOCATION", "/tmp") // Prevents crashpad handler crash under cap_drop: ALL
            .stdout(Stdio::null())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|e| BrowserError::LaunchFailed(format!("Failed to spawn /bin/chromium: {}", e)))?;

        let stderr = child.stderr.take()
            .ok_or_else(|| BrowserError::LaunchFailed("Could not capture Chrome stderr".to_string()))?;

        // Read Chrome's stderr in a channel-based thread.
        // - Every line is forwarded to eprintln! so it appears in docker logs.
        // - When the DevTools URL is found, it is sent via channel.
        let (tx, rx) = mpsc::channel::<String>();

        std::thread::spawn(move || {
            let reader = BufReader::new(stderr);
            let mut url_sent = false;
            for line in reader.lines().flatten() {
                eprintln!("[CHROME] {}", line);
                if !url_sent {
                    if let Some(url) = line.strip_prefix("DevTools listening on ") {
                        let _ = tx.send(url.trim().to_string());
                        url_sent = true;
                        // tx is now done but the thread continues forwarding logs
                    }
                }
            }
            eprintln!("[CHROME] stderr closed (Chrome exited or crashed)");
        });

        let ws_url = match rx.recv_timeout(Duration::from_secs(30)) {
            Ok(url) => url,
            Err(mpsc::RecvTimeoutError::Timeout) => {
                let _ = child.kill();
                return Err(BrowserError::LaunchFailed(
                    "Chrome did not output DevTools URL within 30 seconds".to_string()
                ));
            }
            Err(mpsc::RecvTimeoutError::Disconnected) => {
                return Err(BrowserError::LaunchFailed(
                    "Chrome exited before outputting DevTools URL".to_string()
                ));
            }
        };

        eprintln!("[BROWSER] Chrome ready, connecting to {}", ws_url);

        let browser = Browser::connect(ws_url)
            .map_err(|e| BrowserError::LaunchFailed(e.to_string()))?;

        eprintln!("[BROWSER] Connected to Chrome DevTools");

        Ok(Self { browser, _chrome: child })
    }

    pub fn fetch_page(&self, url: &str) -> Result<PageContent, BrowserError> {
        eprintln!("[BROWSER] Fetching: {}", url);
        let tab = self.browser.new_tab()
            .map_err(|e| BrowserError::LaunchFailed(e.to_string()))?;

        tab.set_default_timeout(Duration::from_secs(30));

        tab.navigate_to(url)
            .map_err(|e| BrowserError::NavigationFailed(e.to_string()))?;

        tab.wait_until_navigated()
            .map_err(|e| BrowserError::NavigationFailed(e.to_string()))?;

        eprintln!("[BROWSER] Page loaded: {}", url);
        extract_page_content(&tab)
    }

    pub fn interact_with_input(
        &self,
        url: &str,
        input_index: usize,
        value: Option<&str>,
    ) -> Result<PageContent, BrowserError> {
        eprintln!("[BROWSER] Interacting with input {} on: {}", input_index, url);
        let tab = self.browser.new_tab()
            .map_err(|e| BrowserError::LaunchFailed(e.to_string()))?;

        tab.set_default_timeout(Duration::from_secs(30));

        tab.navigate_to(url)
            .map_err(|e| BrowserError::NavigationFailed(e.to_string()))?;

        tab.wait_until_navigated()
            .map_err(|e| BrowserError::NavigationFailed(e.to_string()))?;

        let value_json = serde_json::to_string(value.unwrap_or(""))
            .map_err(|e| BrowserError::ExtractionFailed(e.to_string()))?;

        let js = JS_INTERACT
            .replace("__INDEX__", &input_index.to_string())
            .replace("__VALUE__", &value_json);

        tab.evaluate(&js, false)
            .map_err(|e| BrowserError::ExtractionFailed(e.to_string()))?;

        tab.wait_until_navigated()
            .map_err(|e| BrowserError::NavigationFailed(e.to_string()))?;

        extract_page_content(&tab)
    }
}

impl Drop for BrowserInstance {
    fn drop(&mut self) {
        eprintln!("[BROWSER] Shutting down Chrome");
        let _ = self._chrome.kill();
    }
}

// --- JS constants (unchanged from before) ---

const JS_COLLECT: &str = r#"
    (function collectSlots() {
        var out = [];
        var seenRadios = {};
        var els = document.querySelectorAll('input, select, textarea');
        for (var i = 0; i < els.length; i++) {
            var el = els[i];
            if (el.disabled || el.offsetParent === null) continue;
            var tag = el.tagName.toLowerCase();
            var type = (el.getAttribute('type') ||
                (tag === 'select' ? 'select' : tag === 'textarea' ? 'textarea' : 'text')
            ).toLowerCase();
            if (/^(hidden|submit|button|image|file|reset)$/.test(type)) continue;

            function resolveLabel(e) {
                var lbl = e.id ? document.querySelector('label[for="' + e.id + '"]') : null;
                var text = (lbl && lbl.textContent.trim()) ||
                           e.getAttribute('placeholder') || e.name ||
                           e.getAttribute('aria-label') || '';
                return text.trim().substring(0, 50);
            }

            if (type === 'radio') {
                var name = el.name;
                if (!name || seenRadios[name]) continue;
                seenRadios[name] = 1;
                var radios = document.querySelectorAll('input[type="radio"][name="' + name + '"]');
                var opts = [];
                for (var j = 0; j < radios.length; j++) {
                    var r = radios[j];
                    if (r.disabled || r.offsetParent === null) continue;
                    var rl = resolveLabel(r);
                    if (!rl && r.nextSibling && r.nextSibling.nodeType === 3) {
                        rl = r.nextSibling.textContent.trim();
                    }
                    opts.push((rl || r.value).substring(0, 40));
                }
                out.push({kind: 'radio', label: resolveLabel(el) || name, options: opts});
            } else if (tag === 'select') {
                var sopts = [];
                for (var k = 0; k < el.options.length; k++) {
                    var ot = (el.options[k].text || el.options[k].value).trim();
                    if (ot) sopts.push(ot);
                }
                out.push({kind: 'select', label: resolveLabel(el), options: sopts});
            } else if (type === 'checkbox') {
                out.push({kind: 'checkbox', label: resolveLabel(el), checked: el.checked});
            } else {
                out.push({kind: 'text', label: resolveLabel(el)});
            }
        }
        return out;
    })()
"#;

const JS_EXTRACT_INPUTS: &str = r#"JSON.stringify(__COLLECT__)"#;

const JS_INTERACT: &str = r#"
    (function() {
        var seenRadios = {};
        var counter = 0;
        var targetIdx = __INDEX__ - 1;
        var els = document.querySelectorAll('input, select, textarea');

        for (var i = 0; i < els.length; i++) {
            var el = els[i];
            if (el.disabled || el.offsetParent === null) continue;
            var tag = el.tagName.toLowerCase();
            var type = (el.getAttribute('type') ||
                (tag === 'select' ? 'select' : tag === 'textarea' ? 'textarea' : 'text')
            ).toLowerCase();
            if (/^(hidden|submit|button|image|file|reset)$/.test(type)) continue;

            var isRadioSlot = false;
            if (type === 'radio') {
                var name = el.name;
                if (!name || seenRadios[name]) continue;
                seenRadios[name] = 1;
                isRadioSlot = true;
            }

            if (counter === targetIdx) {
                var value = __VALUE__;

                if (type === 'checkbox') {
                    el.checked = !el.checked;
                    el.dispatchEvent(new Event('change', {bubbles: true}));
                } else if (tag === 'select') {
                    var optIdx = parseInt(value, 10) - 1;
                    if (optIdx >= 0 && optIdx < el.options.length) {
                        el.selectedIndex = optIdx;
                        el.dispatchEvent(new Event('change', {bubbles: true}));
                    }
                } else if (isRadioSlot) {
                    var radioEls = Array.from(
                        document.querySelectorAll('input[type="radio"][name="' + el.name + '"]')
                    ).filter(function(r) { return !r.disabled && r.offsetParent !== null; });
                    var rIdx = parseInt(value, 10) - 1;
                    if (rIdx >= 0 && rIdx < radioEls.length) {
                        radioEls[rIdx].checked = true;
                        radioEls[rIdx].dispatchEvent(new Event('change', {bubbles: true}));
                        el = radioEls[rIdx];
                    }
                } else {
                    el.value = value;
                    el.dispatchEvent(new Event('input', {bubbles: true}));
                    el.dispatchEvent(new Event('change', {bubbles: true}));
                }

                var form = el.form || (el.closest ? el.closest('form') : null);
                if (form) {
                    form.submit();
                } else {
                    el.dispatchEvent(new KeyboardEvent('keypress',
                        {key: 'Enter', keyCode: 13, bubbles: true}));
                }
                return true;
            }
            counter++;
        }
        return false;
    })()
"#;

fn extract_page_content(tab: &Arc<Tab>) -> Result<PageContent, BrowserError> {
    let text_result = tab.evaluate(r#"document.body.innerText"#, false)
        .map_err(|e| BrowserError::ExtractionFailed(e.to_string()))?;

    let text = text_result.value
        .and_then(|v| v.as_str().map(|s| s.to_string()))
        .unwrap_or_default();

    let text_lines: Vec<String> = text.lines().map(|l| l.to_string()).collect();

    let links_result = tab.evaluate(r#"
        Array.from(document.querySelectorAll('a[href]'))
            .map(a => a.href)
            .filter(href => href.startsWith('http'))
    "#, false).map_err(|e| BrowserError::ExtractionFailed(e.to_string()))?;

    let links: Vec<(usize, String)> = links_result.value
        .and_then(|v| v.as_array().cloned())
        .unwrap_or_default()
        .into_iter()
        .filter_map(|v| v.as_str().map(|s| s.to_string()))
        .enumerate()
        .map(|(i, url)| (i + 1, url))
        .collect();

    let extract_js = JS_EXTRACT_INPUTS.replace("__COLLECT__", JS_COLLECT);
    let inputs_result = tab.evaluate(&extract_js, false)
        .map_err(|e| BrowserError::ExtractionFailed(e.to_string()))?;

    let inputs_json = inputs_result.value
        .and_then(|v| v.as_str().map(|s| s.to_string()))
        .unwrap_or_else(|| "[]".to_string());

    let inputs = parse_input_fields(&inputs_json);

    Ok(PageContent { text: text_lines, links, inputs })
}

#[derive(serde::Deserialize)]
struct RawInputField {
    kind: String,
    label: String,
    options: Option<Vec<String>>,
    checked: Option<bool>,
}

fn parse_input_fields(json: &str) -> Vec<InputField> {
    let raw: Vec<RawInputField> = serde_json::from_str(json).unwrap_or_default();
    raw.into_iter().enumerate().map(|(i, f)| {
        let kind = match f.kind.as_str() {
            "select"   => InputKind::Select { options: f.options.unwrap_or_default() },
            "radio"    => InputKind::Radio  { options: f.options.unwrap_or_default() },
            "checkbox" => InputKind::Checkbox { checked: f.checked.unwrap_or(false) },
            _          => InputKind::Text,
        };
        InputField { index: i + 1, label: f.label, kind }
    }).collect()
}
