//! Error hygiene for the adapters that carry an API key.

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
