//! Shared internal helpers for adapter modules.

use percent_encoding::{AsciiSet, CONTROLS, utf8_percent_encode};
use reqwest::StatusCode;

use crate::error::{FinanceError, Result};

/// Coin id / ticker vocabulary shared by the keyless crypto exchanges.
#[cfg(any(feature = "binance", feature = "kraken"))]
pub(crate) mod coins;

/// Canonical chart assembly shared by the keyless crypto exchanges.
#[cfg(any(feature = "binance", feature = "kraken"))]
pub(crate) mod crypto_chart;

/// `YYYY-MM-DD` date-range parsing shared by calendar-style adapters.
pub(crate) mod date_range;

/// Error hygiene shared by the adapters that carry an API key.
#[cfg(any(
    feature = "alphavantage",
    feature = "bls",
    feature = "fmp",
    feature = "fred",
    feature = "polygon"
))]
pub(crate) mod keyed;

/// Numeric parsing shared by the macro-data adapters.
#[cfg(any(feature = "bls", feature = "fiscaldata"))]
pub(crate) mod numbers;

/// Period-label resolution shared by the macro-data adapters.
#[cfg(any(feature = "bls", feature = "worldbank"))]
pub(crate) mod periods;

/// Percent-string parsing shared by the adapters that report `"N%"` values.
#[cfg(any(feature = "alphavantage", feature = "fmp"))]
pub(crate) mod percent;

/// Characters that must be percent-encoded inside a URL path segment.
///
/// We use a conservative set: the unreserved characters per RFC 3986
/// (`ALPHA / DIGIT / "-" / "." / "_" / "~"`) are left as-is, everything
/// else — including sub-delims, gen-delims, and `/` — is encoded. Notably
/// this does NOT apply dot-segment resolution, so `".."` survives as
/// literal `..` rather than collapsing the URL path.
const PATH_SEGMENT_ENCODE_SET: &AsciiSet = &CONTROLS
    .add(b' ')
    .add(b'"')
    .add(b'#')
    .add(b'<')
    .add(b'>')
    .add(b'?')
    .add(b'`')
    .add(b'{')
    .add(b'}')
    .add(b'/')
    .add(b'%')
    .add(b':')
    .add(b';')
    .add(b'=')
    .add(b'@')
    .add(b'[')
    .add(b'\\')
    .add(b']')
    .add(b'^')
    .add(b'|')
    .add(b'!')
    .add(b'$')
    .add(b'&')
    .add(b'\'')
    .add(b'(')
    .add(b')')
    .add(b'*')
    .add(b'+')
    .add(b',');

/// Percent-encode a string for safe inclusion as a URL path segment.
///
/// Encodes characters that would otherwise alter URL structure: `?`, `#`,
/// `/`, whitespace, etc. Unlike `url::Url::path_segments_mut().push`, this
/// does NOT apply RFC 3986 dot-segment removal, so a malicious input of
/// `".."` is preserved literally and cannot collapse a path component.
///
/// Use whenever a user-supplied symbol/ticker/CIK is interpolated into a
/// URL path via `format!`.
#[allow(dead_code)] // unused when no URL-building adapter feature is enabled
pub(crate) fn encode_path_segment(segment: &str) -> String {
    utf8_percent_encode(segment, PATH_SEGMENT_ENCODE_SET).to_string()
}

/// The `User-Agent` every adapter identifies itself with. Several keyless
/// public APIs (World Bank, FiscalData, Stooq, …) throttle or reject clients
/// that send no identifiable agent.
#[allow(dead_code)] // used by the keyless adapter modules
pub(crate) fn user_agent() -> String {
    format!(
        "finance-query/{} (https://github.com/Verdenroz/finance-query)",
        env!("CARGO_PKG_VERSION")
    )
}

/// Build a keyless adapter's HTTP client. Deliberately constructed per call —
/// a `reqwest::Client` is bound to the runtime that first drives it, so a
/// cached one fails with hyper `DispatchGone` once that runtime is dropped.
#[allow(dead_code)] // used by the keyless adapter modules
pub(crate) fn keyless_http_client(timeout: std::time::Duration) -> Result<reqwest::Client> {
    Ok(reqwest::Client::builder()
        .timeout(timeout)
        .user_agent(user_agent())
        .build()?)
}

/// The error a non-2xx status maps to when the adapter has no more specific
/// reading of it. `api` names the upstream service in the error message.
///
/// Adapters that distinguish extra statuses match those arms first and let
/// this handle the tail, so every adapter reports the same shape for the
/// statuses none of them special-case.
#[allow(dead_code)] // used by the keyless adapter modules
pub(crate) fn status_error(api: &'static str, status: StatusCode) -> FinanceError {
    match status {
        StatusCode::TOO_MANY_REQUESTS => FinanceError::RateLimited { retry_after: None },
        s => FinanceError::ExternalApiError {
            api: api.to_string(),
            status: s.as_u16(),
        },
    }
}

/// Fail a non-2xx response via [`status_error`]; succeed on any 2xx.
#[allow(dead_code)] // used by the keyless adapter modules
pub(crate) fn check_status(api: &'static str, status: StatusCode) -> Result<()> {
    if status.is_success() {
        return Ok(());
    }
    Err(status_error(api, status))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn plain_symbol_unchanged() {
        assert_eq!(encode_path_segment("AAPL"), "AAPL");
    }

    #[test]
    fn dot_separated_ticker_unchanged() {
        assert_eq!(encode_path_segment("BRK.B"), "BRK.B");
    }

    #[test]
    fn question_mark_is_encoded() {
        assert_eq!(encode_path_segment("FOO?bar"), "FOO%3Fbar");
    }

    #[test]
    fn hash_is_encoded() {
        assert_eq!(encode_path_segment("FOO#bar"), "FOO%23bar");
    }

    #[test]
    fn slash_is_encoded() {
        assert_eq!(encode_path_segment("a/b"), "a%2Fb");
    }

    #[test]
    fn space_is_encoded() {
        assert_eq!(encode_path_segment("a b"), "a%20b");
    }

    #[test]
    fn dot_dot_is_preserved_literally() {
        // Critical: must NOT collapse to empty (no dot-segment removal).
        // The literal ".." characters are unreserved, so they pass through.
        assert_eq!(encode_path_segment(".."), "..");
    }

    #[test]
    fn rate_limit_status_maps_to_rate_limited() {
        assert!(matches!(
            status_error("Test", StatusCode::TOO_MANY_REQUESTS),
            FinanceError::RateLimited { retry_after: None }
        ));
    }

    #[test]
    fn other_failures_carry_the_api_name_and_status() {
        match status_error("Test", StatusCode::BAD_GATEWAY) {
            FinanceError::ExternalApiError { api, status } => {
                assert_eq!(api, "Test");
                assert_eq!(status, 502);
            }
            other => panic!("unexpected error: {other:?}"),
        }
    }

    #[test]
    fn success_statuses_pass_check() {
        assert!(check_status("Test", StatusCode::OK).is_ok());
        assert!(check_status("Test", StatusCode::NO_CONTENT).is_ok());
        assert!(check_status("Test", StatusCode::NOT_FOUND).is_err());
    }

    #[test]
    fn dot_dot_slash_is_encoded() {
        // The slash must be encoded so a malicious "../foo" cannot
        // navigate a path component upward in the resulting URL.
        assert_eq!(encode_path_segment("../foo"), "..%2Ffoo");
    }
}
