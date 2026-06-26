//! FRED API client with rate limiting and request pooling.

use std::sync::Arc;
use std::time::Duration;

use reqwest::{Client, StatusCode};
use tracing::debug;

use super::models::{MacroObservation, MacroSeries, ReleaseDate};
use crate::error::{FinanceError, Result};
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
            .build()?;

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
        let resp = self.http.get(&url).send().await?;

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

        let json: serde_json::Value = resp.json().await?;

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

    /// Fetch upcoming scheduled economic-data release dates.
    ///
    /// Queries the FRED `releases/dates` endpoint with a realtime window from
    /// `today` onward so only future-scheduled releases (CPI, NFP, GDP, FOMC,
    /// etc.) are returned, sorted ascending by date.
    pub async fn release_dates(&self, today: &str) -> Result<Vec<ReleaseDate>> {
        self.limiter.acquire().await;

        let url = format!(
            "{FRED_BASE}/releases/dates?api_key={}&file_type=json\
             &include_release_dates_with_no_data=true&sort_order=asc\
             &realtime_start={today}&realtime_end=9999-12-31",
            self.api_key
        );

        debug!("FRED request: releases/dates from {today}");
        let resp = self.http.get(&url).send().await?;

        match resp.status() {
            StatusCode::OK => {}
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

        let json: serde_json::Value = resp.json().await?;

        let dates = json
            .get("release_dates")
            .and_then(|v| v.as_array())
            .ok_or_else(|| FinanceError::ResponseStructureError {
                field: "release_dates".to_string(),
                context: "FRED response missing release_dates array".to_string(),
            })?
            .iter()
            .filter_map(|rd| {
                Some(ReleaseDate {
                    release_id: rd.get("release_id")?.as_u64()?,
                    release_name: rd.get("release_name")?.as_str()?.to_string(),
                    date: rd.get("date")?.as_str()?.to_string(),
                })
            })
            .collect();

        Ok(dates)
    }
}
