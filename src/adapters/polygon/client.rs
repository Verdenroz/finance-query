//! Polygon.io API client with rate limiting and cursor-based pagination.

use std::sync::Arc;
use std::time::Duration;

use reqwest::{Client, StatusCode};
use serde::de::DeserializeOwned;
use serde_json::Value;
use tracing::debug;

use crate::error::{FinanceError, Result};
use crate::rate_limiter::RateLimiter;

use super::models::PaginatedResponse;

const PG_BASE: &str = "https://api.polygon.io";
const DEFAULT_TIMEOUT: Duration = Duration::from_secs(30);

pub(crate) struct PolygonClientBuilder {
    api_key: String,
    timeout: Duration,
    base_url: Option<String>,
}

impl PolygonClientBuilder {
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

    #[cfg(test)]
    pub fn base_url(mut self, url: impl Into<String>) -> Self {
        self.base_url = Some(url.into());
        self
    }

    pub(super) fn build_with_limiter(self, limiter: Arc<RateLimiter>) -> Result<PolygonClient> {
        let http = Client::builder()
            .timeout(self.timeout)
            .user_agent(format!(
                "finance-query/{} (https://github.com/Verdenroz/finance-query)",
                env!("CARGO_PKG_VERSION")
            ))
            .build()?;

        Ok(PolygonClient {
            api_key: self.api_key,
            http,
            limiter,
            base_url: self.base_url.unwrap_or_else(|| PG_BASE.to_string()),
        })
    }
}

/// Polygon.io API client. Constructed per-call via the module singleton.
pub(crate) struct PolygonClient {
    api_key: String,
    http: Client,
    limiter: Arc<RateLimiter>,
    base_url: String,
}

impl PolygonClient {
    fn check_status(status: StatusCode) -> Result<()> {
        match status {
            StatusCode::OK => Ok(()),
            StatusCode::UNAUTHORIZED | StatusCode::FORBIDDEN => {
                Err(FinanceError::AuthenticationFailed {
                    context: "Polygon API key invalid or missing. Call polygon::init(key) first."
                        .to_string(),
                })
            }
            StatusCode::NOT_FOUND => Err(FinanceError::SymbolNotFound {
                symbol: None,
                context: "Resource not found on Polygon".to_string(),
            }),
            StatusCode::TOO_MANY_REQUESTS => Err(FinanceError::RateLimited {
                retry_after: Some(60),
            }),
            s if s.is_server_error() => Err(FinanceError::ServerError {
                status: s.as_u16(),
                context: "Polygon server error".to_string(),
            }),
            s => Err(FinanceError::ExternalApiError {
                api: "Polygon".to_string(),
                status: s.as_u16(),
            }),
        }
    }

    fn check_body_error(json: &Value) -> Result<()> {
        if let Some(status) = json.get("status").and_then(|v| v.as_str())
            && (status == "ERROR" || status == "NOT_FOUND")
        {
            let msg = json
                .get("error")
                .or_else(|| json.get("message"))
                .and_then(|v| v.as_str())
                .unwrap_or("Unknown error");
            if status == "NOT_FOUND" {
                return Err(FinanceError::SymbolNotFound {
                    symbol: None,
                    context: msg.to_string(),
                });
            }
            return Err(FinanceError::ExternalApiError {
                api: "Polygon".to_string(),
                status: 400,
            });
        }
        Ok(())
    }

    /// Execute a GET request to a Polygon REST path and return raw JSON.
    pub async fn get_raw(&self, path: &str, params: &[(&str, &str)]) -> Result<Value> {
        self.limiter.acquire().await;

        let url = format!("{}{}", self.base_url, path);
        let mut query: Vec<(&str, &str)> = vec![("apiKey", &self.api_key)];
        query.extend_from_slice(params);

        debug!("Polygon request: {path}");
        let resp = self.http.get(&url).query(&query).send().await?;

        Self::check_status(resp.status())?;

        let json: Value = resp.json().await?;
        Self::check_body_error(&json)?;

        Ok(json)
    }

    /// GET and deserialize into a `PaginatedResponse<T>`.
    pub async fn get<T: DeserializeOwned>(
        &self,
        path: &str,
        params: &[(&str, &str)],
    ) -> Result<PaginatedResponse<T>> {
        let json = self.get_raw(path, params).await?;
        serde_json::from_value(json).map_err(|e| FinanceError::ResponseStructureError {
            field: "response".to_string(),
            context: format!("Failed to deserialize Polygon response: {e}"),
        })
    }
}
