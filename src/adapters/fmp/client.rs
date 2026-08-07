//! Financial Modeling Prep API client with rate limiting.

use std::sync::Arc;
use std::time::Duration;

use reqwest::{Client, StatusCode};
use serde::Deserialize;
use serde::de::DeserializeOwned;
use serde_json::Value;
use tracing::debug;

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
        let http = Client::builder()
            .timeout(self.timeout)
            .user_agent(format!(
                "finance-query/{} (https://github.com/Verdenroz/finance-query)",
                env!("CARGO_PKG_VERSION")
            ))
            .build()?;

        Ok(FmpClient {
            api_key: self.api_key,
            http,
            limiter,
            timeout: self.timeout,
            base_url: self.base_url.unwrap_or_else(|| FMP_BASE.to_string()),
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
    fn check_status(status: StatusCode) -> Result<()> {
        match status {
            StatusCode::OK => Ok(()),
            StatusCode::UNAUTHORIZED | StatusCode::FORBIDDEN => {
                Err(FinanceError::AuthenticationFailed {
                    context: "FMP API key invalid or missing. Call fmp::init(key) first."
                        .to_string(),
                })
            }
            StatusCode::NOT_FOUND => Err(FinanceError::SymbolNotFound {
                symbol: None,
                context: "Resource not found on FMP".to_string(),
            }),
            StatusCode::TOO_MANY_REQUESTS => Err(FinanceError::RateLimited {
                retry_after: Some(60),
            }),
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

    fn check_error_envelope(env: &ErrorEnvelope) -> Result<()> {
        if let Some(msg) = &env.error_message {
            let normalized = msg.to_ascii_lowercase();
            if normalized.contains("api key") || normalized.contains("apikey") {
                return Err(FinanceError::AuthenticationFailed {
                    context: msg.clone(),
                });
            }
            return Err(FinanceError::InvalidParameter {
                param: "request".to_string(),
                reason: msg.clone(),
            });
        }
        Ok(())
    }

    /// Execute a GET request to an FMP REST path and return the raw response bytes.
    async fn get_bytes(&self, path: &str, params: &[(&str, &str)]) -> Result<Vec<u8>> {
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
        let status = resp.status();
        let bytes = resp
            .bytes()
            .await
            .map_err(|error| self.map_transport_error(&error))?;
        if let Ok(env) = serde_json::from_slice::<ErrorEnvelope>(&bytes) {
            Self::check_error_envelope(&env)?;
        }
        Self::check_status(status)?;
        Ok(bytes.to_vec())
    }

    fn map_transport_error(&self, error: &reqwest::Error) -> FinanceError {
        if error.is_timeout() {
            return FinanceError::Timeout {
                timeout_ms: self.timeout_ms(),
            };
        }
        FinanceError::ApiError("FMP HTTP request failed".to_string())
    }

    fn timeout_ms(&self) -> u64 {
        self.timeout.as_millis() as u64
    }

    /// Execute a GET request to an FMP REST path and return raw JSON.
    #[allow(dead_code)]
    pub async fn get_raw(&self, path: &str, params: &[(&str, &str)]) -> Result<Value> {
        let bytes = self.get_bytes(path, params).await?;
        if let Ok(env) = serde_json::from_slice::<ErrorEnvelope>(&bytes) {
            Self::check_error_envelope(&env)?;
        }

        Ok(serde_json::from_slice(&bytes)?)
    }

    /// GET and deserialize into `T` directly, parsing the response bytes once.
    pub async fn get<T: DeserializeOwned>(&self, path: &str, params: &[(&str, &str)]) -> Result<T> {
        let bytes = self.get_bytes(path, params).await?;
        if let Ok(env) = serde_json::from_slice::<ErrorEnvelope>(&bytes) {
            Self::check_error_envelope(&env)?;
        }

        serde_json::from_slice::<T>(&bytes).map_err(|e| FinanceError::ResponseStructureError {
            field: "response".to_string(),
            context: format!("Failed to deserialize FMP response: {e}"),
        })
    }

    pub async fn get_csv_value(&self, path: &str, params: &[(&str, &str)]) -> Result<Value> {
        let bytes = self.get_bytes(path, params).await?;
        let mut reader = csv::Reader::from_reader(bytes.as_slice());
        let headers = reader
            .headers()
            .map_err(|error| {
                FinanceError::UnexpectedResponse(format!("invalid FMP CSV headers: {error}"))
            })?
            .clone();
        let mut rows = Vec::new();
        for record in reader.records() {
            let record = record.map_err(|error| {
                FinanceError::UnexpectedResponse(format!("invalid FMP CSV row: {error}"))
            })?;
            let row = headers
                .iter()
                .zip(record.iter())
                .map(|(name, value)| (name.to_string(), csv_scalar(value)))
                .collect();
            rows.push(Value::Object(row));
        }
        Ok(Value::Array(rows))
    }
}

fn csv_scalar(value: &str) -> Value {
    if value.is_empty() {
        Value::Null
    } else if let Ok(value) = value.parse::<i64>() {
        Value::from(value)
    } else if let Ok(value) = value.parse::<u64>() {
        Value::from(value)
    } else if let Ok(value) = value.parse::<f64>() {
        Value::from(value)
    } else if let Ok(value) = value.parse::<bool>() {
        Value::from(value)
    } else {
        Value::from(value)
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
                Ok(env) => FmpClient::check_error_envelope(&env),
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
            FmpClient::check_error_envelope(&auth),
            Err(FinanceError::AuthenticationFailed { .. })
        ));
    }

    #[tokio::test]
    async fn authentication_body_is_preserved_on_http_error() {
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
        let client = FmpClientBuilder::new("test-key")
            .base_url(server.url())
            .build_with_limiter(Arc::new(RateLimiter::new(100.0)))
            .unwrap();

        let err = client
            .get_raw("/stable/quote", &[("symbol", "AAPL")])
            .await
            .unwrap_err();
        assert!(matches!(err, FinanceError::AuthenticationFailed { .. }));
    }
}
