//! Shared REST helpers used across multiple `handlers/*` domain modules:
//! query-param parsing, `ValueFormat`/field-filtering transforms, and
//! `FinanceError` → HTTP response mapping.

use axum::{Json, http::StatusCode, response::IntoResponse};
use finance_query::{FinanceError, ValueFormat};
use finance_query_server::metrics;
use std::collections::HashSet;

/// Parse format query parameter into ValueFormat
pub(crate) fn parse_format(s: Option<&str>) -> ValueFormat {
    s.and_then(ValueFormat::parse).unwrap_or_default()
}

/// Parse comma-separated field names into a set for filtering
pub(crate) fn parse_fields(s: Option<&str>) -> Option<HashSet<String>> {
    s.map(|fields_str| {
        fields_str
            .split(',')
            .map(|f| f.trim().to_string())
            .filter(|f| !f.is_empty())
            .collect()
    })
}

/// Recursively filter a JSON value to only include specified fields.
///
/// Strategy: an object is treated as a **data object** if it has at least one
/// key that directly matches `fields`. In that case only matching keys are kept
/// and all non-matching keys (including nested containers) are dropped.
///
/// If an object has *no* direct matches it is treated as a **transparent
/// wrapper** (e.g. `{ "quotes": { "AAPL": {...} } }`). Its container-typed
/// values are recursed into and the key is kept only when the result is
/// non-empty — preventing false positives from deeply nested metadata fields
/// (e.g. `equityPerformance.benchmark.symbol`) leaking through.
///
/// Arrays always recurse into each element; empty objects produced by
/// filtering are removed from the result.
pub(crate) fn filter_fields(
    value: serde_json::Value,
    fields: &HashSet<String>,
) -> serde_json::Value {
    match value {
        serde_json::Value::Object(map) => {
            let has_direct_match = map.keys().any(|k| fields.contains(k));

            let filtered = map
                .into_iter()
                .filter_map(|(k, v)| {
                    if fields.contains(&k) {
                        // Explicitly requested: keep whole value.
                        Some((k, v))
                    } else if !has_direct_match
                        && matches!(
                            v,
                            serde_json::Value::Object(_) | serde_json::Value::Array(_)
                        )
                    {
                        // Pure wrapper — recurse and prune if nothing matched.
                        let filtered_v = filter_fields(v, fields);
                        match &filtered_v {
                            serde_json::Value::Object(m) if m.is_empty() => None,
                            serde_json::Value::Array(a) if a.is_empty() => None,
                            _ => Some((k, filtered_v)),
                        }
                    } else {
                        None
                    }
                })
                .collect();
            serde_json::Value::Object(filtered)
        }
        serde_json::Value::Array(arr) => {
            let filtered: Vec<_> = arr
                .into_iter()
                .map(|v| filter_fields(v, fields))
                .filter(|v| !matches!(v, serde_json::Value::Object(m) if m.is_empty()))
                .collect();
            serde_json::Value::Array(filtered)
        }
        other => other,
    }
}

/// Apply format transformation and optional field filtering
pub(crate) fn apply_transforms(
    value: serde_json::Value,
    format: ValueFormat,
    fields: Option<&HashSet<String>>,
) -> serde_json::Value {
    let formatted = format.transform(value);
    match fields {
        Some(f) => filter_fields(formatted, f),
        None => formatted,
    }
}

/// Default chart interval, overridable via `DEFAULT_INTERVAL` env var.
pub(crate) fn default_interval() -> String {
    std::env::var("DEFAULT_INTERVAL").unwrap_or_else(|_| "1d".to_string())
}

/// Default chart range, overridable via `DEFAULT_RANGE` env var.
pub(crate) fn default_range() -> String {
    std::env::var("DEFAULT_RANGE").unwrap_or_else(|_| "1mo".to_string())
}

/// Converts generic errors from get_or_fetch into HTTP responses
/// Attempts to downcast to FinanceError to preserve HTTP status code mapping
pub(crate) fn into_error_response(
    e: Box<dyn std::error::Error + Send + Sync>,
) -> axum::response::Response {
    // Try to downcast to FinanceError first to get proper HTTP status codes
    if let Some(yahoo_err) = e.downcast_ref::<FinanceError>() {
        // Track error by type
        let error_type = match yahoo_err {
            FinanceError::SymbolNotFound { .. } => "symbol_not_found",
            FinanceError::AuthenticationFailed { .. } => "authentication_failed",
            FinanceError::RateLimited { .. } => "rate_limited",
            FinanceError::Timeout { .. } => "timeout",
            FinanceError::ServerError { .. } => "server_error",
            _ => "other",
        };
        metrics::ERRORS_TOTAL.with_label_values(&[error_type]).inc();

        // Map FinanceError to appropriate HTTP status codes
        let status = match yahoo_err {
            FinanceError::SymbolNotFound { .. } => StatusCode::NOT_FOUND,
            FinanceError::AuthenticationFailed { .. } => StatusCode::UNAUTHORIZED,
            FinanceError::RateLimited { .. } => StatusCode::TOO_MANY_REQUESTS,
            FinanceError::Timeout { .. } => StatusCode::REQUEST_TIMEOUT,
            FinanceError::ServerError { status, .. } => {
                StatusCode::from_u16(*status).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR)
            }
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        };
        return (
            status,
            Json(serde_json::json!({
                "error": yahoo_err.to_string(),
                "status": status.as_u16()
            })),
        )
            .into_response();
    }

    // Fallback for other errors (e.g., serialization errors)
    metrics::ERRORS_TOTAL
        .with_label_values(&["serialization"])
        .inc();
    (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json(serde_json::json!({
            "error": e.to_string(),
            "status": 500
        })),
    )
        .into_response()
}
