//! Senate eFD headless-Chromium browser client.
//!
//! `efdsearch.senate.gov` has no query API — search results and each
//! filing's transaction table render client-side, behind Akamai bot
//! protection. A vanilla headless Chromium needs specific launch flags to
//! pass it. The block is also IP-reputation-based: the same flags pass from
//! a residential IP and fail (403, `errors.edgesuite.net`) from a VPN or
//! cloud/VPS IP. Repeated requests in a short window can also trigger a
//! separate, IP-independent rate block. This adapter can't detect or work
//! around either — a blocked or rate-limited caller sees every fetch fail,
//! which [`crate::providers::congresstrades`] treats as zero rows from this
//! source, not a hard failure.

use std::time::Duration;

use chromiumoxide::browser::{Browser, BrowserConfig};
use chromiumoxide::cdp::browser_protocol::target::CreateTargetParams;
use chromiumoxide::element::Element;
use chromiumoxide::page::Page;
use futures::StreamExt;

use crate::error::{FinanceError, Result};

pub(super) const SENATE_BASE: &str = "https://efdsearch.senate.gov";

const USER_AGENT: &str = "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/151.0.0.0 Safari/537.36";

const ELEMENT_POLL_INTERVAL: Duration = Duration::from_millis(200);
const ELEMENT_POLL_ATTEMPTS: usize = 75;

fn browser_error(e: impl std::fmt::Display) -> FinanceError {
    FinanceError::ApiError(format!("Senate eFD: {e}"))
}

pub(super) struct SenateTradesClient {
    browser: Browser,
    handler: tokio::task::JoinHandle<()>,
}

impl SenateTradesClient {
    /// Launch a fresh headless Chromium instance for one fetch. Not pooled —
    /// each call to `fetch_congressional_trades_response` gets its own
    /// browser/session, closed at the end of that call.
    pub(super) async fn launch() -> Result<Self> {
        let config = BrowserConfig::builder()
            .no_sandbox()
            .arg("--disable-blink-features=AutomationControlled")
            .arg("--headless=new")
            .arg(format!("--user-agent={USER_AGENT}"))
            .build()
            .map_err(browser_error)?;
        let (browser, mut handler) = Browser::launch(config).await.map_err(browser_error)?;
        let handler = tokio::spawn(async move {
            while let Some(event) = handler.next().await {
                if event.is_err() {
                    break;
                }
            }
        });
        Ok(Self { browser, handler })
    }

    pub(super) async fn new_page(&self, url: &str) -> Result<Page> {
        self.browser
            .new_page(CreateTargetParams::new(url))
            .await
            .map_err(browser_error)
    }

    /// Close the browser and wait for its event-forwarding task to exit.
    pub(super) async fn close(mut self) {
        self.browser.close().await.ok();
        self.browser.wait().await.ok();
        let _ = (&mut self.handler).await;
    }
}

/// Poll for one element until it exists, since the results and report tables
/// render asynchronously after the page load completes.
pub(super) async fn wait_for_element(page: &Page, selector: &str) -> Result<Element> {
    for _ in 0..ELEMENT_POLL_ATTEMPTS {
        if let Ok(element) = page.find_element(selector).await {
            return Ok(element);
        }
        tokio::time::sleep(ELEMENT_POLL_INTERVAL).await;
    }
    Err(browser_error(format!("timed out waiting for {selector}")))
}

/// Poll for at least one matching element, for tables whose rows arrive via
/// an AJAX call after the initial (empty) render.
pub(super) async fn wait_for_elements(page: &Page, selector: &str) -> Result<Vec<Element>> {
    for _ in 0..ELEMENT_POLL_ATTEMPTS {
        if let Ok(elements) = page.find_elements(selector).await
            && !elements.is_empty()
        {
            return Ok(elements);
        }
        tokio::time::sleep(ELEMENT_POLL_INTERVAL).await;
    }
    Err(browser_error(format!("timed out waiting for {selector}")))
}
