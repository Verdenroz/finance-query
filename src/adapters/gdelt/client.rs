//! GDELT DOC 2.0 API HTTP client.
//!
//! Keyless and unauthenticated. GDELT documents no formal quota for the DOC
//! API but asks callers to keep requests to roughly one every 5 seconds, so
//! the client paces itself to that rate rather than waiting to be throttled.

use std::sync::Arc;
use std::time::Duration;

use reqwest::{Client, StatusCode};
use tracing::{debug, warn};

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
            return Err(Self::map_error(status, &bytes));
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

    /// GDELT states its throttle in a plain-text body ("Please limit requests
    /// to one every 5 seconds…") rather than a `Retry-After` header, so the
    /// interval is recovered from the text and the body logged — staying
    /// `RateLimited` keeps `is_retriable` and any retry policy working.
    fn map_error(status: StatusCode, body: &[u8]) -> FinanceError {
        match status {
            StatusCode::TOO_MANY_REQUESTS => {
                let text = String::from_utf8_lossy(body);
                let text = text.trim();
                if !text.is_empty() {
                    warn!("GDELT throttled the request: {text}");
                }
                FinanceError::RateLimited {
                    retry_after: parse_throttle_seconds(text),
                }
            }
            s => FinanceError::ExternalApiError {
                api: "GDELT".to_string(),
                status: s.as_u16(),
            },
        }
    }
}

/// Pull the retry interval out of GDELT's throttle text, which reads
/// "Please limit requests to one every 5 seconds…".
fn parse_throttle_seconds(text: &str) -> Option<u64> {
    let rest = text.split_once("one every")?.1;
    let (num, _) = rest.trim_start().split_once(' ')?;
    num.parse().ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn throttle_interval_is_recovered_from_the_message() {
        assert_eq!(
            parse_throttle_seconds("Please limit requests to one every 5 seconds or contact x"),
            Some(5)
        );
    }

    #[test]
    fn an_unrecognised_throttle_message_yields_no_interval() {
        assert_eq!(parse_throttle_seconds("slow down"), None);
        assert_eq!(parse_throttle_seconds(""), None);
    }

    #[test]
    fn throttle_maps_to_rate_limited_with_the_parsed_interval() {
        let err = GdeltClient::map_error(
            StatusCode::TOO_MANY_REQUESTS,
            b"Please limit requests to one every 5 seconds.",
        );
        assert!(matches!(
            err,
            FinanceError::RateLimited {
                retry_after: Some(5)
            }
        ));
    }
}
