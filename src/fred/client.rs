//! FRED API client with rate limiting and request pooling.

use std::sync::Arc;
use std::time::Duration;

use reqwest::{Client, StatusCode};
use tracing::debug;

use crate::error::{FinanceError, Result};
use crate::fred::models::{MacroObservation, MacroSeries};
use crate::rate_limiter::RateLimiter;

const FRED_BASE: &str = "https://api.stlouisfed.org/fred";
const DEFAULT_TIMEOUT: Duration = Duration::from_secs(30);

pub(crate) struct FredClientBuilder {
    api_key: String,
    timeout: Duration,
}

impl FredClientBuilder {
    pub fn new(api_key: impl Into<String>) -> Self {
        Self {
            api_key: api_key.into(),
            timeout: DEFAULT_TIMEOUT,
        }
    }

    pub fn timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }

    /// Build using a shared `Arc<RateLimiter>` instead of creating a new one.
    ///
    /// Used by the module singleton to share a single rate-limiter across fresh
    /// HTTP clients: the `reqwest::Client` is runtime-bound and must be rebuilt
    /// per request, but the `RateLimiter` state must persist across calls so the
    /// 2 req/sec FRED limit is respected.
    pub(super) fn build_with_limiter(self, limiter: Arc<RateLimiter>) -> Result<FredClient> {
        let http = Client::builder()
            .timeout(self.timeout)
            .user_agent(format!(
                "finance-query/{} (https://github.com/Verdenroz/finance-query)",
                env!("CARGO_PKG_VERSION")
            ))
            .build()
            .map_err(FinanceError::HttpError)?;

        Ok(FredClient {
            api_key: self.api_key,
            http,
            limiter,
        })
    }
}

/// FRED API client. Constructed per-call via [`super::FRED_SINGLETON`].
pub(crate) struct FredClient {
    api_key: String,
    http: Client,
    limiter: Arc<RateLimiter>,
}

impl FredClient {
    /// Fetch all observations for a FRED series by ID (e.g., `"FEDFUNDS"`, `"CPIAUCSL"`).
    pub async fn series(&self, series_id: &str) -> Result<MacroSeries> {
        self.limiter.acquire().await;

        let url = format!(
            "{FRED_BASE}/series/observations?series_id={series_id}&api_key={}&file_type=json",
            self.api_key
        );

        debug!("FRED request: series_id={series_id}");
        let resp = self
            .http
            .get(&url)
            .send()
            .await
            .map_err(FinanceError::HttpError)?;

        match resp.status() {
            StatusCode::OK => {}
            StatusCode::BAD_REQUEST => {
                return Err(FinanceError::InvalidParameter {
                    param: "series_id".to_string(),
                    reason: format!("FRED series '{series_id}' not found or invalid"),
                });
            }
            StatusCode::UNAUTHORIZED | StatusCode::FORBIDDEN => {
                return Err(FinanceError::AuthenticationFailed {
                    context: "FRED API key invalid or missing. Call fred::init(key) first."
                        .to_string(),
                });
            }
            StatusCode::TOO_MANY_REQUESTS => {
                return Err(FinanceError::RateLimited {
                    retry_after: Some(60),
                });
            }
            s => {
                return Err(FinanceError::ExternalApiError {
                    api: "FRED".to_string(),
                    status: s.as_u16(),
                });
            }
        }

        let json: serde_json::Value = resp.json().await.map_err(FinanceError::HttpError)?;

        let observations = json
            .get("observations")
            .and_then(|v| v.as_array())
            .ok_or_else(|| FinanceError::ResponseStructureError {
                field: "observations".to_string(),
                context: "FRED response missing observations array".to_string(),
            })?
            .iter()
            .filter_map(|obs| {
                let date = obs.get("date")?.as_str()?.to_string();
                let raw = obs.get("value")?.as_str()?;
                // FRED uses "." for missing values
                let value = if raw == "." {
                    None
                } else {
                    raw.parse::<f64>().ok()
                };
                Some(MacroObservation { date, value })
            })
            .collect();

        Ok(MacroSeries {
            id: series_id.to_string(),
            observations,
        })
    }
}
