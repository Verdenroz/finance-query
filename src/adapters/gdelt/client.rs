//! GDELT DOC 2.0 API HTTP client.
//!
//! Keyless and unauthenticated. GDELT documents no formal quota for the DOC
//! API but asks callers to keep requests to roughly one every 5 seconds, so
//! the client paces itself to that rate rather than waiting to be throttled.

use std::sync::Arc;
use std::time::Duration;

use reqwest::{Client, StatusCode};
use tracing::debug;

use super::models::GdeltDocResponse;
use crate::adapters::common::keyless_http_client;
use crate::error::{FinanceError, Result};
use crate::rate_limiter::RateLimiter;

pub(super) const GDELT_BASE: &str = "https://api.gdeltproject.org/api/v2/doc/doc";

pub(super) struct GdeltClient {
    http: Client,
    limiter: Arc<RateLimiter>,
    base_url: String,
}

impl GdeltClient {
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

    /// Fetch up to `max_records` articles matching `query` within the last
    /// `timespan` (a GDELT duration like `"2w"`).
    pub(super) async fn article_search(
        &self,
        query: &str,
        timespan: &str,
        max_records: u32,
    ) -> Result<GdeltDocResponse> {
        self.limiter.acquire().await;

        debug!("GDELT request: {query}");

        let resp = self
            .http
            .get(&self.base_url)
            .query(&[
                ("query", query),
                ("mode", "artlist"),
                ("format", "json"),
                ("timespan", timespan),
                ("maxrecords", &max_records.to_string()),
            ])
            .send()
            .await?;
        let status = resp.status();
        let bytes = resp.bytes().await?;

        if !status.is_success() {
            return Err(Self::map_error(status));
        }

        // GDELT answers throttled/malformed requests with a plain-text
        // message rather than JSON (no content-type negotiation on error),
        // so a parse failure surfaces that text instead of a raw serde error.
        serde_json::from_slice(&bytes).map_err(|_| {
            let text = String::from_utf8_lossy(&bytes).trim().to_string();
            FinanceError::ResponseStructureError {
                field: "gdelt.articles".to_string(),
                context: if text.is_empty() {
                    "unrecognised GDELT response".to_string()
                } else {
                    text
                },
            }
        })
    }

    fn map_error(status: StatusCode) -> FinanceError {
        match status {
            StatusCode::TOO_MANY_REQUESTS => FinanceError::RateLimited { retry_after: None },
            s => FinanceError::ExternalApiError {
                api: "GDELT".to_string(),
                status: s.as_u16(),
            },
        }
    }
}
