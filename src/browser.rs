use headless_chrome::{Browser, LaunchOptions};
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
    pub links: Vec<(usize, String)>, // (index, url)
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

        // Extract text content
        let text_result = tab.evaluate(
            r#"document.body.innerText"#,
            false
        ).map_err(|e| BrowserError::ExtractionFailed(e.to_string()))?;

        let text = text_result.value
            .and_then(|v| v.as_str().map(|s| s.to_string()))
            .unwrap_or_default();

        let text_lines: Vec<String> = text.lines().map(|l| l.to_string()).collect();

        // Extract links
        let links_result = tab.evaluate(
            r#"
            Array.from(document.querySelectorAll('a[href]'))
                .map(a => a.href)
                .filter(href => href.startsWith('http'))
            "#,
            false
        ).map_err(|e| BrowserError::ExtractionFailed(e.to_string()))?;

        let links: Vec<(usize, String)> = links_result.value
            .and_then(|v| v.as_array().cloned())
            .unwrap_or_default()
            .into_iter()
            .filter_map(|v| v.as_str().map(|s| s.to_string()))
            .enumerate()
            .map(|(i, url)| (i + 1, url))
            .collect();

        Ok(PageContent { text: text_lines, links })
    }
}
