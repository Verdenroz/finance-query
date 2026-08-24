//! Wikipedia article HTML fetch.

use std::sync::Arc;
use std::time::Duration;

use reqwest::Client;

use crate::adapters::common::{check_status, keyless_http_client};
use crate::error::Result;
use crate::rate_limiter::RateLimiter;

pub(super) const WIKIPEDIA_BASE: &str = "https://en.wikipedia.org/wiki";

pub(super) struct WikipediaClient {
    http: Client,
    limiter: Arc<RateLimiter>,
    base_url: String,
}

impl WikipediaClient {
    pub(super) fn new(
        timeout: Duration,
        limiter: Arc<RateLimiter>,
        base_url: impl Into<String>,
    ) -> Result<Self> {
        Ok(Self {
            http: keyless_http_client(timeout)?,
            limiter,
            base_url: base_url.into(),
        })
    }

    /// Fetch an article's rendered HTML by title (e.g. `"List_of_S%26P_500_companies"`).
    pub(super) async fn page_html(&self, title: &str) -> Result<String> {
        self.limiter.acquire().await;
        let url = format!("{}/{title}", self.base_url);
        let resp = self.http.get(&url).send().await?;
        check_status("Wikipedia", resp.status())?;
        Ok(resp.text().await?)
    }
}
