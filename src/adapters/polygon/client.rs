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
        let scoped = crate::adapters::keys::scoped_key("polygon");
        let base_url = self
            .base_url
            .or_else(|| scoped.as_ref().and_then(|s| s.base_url.clone()))
            .unwrap_or_else(|| PG_BASE.to_string());
        let build_page = || {
            Client::builder()
                .timeout(timeout)
                .connect_timeout(Duration::from_secs(10))
                .read_timeout(Duration::from_secs(30))
                .user_agent(concat!("finance-query/", env!("CARGO_PKG_VERSION")))
                .redirect(reqwest::redirect::Policy::none())
                .build()
        };
        let page_http = match &scoped {
            Some(key) => key.http_client("polygon_page", build_page)?,
            None => build_page()?,
        };
        let build_http = || {
            Client::builder()
                .timeout(timeout)
                .user_agent(format!(
                    "finance-query/{} (https://github.com/Verdenroz/finance-query)",
                    env!("CARGO_PKG_VERSION")
                ))
                .build()
        };
        let http = match &scoped {
            Some(key) => key.http_client("polygon", build_http)?,
            None => build_http()?,
        };

        Ok(PolygonClient {
            page_http,
            api_key: self.api_key,
            http,
            limiter,
            base_url,
            timeout,
        })
    }
}

/// Massive API client. Constructed per-call via the module singleton.
pub(crate) struct PolygonClient {
    page_http: Client,
    api_key: String,
    http: Client,
    limiter: Arc<RateLimiter>,
    base_url: String,
    timeout: Duration,
}

impl PolygonClient {
    pub(super) fn page_url(
        &self,
        target: &str,
        path: &str,
        params: &[(&str, &str)],
        rule: PathRule,
    ) -> Result<reqwest::Url> {
        let invalid = || FinanceError::InvalidParameter {
            param: "cursor".into(),
            reason: "invalid continuation origin, path or request options".into(),
        };
        let base = reqwest::Url::parse(&self.base_url).map_err(|_| invalid())?;
        let mut url = base.join(target).map_err(|_| invalid())?;
        let expected = base.join(path).map_err(|_| invalid())?;
        if url.origin() != base.origin()
            || !rule.allows(url.path(), expected.path())
            || !url.username().is_empty()
            || url.password().is_some()
            || url.fragment().is_some()
        {
            return Err(invalid());
        }
        let mut query = Vec::new();
        for (key, value) in url.query_pairs() {
            if matches!(
                key.to_ascii_lowercase().as_str(),
                "apikey" | "api_key" | "token" | "authorization"
            ) {
                continue;
            }
            if let Some((_, expected)) = params.iter().find(|(name, _)| *name == key) {
                if value != *expected {
                    return Err(invalid());
                }
            } else if key == "cursor" {
                query.push((key.into_owned(), value.into_owned()));
            } else {
                return Err(invalid());
            }
        }
        url.set_query(None);
        url.query_pairs_mut()
            .extend_pairs(query)
            .extend_pairs(params.iter().copied());
        Ok(url)
    }

    pub(super) async fn page<T: DeserializeOwned>(
        &self,
        path: &str,
        params: &[(&str, &str)],
        rule: PathRule,
        cursor: Option<&crate::PageCursor>,
        operation: &str,
        request: &str,
    ) -> Result<(T, String)> {
        if let Some(cursor) = cursor {
            cursor.validate(operation, request)?;
            if cursor.provider() != crate::Provider::Polygon {
                return Err(FinanceError::InvalidParameter {
                    param: "cursor".into(),
                    reason: "wrong provider".into(),
                });
            }
        }
        let url = self.page_url(
            cursor.map_or(path, |c| c.target.as_str()),
            path,
            params,
            rule,
        )?;
        self.limiter.acquire().await;
        let response = self
            .page_http
            .get(url.clone())
            .query(&[("apiKey", self.api_key.as_str())])
            .send()
            .await
            .map_err(|e| self.map_transport_error(&e))?;
        if response.status() == StatusCode::TOO_MANY_REQUESTS {
            return Err(FinanceError::RateLimited {
                retry_after: crate::adapters::common::keyed::retry_after(response.headers()),
            });
        }
        Self::check_status(response.status())?;
        let bytes = crate::adapters::common::keyed::bounded_body(
            response,
            32 * 1024 * 1024,
            "Polygon",
            self.timeout,
        )
        .await?;
        let envelope: ErrorEnvelope =
            serde_json::from_slice(&bytes).map_err(|_| FinanceError::ResponseStructureError {
                field: "response".into(),
                context: "invalid Polygon page".into(),
            })?;
        Self::check_error_envelope(&envelope, &self.api_key)?;
        if !matches!(envelope.status.as_deref(), Some("OK" | "DELAYED")) {
            return Err(FinanceError::ResponseStructureError {
                field: "status".into(),
                context: "missing successful Polygon page status".into(),
            });
        }
        let body =
            serde_json::from_slice(&bytes).map_err(|_| FinanceError::ResponseStructureError {
                field: "response".into(),
                context: "invalid Polygon page fields".into(),
            })?;
        Ok((body, url.to_string()))
    }

    #[allow(clippy::too_many_arguments)]
    pub(super) fn continuation(
        &self,
        next: Option<String>,
        path: &str,
        params: &[(&str, &str)],
        rule: PathRule,
        operation: &str,
        request: &str,
        current: &str,
    ) -> Result<Option<crate::PageCursor>> {
        next.map(|target| {
            let target = self.page_url(&target, path, params, rule)?.to_string();
            if target == current || !rule.advances(&target, current) {
                return Err(FinanceError::ResponseStructureError {
                    field: "next_url".into(),
                    context: "provider repeated its continuation".into(),
                });
            }
            Ok(crate::PageCursor {
                version: 1,
                provider: crate::Provider::Polygon,
                operation: operation.into(),
                request: request.into(),
                target,
            })
        })
        .transpose()
    }
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
        if status == StatusCode::TOO_MANY_REQUESTS {
            return Err(FinanceError::RateLimited {
                retry_after: crate::adapters::common::keyed::retry_after(resp.headers())
                    .or(Some(60)),
            });
        }
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

const DAY_MS: i64 = 86_400_000;

/// How far a continuation's path may differ from the request path.
#[derive(Clone, Copy)]
pub(super) enum PathRule {
    Exact,
    /// Aggregates advance their trailing `from`/`to` segments to bar timestamps.
    AggregateWindow,
}

impl PathRule {
    fn allows(self, actual: &str, expected: &str) -> bool {
        match self {
            Self::Exact => actual == expected,
            Self::AggregateWindow => {
                let (Some((prefix, from, to)), Some((want_prefix, want_from, want_to))) =
                    (aggregate_window(actual), aggregate_window(expected))
                else {
                    return false;
                };
                // Date bounds are UTC midnights but bars follow exchange time, which
                // can run into the next UTC day.
                let within = |bound| (want_from..want_to + 2 * DAY_MS).contains(&bound);
                prefix == want_prefix && within(from) && within(to)
            }
        }
    }

    fn advances(self, next: &str, current: &str) -> bool {
        let window = |url: &str| {
            reqwest::Url::parse(url)
                .ok()
                .and_then(|url| aggregate_window(url.path()).map(|(_, from, to)| (from, to)))
        };
        match self {
            Self::Exact => true,
            Self::AggregateWindow => matches!(
                (window(next), window(current)),
                (Some((from, to)), Some((current_from, current_to)))
                    if from >= current_from && to <= current_to
            ),
        }
    }
}

/// Splits `.../{from}/{to}` into the fixed prefix and the window in Unix milliseconds.
fn aggregate_window(path: &str) -> Option<(&str, i64, i64)> {
    let (rest, to) = path.rsplit_once('/')?;
    let (prefix, from) = rest.rsplit_once('/')?;
    Some((prefix, window_bound(from)?, window_bound(to)?))
}

fn window_bound(segment: &str) -> Option<i64> {
    if !segment.is_empty() && segment.bytes().all(|b| b.is_ascii_digit()) {
        return segment.parse().ok();
    }
    let date = chrono::NaiveDate::parse_from_str(segment, "%Y-%m-%d").ok()?;
    Some(date.and_hms_opt(0, 0, 0)?.and_utc().timestamp_millis())
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

    #[test]
    fn aggregate_continuations_may_only_narrow_the_requested_window() {
        let path = "/v2/aggs/ticker/AAPL/range/1/day/2020-01-02/2020-01-10";
        let rule = PathRule::AggregateWindow;
        let url = |from: &str, to: &str| {
            format!("https://api.massive.com/v2/aggs/ticker/AAPL/range/1/day/{from}/{to}")
        };
        assert!(rule.allows(
            "/v2/aggs/ticker/AAPL/range/1/day/1578114000000/2020-01-10",
            path
        ));
        assert!(rule.allows(
            "/v2/aggs/ticker/AAPL/range/1/day/2020-01-02/1578459600000",
            path
        ));
        assert!(!rule.allows(
            "/v2/aggs/ticker/AAPL/range/1/day/1577836800000/2020-01-10",
            path
        ));
        assert!(!rule.allows(
            "/v2/aggs/ticker/AAPL/range/1/day/2020-01-02/2020-01-31",
            path
        ));
        assert!(!rule.allows(
            "/v2/aggs/ticker/MSFT/range/1/day/1578114000000/2020-01-10",
            path
        ));
        assert!(!PathRule::Exact.allows(
            "/v2/aggs/ticker/AAPL/range/1/day/1578114000000/2020-01-10",
            path
        ));

        let current = url("1578114000000", "2020-01-10");
        assert!(rule.advances(&url("1578286800000", "2020-01-10"), &current));
        assert!(!rule.advances(&url("2020-01-02", "2020-01-10"), &current));
        let descending = url("2020-01-02", "1578459600000");
        assert!(rule.advances(&url("2020-01-02", "1578373200000"), &descending));
        assert!(!rule.advances(&url("2020-01-02", "2020-01-10"), &descending));
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
