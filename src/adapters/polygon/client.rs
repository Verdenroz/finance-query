//! Polygon/Massive API client with rate limiting and cursor-based pagination.

use std::sync::Arc;
use std::time::Duration;

use reqwest::{Client, StatusCode};
use serde::Deserialize;
use serde::de::DeserializeOwned;
#[cfg(test)]
use serde_json::Value;
use tracing::debug;

use crate::adapters::common::keyed::{is_auth_error, redact_key, transport_error};
use crate::error::{FinanceError, Result};
use crate::rate_limiter::RateLimiter;

use super::models::PaginatedResponseDTO;

const PG_BASE: &str = "https://api.massive.com";
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
        let timeout = self.timeout;
        let http = Client::builder()
            .timeout(timeout)
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
            timeout,
        })
    }
}

/// Massive API client. Constructed per-call via the module singleton.
pub(crate) struct PolygonClient {
    api_key: String,
    http: Client,
    limiter: Arc<RateLimiter>,
    base_url: String,
    timeout: Duration,
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

    fn check_error_envelope(env: &ErrorEnvelope, api_key: &str) -> Result<()> {
        let Some(status) = env.status.as_deref() else {
            return Ok(());
        };
        if status != "ERROR" && status != "NOT_FOUND" && status != "NOT_AUTHORIZED" {
            return Ok(());
        }
        let msg = redact_key(
            env.error
                .as_ref()
                .and_then(|v| v.as_str())
                .or_else(|| env.message.as_ref().and_then(|v| v.as_str()))
                .unwrap_or("Unknown error"),
            api_key,
        );
        if status == "NOT_FOUND" {
            return Err(FinanceError::SymbolNotFound {
                symbol: None,
                context: msg,
            });
        }
        let normalized = msg.to_ascii_lowercase();
        if status == "NOT_AUTHORIZED"
            || is_auth_error(&normalized)
            || normalized.contains("not authorized")
            || normalized.contains("not entitled")
            || normalized.contains("upgrade your plan")
        {
            return Err(FinanceError::AuthenticationFailed { context: msg });
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
        let resp = self
            .http
            .get(&url)
            .query(&query)
            .send()
            .await
            .map_err(|error| self.map_transport_error(&error))?;
        let status = resp.status();
        let bytes = resp
            .bytes()
            .await
            .map_err(|error| self.map_transport_error(&error))?;
        Self::check_status(status)?;
        if let Ok(env) = serde_json::from_slice::<ErrorEnvelope>(&bytes) {
            Self::check_error_envelope(&env, &self.api_key)?;
        }
        Ok(bytes)
    }

    fn map_transport_error(&self, error: &reqwest::Error) -> FinanceError {
        transport_error("Polygon", self.timeout, error)
    }

    /// Execute a GET request to a Polygon REST path and return raw JSON.
    #[cfg(test)]
    pub async fn get_raw(&self, path: &str, params: &[(&str, &str)]) -> Result<Value> {
        let bytes = self.get_bytes(path, params).await?;
        Ok(serde_json::from_slice(bytes.as_ref())?)
    }

    /// GET and deserialize into a `PaginatedResponseDTO<T>`, parsing the response bytes once.
    pub async fn get<T: DeserializeOwned>(
        &self,
        path: &str,
        params: &[(&str, &str)],
    ) -> Result<PaginatedResponseDTO<T>> {
        let bytes = self.get_bytes(path, params).await?;

        serde_json::from_slice(bytes.as_ref()).map_err(|e| FinanceError::ResponseStructureError {
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

        serde_json::from_slice(bytes.as_ref()).map_err(|e| FinanceError::ResponseStructureError {
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
    use crate::rate_limiter::RateLimiter;

    #[test]
    fn error_envelope_maps_bodies_to_errors() {
        /// What `check_error_envelope` is expected to produce for a body.
        enum Want {
            Ok,
            NotFound(&'static str),
            External,
            Auth,
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
            (
                r#"{"status":"ERROR","error":"API Key was not provided"}"#,
                Want::Auth,
            ),
            (
                r#"{"status":"ERROR","error":"You are not entitled to this data. Please upgrade your plan"}"#,
                Want::Auth,
            ),
            (
                r#"{"status":"NOT_AUTHORIZED","message":"plan restriction"}"#,
                Want::Auth,
            ),
            (r#"{"status":"OK"}"#, Want::Ok),
            (r#"[{"ticker":"AAPL"}]"#, Want::Ok),
            (r#"{}"#, Want::Ok),
        ];

        for (body, want) in cases {
            let checked = match serde_json::from_slice::<ErrorEnvelope>(body.as_bytes()) {
                Ok(env) => PolygonClient::check_error_envelope(&env, "test-key"),
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
                (Want::Auth, Err(FinanceError::AuthenticationFailed { .. })) => {}
                (_, got) => panic!("body {body}: unexpected {got:?}"),
            }
        }
    }

    fn client(api_key: &str, base_url: &str) -> PolygonClient {
        PolygonClientBuilder::new(api_key)
            .base_url(base_url)
            .timeout(Duration::from_secs(5))
            .build_with_limiter(Arc::new(RateLimiter::new(100.0)))
            .unwrap()
    }

    #[tokio::test]
    async fn http_403_maps_to_authentication_error() {
        let mut server = mockito::Server::new_async().await;
        let _mock = server
            .mock("GET", "/v2/aggs/ticker/AAPL/prev")
            .match_query(mockito::Matcher::UrlEncoded(
                "apiKey".into(),
                "test-key".into(),
            ))
            .with_status(403)
            .with_header("content-type", "application/json")
            .with_body(r#"{"status":"ERROR","error":"API Key is invalid"}"#)
            .create_async()
            .await;

        let err = client("test-key", &server.url())
            .get_raw("/v2/aggs/ticker/AAPL/prev", &[])
            .await
            .unwrap_err();
        assert!(matches!(err, FinanceError::AuthenticationFailed { .. }));
    }

    #[tokio::test]
    async fn errors_never_render_the_api_key() {
        const KEY: &str = "SUPERSECRETKEY123";

        let mut server = mockito::Server::new_async().await;
        let _mock = server
            .mock("GET", "/v3/reference/tickers")
            .match_query(mockito::Matcher::Any)
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(format!(
                r#"{{"status":"NOT_AUTHORIZED","message":"API Key {KEY} is not entitled"}}"#
            ))
            .create_async()
            .await;

        let echoed = client(KEY, &server.url())
            .get_raw("/v3/reference/tickers", &[])
            .await
            .unwrap_err();
        let unreachable = client(KEY, "http://127.0.0.1:1")
            .get_raw("/v3/reference/tickers", &[])
            .await
            .unwrap_err();

        for err in [echoed, unreachable] {
            assert!(!format!("{err}").contains(KEY), "{err}");
            assert!(!format!("{err:?}").contains(KEY), "{err:?}");
        }
    }
}
