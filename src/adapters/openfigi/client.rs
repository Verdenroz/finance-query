//! OpenFIGI v3 mapping HTTP client.
//!
//! Keyless by default. `OPENFIGI_API_KEY` is optional and only raises the
//! quota — the endpoint and the response shape are identical either way.

use std::sync::Arc;
use std::time::Duration;

use reqwest::{Client, StatusCode};
use tracing::debug;

use super::models::{MappingJob, MappingResult};
use crate::adapters::common::{keyless_http_client, status_error};
use crate::error::{FinanceError, Result};
use crate::rate_limiter::RateLimiter;

pub(super) const OPENFIGI_BASE: &str = "https://api.openfigi.com/v3";

/// Jobs per request without an API key.
pub(super) const ANONYMOUS_MAX_JOBS_PER_REQUEST: usize = 10;
/// Jobs per request with an API key.
pub(super) const AUTHENTICATED_MAX_JOBS_PER_REQUEST: usize = 100;

pub(super) struct OpenFigiClient {
    http: Client,
    limiter: Arc<RateLimiter>,
    base_url: String,
    api_key: Option<String>,
}

impl OpenFigiClient {
    pub(super) fn max_jobs_per_request(&self) -> usize {
        if self.api_key.is_some() {
            AUTHENTICATED_MAX_JOBS_PER_REQUEST
        } else {
            ANONYMOUS_MAX_JOBS_PER_REQUEST
        }
    }

    pub(super) fn new(
        timeout: Duration,
        limiter: Arc<RateLimiter>,
        base_url: impl Into<String>,
        api_key: Option<String>,
    ) -> Result<Self> {
        Ok(Self {
            http: keyless_http_client(timeout)?,
            limiter,
            base_url: base_url.into(),
            api_key,
        })
    }

    /// Submit one batch of mapping jobs.
    ///
    /// The response array is positional: element `i` answers job `i`. Callers
    /// rely on that to pair results back to their identifiers, so the length
    /// is validated rather than assumed.
    pub(super) async fn map(&self, jobs: &[MappingJob<'_>]) -> Result<Vec<MappingResult>> {
        self.limiter.acquire().await;

        let url = format!("{}/mapping", self.base_url);
        debug!("OpenFIGI request: {} job(s)", jobs.len());

        let mut request = self.http.post(&url).json(jobs);
        if let Some(key) = &self.api_key {
            request = request.header("X-OPENFIGI-APIKEY", key);
        }

        let resp = request.send().await?;
        let status = resp.status();
        let bytes = resp.bytes().await?;

        if !status.is_success() {
            return Err(match status {
                // OpenFIGI rejects an oversized batch with 413.
                StatusCode::PAYLOAD_TOO_LARGE => FinanceError::InvalidParameter {
                    param: "identifiers".to_string(),
                    reason: format!(
                        "OpenFIGI accepts at most {} identifiers per request for this tier",
                        self.max_jobs_per_request()
                    ),
                },
                s => status_error("OpenFIGI", s),
            });
        }

        let results: Vec<MappingResult> =
            serde_json::from_slice(&bytes).map_err(|e| FinanceError::ResponseStructureError {
                field: "openfigi.mapping".to_string(),
                context: format!("unrecognised OpenFIGI payload: {e}"),
            })?;

        if results.len() != jobs.len() {
            return Err(FinanceError::ResponseStructureError {
                field: "openfigi.mapping".to_string(),
                context: format!(
                    "OpenFIGI returned {} results for {} jobs; results are positional and \
                     cannot be paired back",
                    results.len(),
                    jobs.len()
                ),
            });
        }
        Ok(results)
    }
}
