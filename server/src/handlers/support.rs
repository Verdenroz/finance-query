//! Shared REST helpers used across multiple `handlers/*` domain modules:
//! query-param parsing.

use finance_query::ValueFormat;

/// Parse format query parameter into ValueFormat
pub(crate) fn parse_format(s: Option<&str>) -> ValueFormat {
    s.and_then(ValueFormat::parse).unwrap_or_default()
}

/// Default chart interval, overridable via `DEFAULT_INTERVAL` env var.
pub(crate) fn default_interval() -> String {
    std::env::var("DEFAULT_INTERVAL").unwrap_or_else(|_| "1d".to_string())
}

/// Default chart range, overridable via `DEFAULT_RANGE` env var.
pub(crate) fn default_range() -> String {
    std::env::var("DEFAULT_RANGE").unwrap_or_else(|_| "1mo".to_string())
}
