//! Error hygiene for the adapters that carry an API key.

/// Build the adapter's HTTP client, reused per runtime when a scoped key is set.
///
/// Redirects are refused so a key carried in the query string never follows a
/// redirect to another origin.
#[cfg(any(feature = "polygon", feature = "fmp"))]
pub(crate) fn http_client(
    scoped: Option<&crate::adapters::keys::ScopedKey>,
    provider: &'static str,
    timeout: std::time::Duration,
) -> reqwest::Result<reqwest::Client> {
    let build = || {
        reqwest::Client::builder()
            .timeout(timeout)
            .connect_timeout(std::time::Duration::from_secs(10))
            .user_agent(format!(
                "finance-query/{} (https://github.com/Verdenroz/finance-query)",
                env!("CARGO_PKG_VERSION")
            ))
            .redirect(reqwest::redirect::Policy::none())
            .build()
    };
    match scoped {
        Some(key) => key.http_client(provider, build),
        None => build(),
    }
}

/// HTTP 429, honouring the provider's `Retry-After` and defaulting to a minute.
#[cfg(any(feature = "polygon", feature = "fmp"))]
pub(crate) fn rate_limited(headers: &reqwest::header::HeaderMap) -> crate::FinanceError {
    crate::FinanceError::RateLimited {
        retry_after: retry_after_at(headers, chrono::Utc::now()).or(Some(60)),
    }
}

#[cfg(any(feature = "polygon", feature = "fmp"))]
fn retry_after_at(
    headers: &reqwest::header::HeaderMap,
    now: chrono::DateTime<chrono::Utc>,
) -> Option<u64> {
    let value = headers.get("retry-after")?.to_str().ok()?;
    value.parse().ok().or_else(|| {
        chrono::DateTime::parse_from_rfc2822(value)
            .ok()
            .map(|date| {
                let delay = (date.with_timezone(&chrono::Utc) - now)
                    .to_std()
                    .unwrap_or_default();
                delay.as_secs() + u64::from(delay.subsec_nanos() != 0)
            })
    })
}

#[cfg(any(feature = "polygon", feature = "fmp"))]
pub(crate) async fn bounded_body(
    mut response: reqwest::Response,
    limit: usize,
    api: &str,
    timeout: std::time::Duration,
) -> crate::Result<Vec<u8>> {
    let mut bytes = Vec::new();
    while let Some(chunk) = response
        .chunk()
        .await
        .map_err(|error| transport_error(api, timeout, &error))?
    {
        if chunk.len() > limit.saturating_sub(bytes.len()) {
            return Err(crate::FinanceError::ResponseStructureError {
                field: "response".into(),
                context: "provider response exceeds byte limit".into(),
            });
        }
        bytes.extend_from_slice(&chunk);
    }
    Ok(bytes)
}

/// Map a transport failure without keeping the `reqwest::Error`.
///
/// A `reqwest::Error` renders the full request URL in both its `Display` and
/// its `Debug` impl, so wrapping one puts the query string, and with it the
/// API key, into every log line that formats the error.
#[cfg(any(
    feature = "alphavantage",
    feature = "fmp",
    feature = "fred",
    feature = "polygon"
))]
pub(crate) fn transport_error(
    api: &str,
    timeout: std::time::Duration,
    error: &reqwest::Error,
) -> crate::error::FinanceError {
    use crate::error::FinanceError;

    if error.is_timeout() {
        return FinanceError::Timeout {
            timeout_ms: timeout.as_millis() as u64,
        };
    }
    FinanceError::NetworkError {
        api: api.to_string(),
    }
}

/// Strip the configured API key out of a message the provider wrote.
///
/// Several providers quote the submitted key back in their authentication
/// failures, which would otherwise be forwarded verbatim into an error.
pub(crate) fn redact_key(message: &str, api_key: &str) -> String {
    if api_key.trim().is_empty() {
        return message.to_string();
    }
    message.replace(api_key, "[redacted]")
}

/// Whether a lowercased provider error message is complaining about the API key.
///
/// Callers with extra provider-specific phrasing (e.g. "not entitled") should
/// OR this with their own checks rather than restate the base set.
#[cfg(any(
    feature = "alphavantage",
    feature = "fmp",
    feature = "fred",
    feature = "polygon"
))]
pub(crate) fn is_auth_error(normalized_message: &str) -> bool {
    normalized_message.contains("api key")
        || normalized_message.contains("apikey")
        || normalized_message.contains("api_key")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(any(feature = "polygon", feature = "fmp"))]
    #[test]
    fn rate_limited_reads_retry_after_seconds_and_http_dates() {
        let delay = |headers: &reqwest::header::HeaderMap| match rate_limited(headers) {
            crate::FinanceError::RateLimited { retry_after } => retry_after,
            other => panic!("unexpected {other:?}"),
        };
        let mut headers = reqwest::header::HeaderMap::new();
        assert_eq!(delay(&headers), Some(60));
        headers.insert("retry-after", "12".parse().unwrap());
        assert_eq!(delay(&headers), Some(12));
        headers.insert(
            "retry-after",
            "Wed, 21 Oct 2015 07:28:00 GMT".parse().unwrap(),
        );
        assert_eq!(delay(&headers), Some(0));
        headers.insert(
            "retry-after",
            "Wed, 21 Oct 2015 07:28:01 GMT".parse().unwrap(),
        );
        let now = chrono::DateTime::parse_from_rfc3339("2015-10-21T07:28:00.500Z")
            .unwrap()
            .with_timezone(&chrono::Utc);
        assert_eq!(retry_after_at(&headers, now), Some(1));
    }

    #[test]
    fn echoed_key_is_replaced() {
        assert_eq!(
            redact_key("Invalid key abc123 supplied", "abc123"),
            "Invalid key [redacted] supplied"
        );
    }

    #[test]
    fn every_occurrence_is_replaced() {
        assert_eq!(
            redact_key("abc123/abc123", "abc123"),
            "[redacted]/[redacted]"
        );
    }

    #[test]
    fn message_without_the_key_is_untouched() {
        assert_eq!(redact_key("Invalid request", "abc123"), "Invalid request");
    }

    #[test]
    fn blank_key_matches_nothing() {
        assert_eq!(redact_key("Invalid request", "   "), "Invalid request");
    }

    #[test]
    fn auth_error_matches_known_phrasings() {
        assert!(is_auth_error("invalid api key supplied"));
        assert!(is_auth_error("bad apikey"));
        assert!(is_auth_error("api_key is not registered"));
        assert!(!is_auth_error("symbol not found"));
    }
}
