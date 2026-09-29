//! Financial Modeling Prep API client with rate limiting.

use std::sync::Arc;
use std::time::Duration;

use reqwest::header::HeaderMap;
use reqwest::{Client, StatusCode};
use serde::Deserialize;
use serde::de::DeserializeOwned;
#[cfg(test)]
use serde_json::Value;
use tracing::debug;

use crate::adapters::common::keyed::{
    bounded_body, http_client, is_auth_error, rate_limited, redact_key, transport_error,
};
use crate::error::{FinanceError, Result};
use crate::rate_limiter::RateLimiter;

const FMP_BASE: &str = "https://financialmodelingprep.com";
const DEFAULT_TIMEOUT: Duration = Duration::from_secs(30);

pub(crate) struct FmpClientBuilder {
    api_key: String,
    timeout: Duration,
    base_url: Option<String>,
}

impl FmpClientBuilder {
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

    pub(super) fn build_with_limiter(self, limiter: Arc<RateLimiter>) -> Result<FmpClient> {
        let scoped = crate::adapters::keys::scoped_key("fmp");
        let base_url = self
            .base_url
            .or_else(|| scoped.as_ref().and_then(|s| s.base_url.clone()))
            .unwrap_or_else(|| FMP_BASE.to_string());
        let http = http_client(scoped.as_ref(), "fmp", self.timeout)?;

        Ok(FmpClient {
            api_key: self.api_key,
            http,
            limiter,
            timeout: self.timeout,
            base_url,
        })
    }
}

/// Financial Modeling Prep API client. Constructed per-call via the module singleton.
pub(crate) struct FmpClient {
    api_key: String,
    http: Client,
    limiter: Arc<RateLimiter>,
    timeout: Duration,
    base_url: String,
}

impl FmpClient {
    fn check_status(status: StatusCode, headers: &HeaderMap) -> Result<()> {
        match status {
            StatusCode::OK => Ok(()),
            StatusCode::UNAUTHORIZED => Err(FinanceError::AuthenticationFailed {
                context: "FMP API key invalid or missing. Call fmp::init(key) first.".to_string(),
            }),
            StatusCode::FORBIDDEN => Err(FinanceError::AuthenticationFailed {
                context: "FMP access denied for this endpoint (HTTP 403)".into(),
            }),
            StatusCode::NOT_FOUND => Err(FinanceError::SymbolNotFound {
                symbol: None,
                context: "Resource not found on FMP".to_string(),
            }),
            StatusCode::TOO_MANY_REQUESTS => Err(rate_limited(headers)),
            s if s.is_server_error() => Err(FinanceError::ServerError {
                status: s.as_u16(),
                context: "FMP server error".to_string(),
            }),
            s => Err(FinanceError::ExternalApiError {
                api: "FMP".to_string(),
                status: s.as_u16(),
            }),
        }
    }

    fn check_error_envelope(env: &ErrorEnvelope, api_key: &str) -> Result<()> {
        if let Some(msg) = &env.error_message {
            let msg = redact_key(msg, api_key);
            if is_auth_error(&msg.to_ascii_lowercase()) {
                return Err(FinanceError::AuthenticationFailed { context: msg });
            }
            return Err(FinanceError::InvalidParameter {
                param: "request".to_string(),
                reason: msg,
            });
        }
        Ok(())
    }

    /// Execute a GET request to an FMP REST path and return the raw response bytes.
    ///
    /// `byte_limit` bounds the body for endpoints whose size depends on the account's data.
    pub(super) async fn get_bytes(
        &self,
        path: &str,
        params: &[(&str, &str)],
        byte_limit: Option<usize>,
    ) -> Result<Vec<u8>> {
        self.limiter.acquire().await;

        let url = format!("{}{}", self.base_url, path);
        let mut query: Vec<(&str, &str)> = vec![("apikey", &self.api_key)];
        query.extend_from_slice(params);

        debug!("FMP request: {path}");
        let resp = self
            .http
            .get(&url)
            .query(&query)
            .send()
            .await
            .map_err(|error| self.map_transport_error(&error))?;
        Self::check_status(resp.status(), resp.headers())?;

        let bytes = match byte_limit {
            Some(limit) => bounded_body(resp, limit, "FMP", self.timeout).await?,
            None => resp
                .bytes()
                .await
                .map_err(|error| self.map_transport_error(&error))?
                .to_vec(),
        };
        if let Ok(env) = serde_json::from_slice::<ErrorEnvelope>(&bytes) {
            Self::check_error_envelope(&env, &self.api_key)?;
        }
        Ok(bytes)
    }

    fn map_transport_error(&self, error: &reqwest::Error) -> FinanceError {
        transport_error("FMP", self.timeout, error)
    }

    /// Execute a GET request to an FMP REST path and return raw JSON.
    #[cfg(test)]
    pub(crate) async fn get_raw(&self, path: &str, params: &[(&str, &str)]) -> Result<Value> {
        let bytes = self.get_bytes(path, params, None).await?;
        Ok(serde_json::from_slice(bytes.as_ref())?)
    }

    /// GET and deserialize into `T` directly, parsing the response bytes once.
    pub async fn get<T: DeserializeOwned>(&self, path: &str, params: &[(&str, &str)]) -> Result<T> {
        self.get_limited(path, params, None).await
    }

    /// [`get`](Self::get) with the response body bounded to `byte_limit` bytes.
    pub(crate) async fn get_limited<T: DeserializeOwned>(
        &self,
        path: &str,
        params: &[(&str, &str)],
        byte_limit: Option<usize>,
    ) -> Result<T> {
        let bytes = self.get_bytes(path, params, byte_limit).await?;
        serde_json::from_slice::<T>(bytes.as_ref()).map_err(|e| {
            FinanceError::ResponseStructureError {
                field: "response".to_string(),
                context: format!(
                    "Failed to deserialize FMP response: {}",
                    redact_key(&e.to_string(), &self.api_key)
                ),
            }
        })
    }
}

/// Cheap-to-parse subset of an FMP response used to detect the
/// `{"Error Message": "..."}` envelope without touching the full body.
#[derive(Deserialize)]
struct ErrorEnvelope {
    #[serde(rename = "Error Message")]
    error_message: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn error_envelope_maps_bodies_to_errors() {
        // `{"Error Message": <str>}` at HTTP 200 is FMP's error shape; anything
        // else — including a top-level array — must pass through untouched.
        let cases: [(&str, Option<&str>); 4] = [
            (r#"{"Error Message":""}"#, Some("")),
            (r#"[{"symbol":"AAPL"}]"#, None),
            (r#"{}"#, None),
            (r#"{"Error Message":null}"#, None),
        ];
        for (body, expected) in cases {
            let checked = match serde_json::from_slice::<ErrorEnvelope>(body.as_bytes()) {
                Ok(env) => FmpClient::check_error_envelope(&env, "test-key"),
                Err(_) => Ok(()),
            };
            match (expected, checked) {
                (None, Ok(())) => {}
                (Some(msg), Err(FinanceError::InvalidParameter { param, reason })) => {
                    assert_eq!(param, "request", "body {body}");
                    assert_eq!(reason, msg, "body {body}");
                }
                (e, got) => panic!("body {body}: expected {e:?}, got {got:?}"),
            }
        }

        let auth = serde_json::from_str::<ErrorEnvelope>(r#"{"Error Message":"Invalid API KEY"}"#)
            .unwrap();
        assert!(matches!(
            FmpClient::check_error_envelope(&auth, "test-key"),
            Err(FinanceError::AuthenticationFailed { .. })
        ));
    }

    fn client(api_key: &str, base_url: &str) -> FmpClient {
        FmpClientBuilder::new(api_key)
            .base_url(base_url)
            .timeout(Duration::from_secs(5))
            .build_with_limiter(Arc::new(RateLimiter::new(100.0)))
            .unwrap()
    }

    #[tokio::test]
    async fn http_401_maps_to_authentication_error() {
        let mut server = mockito::Server::new_async().await;
        let _mock = server
            .mock("GET", "/stable/quote")
            .match_query(mockito::Matcher::AllOf(vec![
                mockito::Matcher::UrlEncoded("apikey".into(), "test-key".into()),
                mockito::Matcher::UrlEncoded("symbol".into(), "AAPL".into()),
            ]))
            .with_status(401)
            .with_header("content-type", "application/json")
            .with_body(r#"{"Error Message":"Invalid API KEY"}"#)
            .create_async()
            .await;

        let err = client("test-key", &server.url())
            .get_raw("/stable/quote", &[("symbol", "AAPL")])
            .await
            .unwrap_err();
        assert!(matches!(err, FinanceError::AuthenticationFailed { .. }));
    }

    #[tokio::test]
    async fn errors_never_render_the_api_key() {
        const KEY: &str = "SUPERSECRETKEY123";

        let mut server = mockito::Server::new_async().await;
        let _mock = server
            .mock("GET", "/stable/quote")
            .match_query(mockito::Matcher::Any)
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(format!(
                r#"{{"Error Message":"Invalid API KEY {KEY} supplied"}}"#
            ))
            .create_async()
            .await;

        let echoed = client(KEY, &server.url())
            .get_raw("/stable/quote", &[])
            .await
            .unwrap_err();
        let unreachable = client(KEY, "http://127.0.0.1:1")
            .get_raw("/stable/quote", &[])
            .await
            .unwrap_err();

        for err in [echoed, unreachable] {
            assert!(!format!("{err}").contains(KEY), "{err}");
            assert!(!format!("{err:?}").contains(KEY), "{err:?}");
        }
    }
}
