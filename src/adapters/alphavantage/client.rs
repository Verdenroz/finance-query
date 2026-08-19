//! Alpha Vantage API client with rate limiting.

use std::sync::Arc;
use std::time::Duration;

use reqwest::{Client, StatusCode};
use serde_json::Value;
use tracing::debug;

use crate::adapters::common::keyed::{is_auth_error, redact_key, transport_error};
use crate::error::{FinanceError, Result};
use crate::rate_limiter::RateLimiter;

const AV_BASE: &str = "https://www.alphavantage.co/query";

pub(crate) struct AlphaVantageClientBuilder {
    api_key: String,
    timeout: Duration,
    base_url: Option<String>,
}

impl AlphaVantageClientBuilder {
    pub fn new(api_key: impl Into<String>) -> Self {
        Self {
            api_key: api_key.into(),
            timeout: Duration::from_secs(30),
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

    pub(super) fn build_with_limiter(
        self,
        limiter: Arc<RateLimiter>,
    ) -> Result<AlphaVantageClient> {
        let http = Client::builder()
            .timeout(self.timeout)
            .user_agent(format!(
                "finance-query/{} (https://github.com/Verdenroz/finance-query)",
                env!("CARGO_PKG_VERSION")
            ))
            .build()?;

        Ok(AlphaVantageClient {
            api_key: self.api_key,
            http,
            limiter,
            timeout: self.timeout,
            base_url: self.base_url.unwrap_or_else(|| AV_BASE.to_string()),
        })
    }
}

/// Alpha Vantage API client. Constructed per-call via the module singleton.
pub(crate) struct AlphaVantageClient {
    api_key: String,
    http: Client,
    limiter: Arc<RateLimiter>,
    timeout: Duration,
    base_url: String,
}

impl AlphaVantageClient {
    fn check_error_payload(json: &Value, api_key: &str) -> Result<()> {
        if let Some(error_msg) = json.get("Error Message").and_then(|v| v.as_str()) {
            return Err(FinanceError::InvalidParameter {
                param: "function".to_string(),
                reason: redact_key(error_msg, api_key),
            });
        }
        if let Some(note) = json.get("Note").and_then(|v| v.as_str())
            && note.contains("call frequency")
        {
            return Err(FinanceError::RateLimited {
                retry_after: Some(60),
            });
        }
        if let Some(info) = json.get("Information").and_then(|v| v.as_str()) {
            if info.contains("rate limit") || info.contains("API call frequency") {
                return Err(FinanceError::RateLimited {
                    retry_after: Some(60),
                });
            }
            let info = redact_key(info, api_key);
            if is_auth_error(&info.to_ascii_lowercase()) {
                return Err(FinanceError::AuthenticationFailed { context: info });
            }
            return Err(FinanceError::ApiError(format!("AlphaVantage: {info}")));
        }
        Ok(())
    }

    fn map_transport_error(&self, error: &reqwest::Error) -> FinanceError {
        transport_error("AlphaVantage", self.timeout, error)
    }

    fn check_status(status: StatusCode) -> Result<()> {
        match status {
            StatusCode::OK => Ok(()),
            StatusCode::UNAUTHORIZED | StatusCode::FORBIDDEN => {
                Err(FinanceError::AuthenticationFailed {
                    context: "Alpha Vantage API key invalid or missing. Call alphavantage::init(key) first.".to_string(),
                })
            }
            StatusCode::TOO_MANY_REQUESTS => Err(FinanceError::RateLimited {
                retry_after: Some(60),
            }),
            s => Err(FinanceError::ExternalApiError {
                api: "AlphaVantage".to_string(),
                status: s.as_u16(),
            }),
        }
    }

    /// Execute a GET request to the Alpha Vantage API.
    ///
    /// All AV endpoints use the same base URL with `function=` and `apikey=` query params.
    /// Additional params are passed as `&[(&str, &str)]`.
    pub async fn get(&self, function: &str, params: &[(&str, &str)]) -> Result<Value> {
        self.limiter.acquire().await;

        let mut query: Vec<(&str, &str)> = vec![("function", function), ("apikey", &self.api_key)];
        query.extend_from_slice(params);

        debug!("AlphaVantage request: function={function}");
        let resp = self
            .http
            .get(&self.base_url)
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

        let payload = serde_json::from_slice::<Value>(&bytes).ok();
        if let Some(payload) = &payload {
            Self::check_error_payload(payload, &self.api_key)?;
        }

        let json = payload.ok_or_else(|| FinanceError::ResponseStructureError {
            field: "response".to_string(),
            context: "Alpha Vantage returned invalid JSON".to_string(),
        })?;
        Ok(json)
    }

    /// Execute a GET request that returns CSV data (for calendar endpoints).
    pub async fn get_csv(&self, function: &str, params: &[(&str, &str)]) -> Result<String> {
        self.limiter.acquire().await;

        let mut query: Vec<(&str, &str)> = vec![("function", function), ("apikey", &self.api_key)];
        query.extend_from_slice(params);

        debug!("AlphaVantage CSV request: function={function}");
        let resp = self
            .http
            .get(&self.base_url)
            .query(&query)
            .send()
            .await
            .map_err(|error| self.map_transport_error(&error))?;
        let status = resp.status();
        let body = resp
            .text()
            .await
            .map_err(|error| self.map_transport_error(&error))?;

        Self::check_status(status)?;

        if let Ok(payload) = serde_json::from_str::<Value>(&body) {
            Self::check_error_payload(&payload, &self.api_key)?;
        }
        Ok(body)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn client(api_key: &str, base_url: &str) -> AlphaVantageClient {
        AlphaVantageClientBuilder::new(api_key)
            .base_url(base_url)
            .timeout(Duration::from_secs(5))
            .build_with_limiter(Arc::new(RateLimiter::new(100.0)))
            .unwrap()
    }

    #[tokio::test]
    async fn csv_request_recognizes_json_authentication_error() {
        let mut server = mockito::Server::new_async().await;
        let _mock = server
            .mock("GET", "/")
            .match_query(mockito::Matcher::AllOf(vec![
                mockito::Matcher::UrlEncoded("function".into(), "DIVIDENDS".into()),
                mockito::Matcher::UrlEncoded("apikey".into(), "test-key".into()),
            ]))
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(r#"{"Information":"Invalid API key. Please visit the support page."}"#)
            .create_async()
            .await;

        let err = client("test-key", &server.url())
            .get_csv("DIVIDENDS", &[])
            .await
            .unwrap_err();
        assert!(matches!(err, FinanceError::AuthenticationFailed { .. }));
    }

    #[tokio::test]
    async fn rate_limited_status_outranks_the_json_body() {
        let mut server = mockito::Server::new_async().await;
        let _mock = server
            .mock("GET", "/")
            .match_query(mockito::Matcher::Any)
            .with_status(429)
            .with_header("content-type", "application/json")
            .with_body(r#"{"Error Message":"Invalid API call."}"#)
            .create_async()
            .await;

        let err = client("test-key", &server.url())
            .get("OVERVIEW", &[])
            .await
            .unwrap_err();
        assert!(matches!(err, FinanceError::RateLimited { .. }));
        assert!(err.is_retriable());
    }

    #[tokio::test]
    async fn errors_never_render_the_api_key() {
        const KEY: &str = "SUPERSECRETKEY123";

        let mut server = mockito::Server::new_async().await;
        let _mock = server
            .mock("GET", "/")
            .match_query(mockito::Matcher::Any)
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(format!(
                r#"{{"Information":"Invalid API key {KEY}. Please visit the support page."}}"#
            ))
            .create_async()
            .await;

        let echoed = client(KEY, &server.url())
            .get("OVERVIEW", &[])
            .await
            .unwrap_err();
        let unreachable = client(KEY, "http://127.0.0.1:1")
            .get("OVERVIEW", &[])
            .await
            .unwrap_err();

        for err in [echoed, unreachable] {
            assert!(!format!("{err}").contains(KEY), "{err}");
            assert!(!format!("{err:?}").contains(KEY), "{err:?}");
        }
    }
}
