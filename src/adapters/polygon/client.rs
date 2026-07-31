//! Polygon.io API client with rate limiting and cursor-based pagination.

use std::sync::Arc;
use std::time::Duration;

use reqwest::{Client, StatusCode};
use serde::Deserialize;
use serde::de::DeserializeOwned;
#[cfg(test)]
use serde_json::Value;
use tracing::debug;

use crate::error::{FinanceError, Result};
use crate::rate_limiter::RateLimiter;

use super::models::PaginatedResponseDTO;

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

    /// Retained for the error-parity test; `check_error_envelope` is the live path.
    fn check_error_envelope(env: &ErrorEnvelope) -> Result<()> {
        let Some(status) = env.status.as_deref() else {
            return Ok(());
        };
        if status != "ERROR" && status != "NOT_FOUND" {
            return Ok(());
        }
        let msg = env
            .error
            .as_ref()
            .and_then(|v| v.as_str())
            .or_else(|| env.message.as_ref().and_then(|v| v.as_str()))
            .unwrap_or("Unknown error");
        if status == "NOT_FOUND" {
            return Err(FinanceError::SymbolNotFound {
                symbol: None,
                context: msg.to_string(),
            });
        }
        Err(FinanceError::ExternalApiError {
            api: "Polygon".to_string(),
            status: 400,
        })
    }

    /// Execute a GET request to a Polygon REST path and return the raw response bytes.
    async fn get_bytes(&self, path: &str, params: &[(&str, &str)]) -> Result<impl AsRef<[u8]>> {
        self.limiter.acquire().await;

        let url = format!("{}{}", self.base_url, path);
        let mut query: Vec<(&str, &str)> = vec![("apiKey", &self.api_key)];
        query.extend_from_slice(params);

        debug!("Polygon request: {path}");
        let resp = self.http.get(&url).query(&query).send().await?;

        Self::check_status(resp.status())?;

        Ok(resp.bytes().await?)
    }

    /// Execute a GET request to a Polygon REST path and return raw JSON.
    #[cfg(test)]
    pub async fn get_raw(&self, path: &str, params: &[(&str, &str)]) -> Result<Value> {
        let bytes = self.get_bytes(path, params).await?;
        if let Ok(env) = serde_json::from_slice::<ErrorEnvelope>(bytes.as_ref()) {
            Self::check_error_envelope(&env)?;
        }

        Ok(serde_json::from_slice(bytes.as_ref())?)
    }

    /// GET and deserialize into a `PaginatedResponseDTO<T>`, parsing the response bytes once.
    pub async fn get<T: DeserializeOwned>(
        &self,
        path: &str,
        params: &[(&str, &str)],
    ) -> Result<PaginatedResponseDTO<T>> {
        let bytes = self.get_bytes(path, params).await?;
        let bytes = bytes.as_ref();

        if let Ok(env) = serde_json::from_slice::<ErrorEnvelope>(bytes) {
            Self::check_error_envelope(&env)?;
        }

        serde_json::from_slice(bytes).map_err(|e| FinanceError::ResponseStructureError {
            field: "response".to_string(),
            context: format!("Failed to deserialize Polygon response: {e}"),
        })
    }

    /// GET and deserialize directly into `T`, wrapping parse failures as
    /// `ResponseStructureError { field, context: "Failed to parse {desc}: {e}" }`.
    pub async fn get_as<T: DeserializeOwned>(
        &self,
        path: &str,
        params: &[(&str, &str)],
        field: &str,
        desc: &str,
    ) -> Result<T> {
        let bytes = self.get_bytes(path, params).await?;
        let bytes = bytes.as_ref();

        if let Ok(env) = serde_json::from_slice::<ErrorEnvelope>(bytes) {
            Self::check_error_envelope(&env)?;
        }

        serde_json::from_slice(bytes).map_err(|e| FinanceError::ResponseStructureError {
            field: field.to_string(),
            context: format!("Failed to parse {desc}: {e}"),
        })
    }
}

/// Cheap-to-parse subset of a Polygon response used to detect the
/// `status: "ERROR" | "NOT_FOUND"` envelope without touching the full body.
#[derive(Deserialize)]
struct ErrorEnvelope {
    status: Option<String>,
    /// Typed as `Value`, not `String`: Polygon has been seen returning a
    /// structured `error`, and a strict type would fail the envelope parse and
    /// skip the status check entirely, turning an error body into a success.
    error: Option<serde_json::Value>,
    message: Option<serde_json::Value>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn error_envelope_maps_bodies_to_errors() {
        /// What `check_error_envelope` is expected to produce for a body.
        enum Want {
            Ok,
            NotFound(&'static str),
            External,
        }

        let cases = [
            (
                r#"{"status":"ERROR","error":"bad request"}"#,
                Want::External,
            ),
            (
                r#"{"status":"ERROR","message":"bad request msg"}"#,
                Want::External,
            ),
            (r#"{"status":"ERROR"}"#, Want::External),
            (
                r#"{"status":"NOT_FOUND","error":"no ticker found"}"#,
                Want::NotFound("no ticker found"),
            ),
            (r#"{"status":"NOT_FOUND"}"#, Want::NotFound("Unknown error")),
            (r#"{"status":"ERROR","error":{"code":1}}"#, Want::External),
            (
                r#"{"status":"NOT_FOUND","error":{"code":2}}"#,
                Want::NotFound("Unknown error"),
            ),
            (r#"{"status":"OK"}"#, Want::Ok),
            (r#"[{"ticker":"AAPL"}]"#, Want::Ok),
            (r#"{}"#, Want::Ok),
        ];

        for (body, want) in cases {
            let checked = match serde_json::from_slice::<ErrorEnvelope>(body.as_bytes()) {
                Ok(env) => PolygonClient::check_error_envelope(&env),
                Err(_) => Ok(()),
            };
            match (want, checked) {
                (Want::Ok, Ok(())) => {}
                (Want::NotFound(ctx), Err(FinanceError::SymbolNotFound { symbol, context })) => {
                    assert_eq!(symbol, None, "body {body}");
                    assert_eq!(context, ctx, "body {body}");
                }
                (Want::External, Err(FinanceError::ExternalApiError { api, status })) => {
                    assert_eq!(api, "Polygon", "body {body}");
                    assert_eq!(status, 400, "body {body}");
                }
                (_, got) => panic!("body {body}: unexpected {got:?}"),
            }
        }
    }
}
