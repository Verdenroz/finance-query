//! FRED API client with rate limiting and request pooling.

use std::sync::Arc;
use std::time::Duration;

use reqwest::{Client, StatusCode};
use tracing::debug;

use super::models::{MacroObservation, MacroSeries, ReleaseDate};
use crate::adapters::common::keyed::{redact_key, transport_error};
use crate::error::{FinanceError, Result};
use crate::rate_limiter::RateLimiter;

const FRED_BASE: &str = "https://api.stlouisfed.org/fred";
const DEFAULT_TIMEOUT: Duration = Duration::from_secs(30);

pub(crate) struct FredClientBuilder {
    api_key: String,
    timeout: Duration,
    base_url: Option<String>,
}

impl FredClientBuilder {
    pub fn new(api_key: impl Into<String>) -> Self {
        Self {
            api_key: api_key.into(),
            timeout: DEFAULT_TIMEOUT,
            base_url: None,
        }
    }

    pub fn timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }

    /// Override the API base URL (used by tests to point at a mock server).
    #[cfg(test)]
    pub fn base_url(mut self, url: impl Into<String>) -> Self {
        self.base_url = Some(url.into());
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
            timeout: self.timeout,
            base_url: self.base_url.unwrap_or_else(|| FRED_BASE.to_string()),
        })
    }
}

/// FRED API client. Constructed per-call via [`super::FRED_SINGLETON`].
pub(crate) struct FredClient {
    api_key: String,
    http: Client,
    limiter: Arc<RateLimiter>,
    timeout: Duration,
    base_url: String,
}

impl FredClient {
    fn response_error(status: StatusCode, body: &[u8], api_key: &str) -> FinanceError {
        let message = serde_json::from_slice::<FredErrorEnvelope>(body)
            .ok()
            .and_then(|error| error.error_message)
            .map(|message| redact_key(&message, api_key))
            .unwrap_or_else(|| format!("FRED returned HTTP {status}"));
        let normalized = message.to_ascii_lowercase();
        if normalized.contains("api_key") || normalized.contains("api key") {
            return FinanceError::AuthenticationFailed { context: message };
        }
        match status {
            StatusCode::BAD_REQUEST => FinanceError::InvalidParameter {
                param: "request".to_string(),
                reason: message,
            },
            StatusCode::UNAUTHORIZED | StatusCode::FORBIDDEN => {
                FinanceError::AuthenticationFailed { context: message }
            }
            StatusCode::TOO_MANY_REQUESTS | StatusCode::LOCKED => FinanceError::RateLimited {
                retry_after: Some(60),
            },
            s if s.is_server_error() => FinanceError::ServerError {
                status: s.as_u16(),
                context: message,
            },
            s => FinanceError::ExternalApiError {
                api: "FRED".to_string(),
                status: s.as_u16(),
            },
        }
    }

    async fn decode_json<T: serde::de::DeserializeOwned>(
        &self,
        response: reqwest::Response,
        field: &str,
    ) -> Result<T> {
        let status = response.status();
        let bytes = response
            .bytes()
            .await
            .map_err(|error| self.map_transport_error(&error))?;
        if !status.is_success() {
            return Err(Self::response_error(status, &bytes, &self.api_key));
        }
        serde_json::from_slice(&bytes).map_err(|error| FinanceError::ResponseStructureError {
            field: field.to_string(),
            context: format!("Failed to deserialize FRED response: {error}"),
        })
    }

    /// Rate-limited GET against a FRED path, deserialized into `T`.
    ///
    /// The api key and `file_type=json` are always appended, so callers pass
    /// only the endpoint-specific parameters.
    pub async fn get_json<T: serde::de::DeserializeOwned>(
        &self,
        path: &str,
        params: &[(&str, &str)],
    ) -> Result<T> {
        self.limiter.acquire().await;

        let url = format!("{}/{path}", self.base_url);
        let mut query: Vec<(&str, &str)> = vec![("api_key", &self.api_key), ("file_type", "json")];
        query.extend_from_slice(params);

        debug!("FRED request: {path}");
        let response = self
            .http
            .get(&url)
            .query(&query)
            .send()
            .await
            .map_err(|error| self.map_transport_error(&error))?;
        self.decode_json(response, path).await
    }

    fn map_transport_error(&self, error: &reqwest::Error) -> FinanceError {
        transport_error("FRED", self.timeout, error)
    }

    /// Fetch all observations for a FRED series by ID (e.g., `"FEDFUNDS"`, `"CPIAUCSL"`).
    pub async fn series(&self, series_id: &str) -> Result<MacroSeries> {
        self.observations(series_id, &[]).await
    }

    /// Fetch only the most recent observation for a series.
    ///
    /// Pollers reading one value would otherwise download the whole history.
    pub async fn latest_observation(&self, series_id: &str) -> Result<Option<MacroObservation>> {
        let series = self
            .observations(series_id, &[("sort_order", "desc"), ("limit", "1")])
            .await?;
        Ok(series.observations.into_iter().next())
    }

    /// Query `series/observations` with optional endpoint parameters.
    async fn observations(&self, series_id: &str, extra: &[(&str, &str)]) -> Result<MacroSeries> {
        let mut params = vec![("series_id", series_id)];
        params.extend_from_slice(extra);
        let json: serde_json::Value = self.get_json("series/observations", &params).await?;

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
    pub async fn release_dates(&self, today: &str) -> Result<Vec<ReleaseDate>> {
        let json: serde_json::Value = self
            .get_json(
                "releases/dates",
                &[
                    ("include_release_dates_with_no_data", "true"),
                    ("sort_order", "asc"),
                    ("realtime_start", today),
                    ("realtime_end", "9999-12-31"),
                ],
            )
            .await?;

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

#[derive(serde::Deserialize)]
struct FredErrorEnvelope {
    error_message: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn client(api_key: &str, base_url: &str) -> FredClient {
        FredClientBuilder::new(api_key)
            .timeout(Duration::from_secs(5))
            .base_url(base_url)
            .build_with_limiter(Arc::new(RateLimiter::new(100.0)))
            .unwrap()
    }

    #[tokio::test]
    async fn errors_never_render_the_api_key() {
        const KEY: &str = "SUPERSECRETKEY123";

        let mut server = mockito::Server::new_async().await;
        let _mock = server
            .mock("GET", "/series/observations")
            .match_query(mockito::Matcher::Any)
            .with_status(400)
            .with_header("content-type", "application/json")
            .with_body(format!(
                r#"{{"error_code":400,"error_message":"api_key {KEY} is not registered"}}"#
            ))
            .create_async()
            .await;

        let echoed = client(KEY, &server.url()).series("GDP").await.unwrap_err();
        let unreachable = client(KEY, "http://127.0.0.1:1")
            .series("GDP")
            .await
            .unwrap_err();

        for err in [echoed, unreachable] {
            assert!(!format!("{err}").contains(KEY), "{err}");
            assert!(!format!("{err:?}").contains(KEY), "{err:?}");
        }
    }
}
