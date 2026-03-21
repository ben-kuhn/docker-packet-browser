use headless_chrome::{Browser, LaunchOptions, Tab};
use std::sync::Arc;
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

pub struct PageContent {
    pub text: Vec<String>,
    pub links: Vec<(usize, String)>,   // (index, url)
    pub inputs: Vec<(usize, String)>,  // (index, label/placeholder)
}

pub struct BrowserInstance {
    browser: Browser,
}

impl BrowserInstance {
    pub fn new() -> Result<Self, BrowserError> {
        let launch_options = LaunchOptions::default_builder()
            .headless(true)
            .sandbox(false) // Sandbox requires kernel user namespaces; disabled for container security model
            .build()
            .map_err(|e| BrowserError::LaunchFailed(e.to_string()))?;

        let browser = Browser::new(launch_options)
            .map_err(|e| BrowserError::LaunchFailed(e.to_string()))?;

        Ok(Self { browser })
    }

    pub fn fetch_page(&self, url: &str) -> Result<PageContent, BrowserError> {
        let tab = self.browser.new_tab()
            .map_err(|e| BrowserError::LaunchFailed(e.to_string()))?;

        tab.navigate_to(url)
            .map_err(|e| BrowserError::NavigationFailed(e.to_string()))?;

        tab.wait_until_navigated()
            .map_err(|e| BrowserError::NavigationFailed(e.to_string()))?;

        extract_page_content(&tab)
    }

    pub fn fill_and_submit(&self, url: &str, input_index: usize, value: &str) -> Result<PageContent, BrowserError> {
        let tab = self.browser.new_tab()
            .map_err(|e| BrowserError::LaunchFailed(e.to_string()))?;

        tab.navigate_to(url)
            .map_err(|e| BrowserError::NavigationFailed(e.to_string()))?;

        tab.wait_until_navigated()
            .map_err(|e| BrowserError::NavigationFailed(e.to_string()))?;

        let escaped = serde_json::to_string(value)
            .map_err(|e| BrowserError::ExtractionFailed(e.to_string()))?;

        let js = format!(r#"
            (function() {{
                const inputs = Array.from(document.querySelectorAll(
                    'input[type="text"], input[type="search"], input[type="email"], input[type="number"], input:not([type]), textarea'
                )).filter(el => !el.disabled && el.offsetParent !== null);
                const el = inputs[{}];
                if (!el) return false;
                el.value = {};
                el.dispatchEvent(new Event('input', {{bubbles: true}}));
                el.dispatchEvent(new Event('change', {{bubbles: true}}));
                if (el.form) {{
                    el.form.submit();
                }} else {{
                    el.dispatchEvent(new KeyboardEvent('keypress', {{key: 'Enter', keyCode: 13, bubbles: true}}));
                }}
                return true;
            }})()
        "#, input_index - 1, escaped);

        tab.evaluate(&js, false)
            .map_err(|e| BrowserError::ExtractionFailed(e.to_string()))?;

        tab.wait_until_navigated()
            .map_err(|e| BrowserError::NavigationFailed(e.to_string()))?;

        extract_page_content(&tab)
    }
}

fn extract_page_content(tab: &Arc<Tab>) -> Result<PageContent, BrowserError> {
    // Text
    let text_result = tab.evaluate(r#"document.body.innerText"#, false)
        .map_err(|e| BrowserError::ExtractionFailed(e.to_string()))?;

    let text = text_result.value
        .and_then(|v| v.as_str().map(|s| s.to_string()))
        .unwrap_or_default();

    let text_lines: Vec<String> = text.lines().map(|l| l.to_string()).collect();

    // Links
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

    // Input fields
    let inputs_result = tab.evaluate(r#"
        Array.from(document.querySelectorAll(
            'input[type="text"], input[type="search"], input[type="email"], input[type="number"], input:not([type]), textarea'
        ))
        .filter(el => !el.disabled && el.offsetParent !== null)
        .map((el, i) => {
            const label = (document.querySelector(`label[for="${el.id}"]`) || {}).textContent
                || el.placeholder || el.name || el.getAttribute('aria-label') || ('Field ' + (i + 1));
            return label.trim().substring(0, 50);
        })
    "#, false).map_err(|e| BrowserError::ExtractionFailed(e.to_string()))?;

    let inputs: Vec<(usize, String)> = inputs_result.value
        .and_then(|v| v.as_array().cloned())
        .unwrap_or_default()
        .into_iter()
        .filter_map(|v| v.as_str().map(|s| s.to_string()))
        .enumerate()
        .map(|(i, label)| (i + 1, label))
        .collect();

    Ok(PageContent { text: text_lines, links, inputs })
}
