//! Alpha Vantage API client with rate limiting.

use std::sync::Arc;
use std::time::Duration;

use reqwest::{Client, StatusCode};
use serde_json::Value;
use tracing::debug;

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
            base_url: self.base_url.unwrap_or_else(|| AV_BASE.to_string()),
        })
    }
}

/// Alpha Vantage API client. Constructed per-call via the module singleton.
pub(crate) struct AlphaVantageClient {
    api_key: String,
    http: Client,
    limiter: Arc<RateLimiter>,
    base_url: String,
}

impl AlphaVantageClient {
    fn check_error_payload(json: &Value) -> Result<()> {
        if let Some(error_msg) = json.get("Error Message").and_then(|v| v.as_str()) {
            return Err(FinanceError::InvalidParameter {
                param: "function".to_string(),
                reason: error_msg.to_string(),
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
            let normalized = info.to_ascii_lowercase();
            if normalized.contains("api key") || normalized.contains("apikey") {
                return Err(FinanceError::AuthenticationFailed {
                    context: info.to_string(),
                });
            }
            return Err(FinanceError::ApiError(format!("AlphaVantage: {info}")));
        }
        Ok(())
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
        let resp = self.http.get(&self.base_url).query(&query).send().await?;
        let status = resp.status();
        let bytes = resp.bytes().await?;
        let payload = serde_json::from_slice::<Value>(&bytes).ok();
        if let Some(payload) = &payload {
            Self::check_error_payload(payload)?;
        }

        match status {
            StatusCode::OK => {}
            StatusCode::UNAUTHORIZED | StatusCode::FORBIDDEN => {
                return Err(FinanceError::AuthenticationFailed {
                    context: "Alpha Vantage API key invalid or missing. Call alphavantage::init(key) first.".to_string(),
                });
            }
            StatusCode::TOO_MANY_REQUESTS => {
                return Err(FinanceError::RateLimited {
                    retry_after: Some(60),
                });
            }
            s => {
                return Err(FinanceError::ExternalApiError {
                    api: "AlphaVantage".to_string(),
                    status: s.as_u16(),
                });
            }
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
        let resp = self.http.get(&self.base_url).query(&query).send().await?;
        let status = resp.status();
        let body = resp.text().await?;

        match status {
            StatusCode::OK => {}
            StatusCode::UNAUTHORIZED | StatusCode::FORBIDDEN => {
                return Err(FinanceError::AuthenticationFailed {
                    context: "Alpha Vantage API key invalid or missing.".to_string(),
                });
            }
            StatusCode::TOO_MANY_REQUESTS => {
                return Err(FinanceError::RateLimited {
                    retry_after: Some(60),
                });
            }
            s => {
                return Err(FinanceError::ExternalApiError {
                    api: "AlphaVantage".to_string(),
                    status: s.as_u16(),
                });
            }
        }

        if let Ok(payload) = serde_json::from_str::<Value>(&body) {
            Self::check_error_payload(&payload)?;
        }
        Ok(body)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_client(base_url: &str) -> AlphaVantageClient {
        AlphaVantageClientBuilder::new("test-key")
            .base_url(base_url)
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

        let err = test_client(&server.url())
            .get_csv("DIVIDENDS", &[])
            .await
            .unwrap_err();
        assert!(matches!(err, FinanceError::AuthenticationFailed { .. }));
    }
}
