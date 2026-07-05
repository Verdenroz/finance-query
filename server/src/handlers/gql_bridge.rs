//! REST → GraphQL execution bridge: shared by every `handlers/*` module that
//! delegates field selection/execution to the in-process `FinanceSchema`.
//!
//! Mirrors the MCP bridge in `finance-query-mcp/src/tools/gql.rs` — REST and
//! MCP both splice a validated selection set into a query string, execute
//! against the same shared schema, and map GraphQL errors back to their
//! respective transport's error shape.

use axum::{Json, http::StatusCode, response::IntoResponse};
use tracing::error;

use finance_query_server::graphql;
use finance_query_server::graphql::pagination::{connection_nodes, connection_page_info};
use finance_query_server::services::{parse_interval, parse_range};

/// (REST path key, GraphQL field name, VALID fields, composite sub-field map).
pub(crate) type RestTypeSpec = (
    &'static str,
    &'static str,
    &'static [&'static str],
    &'static [(&'static str, &'static str)],
);

/// Build a selection set for a type with composite (object-typed) top-level
/// fields, expanding any composite field with its required nested
/// sub-selection instead of splicing it in bare (invalid GraphQL for a
/// non-scalar field).
pub(crate) fn build_rest_composite_selection(
    fields: Option<&str>,
    valid_fields: &[&str],
    composite_fields: &[(&str, &str)],
) -> String {
    let mut chosen: Vec<&str> = match fields {
        Some(raw) if !raw.trim().is_empty() => raw
            .split(',')
            .map(|s| s.trim())
            .filter(|f| !f.is_empty() && valid_fields.contains(f))
            .collect(),
        _ => valid_fields.to_vec(),
    };
    // Every requested name was unknown — fall back to the full set rather
    // than emitting an empty (syntactically invalid) GraphQL selection.
    if chosen.is_empty() {
        chosen = valid_fields.to_vec();
    }
    let mut sel = String::from("{ ");
    for f in chosen {
        sel.push_str(f);
        if let Some((_, nested)) = composite_fields.iter().find(|(n, _)| *n == f) {
            sel.push(' ');
            sel.push_str(nested);
        }
        sel.push(' ');
    }
    sel.push('}');
    sel
}

// Build a GraphQL selection set from an optional comma-separated `fields` param.
// When `fields` is None, empty, or matches no `valid_fields` entry, selects
// all `valid_fields` — an empty selection set is invalid GraphQL syntax, so a
// caller typo must fall back rather than produce a hard parse error.
pub(crate) fn build_rest_selection(fields: Option<&str>, valid_fields: &[&str]) -> String {
    let requested: Vec<&str> = match fields {
        Some(raw) if !raw.trim().is_empty() => raw
            .split(',')
            .map(|s| s.trim())
            .filter(|f| !f.is_empty() && valid_fields.contains(f))
            .collect(),
        _ => Vec::new(),
    };
    let chosen: &[&str] = if requested.is_empty() {
        valid_fields
    } else {
        &requested
    };
    let mut sel = String::from("{ ");
    for f in chosen {
        sel.push_str(f);
        sel.push(' ');
    }
    sel.push('}');
    sel
}

/// Execute a GraphQL query for a REST handler, mapping GraphQL errors back to
/// the correct HTTP status code via the `status` extension `to_gql_error` sets
/// (same taxonomy as `error_response`/`gql_errors_to_mcp`) instead of a blanket
/// 400. Returns the raw (still-enveloped) JSON `data` on success, or a ready
/// `Response` to return directly on error.
pub(crate) async fn execute_gql_rest(
    schema: &graphql::FinanceSchema,
    query: &str,
    variables: async_graphql::Variables,
) -> Result<serde_json::Value, axum::response::Response> {
    let timer = crate::metrics::GraphqlTimer::new("rest_bridge");
    let response: async_graphql::Response = schema
        .execute(async_graphql::Request::new(query).variables(variables))
        .await;
    timer.observe(response.errors.is_empty());

    if !response.errors.is_empty() {
        let status = response.errors.iter().find_map(|e| {
            e.extensions
                .as_ref()
                .and_then(|ext| ext.get("status"))
                .and_then(|v| serde_json::to_value(v).ok())
                .and_then(|v| v.as_i64())
                .map(|s| s as u16)
        });
        let msg: String = response
            .errors
            .iter()
            .map(|e| e.message.as_str())
            .collect::<Vec<_>>()
            .join("; ");
        let http_status = match status {
            Some(s) => StatusCode::from_u16(s).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
            None => StatusCode::BAD_REQUEST,
        };
        error!("GraphQL query failed: {}", msg);
        let error_body = serde_json::json!({ "error": msg, "status": http_status.as_u16() });
        return Err((http_status, Json(error_body)).into_response());
    }

    Ok(response.data.into_json().unwrap_or(serde_json::Value::Null))
}

/// Reshape a GraphQL Connection JSON value (`{edges:[{node,...}], pageInfo}`)
/// into REST's legacy bare-array shape when the caller didn't opt into
/// pagination (`paginated = false`), or the new `{items, pageInfo}` envelope
/// when they did (passed `limit`/`cursor`). Keeps existing REST responses
/// byte-identical by default even though every converted field now returns a
/// `Connection` under the hood.
pub(crate) fn unwrap_connection(data: serde_json::Value, paginated: bool) -> serde_json::Value {
    let nodes = connection_nodes(&data);
    if paginated {
        serde_json::json!({ "items": nodes, "pageInfo": connection_page_info(&data) })
    } else {
        serde_json::Value::Array(nodes)
    }
}

// Map a REST interval string to a GqlInterval enum literal for GraphQL query building.
pub(crate) fn interval_to_gql(s: &str) -> &'static str {
    use finance_query::Interval;
    match parse_interval(s) {
        Interval::OneMinute => "ONE_MINUTE",
        Interval::FiveMinutes => "FIVE_MINUTES",
        Interval::FifteenMinutes => "FIFTEEN_MINUTES",
        Interval::ThirtyMinutes => "THIRTY_MINUTES",
        Interval::OneHour => "ONE_HOUR",
        Interval::OneDay => "ONE_DAY",
        Interval::OneWeek => "ONE_WEEK",
        Interval::OneMonth => "ONE_MONTH",
        Interval::ThreeMonths => "THREE_MONTHS",
    }
}

pub(crate) fn range_to_gql(s: &str) -> &'static str {
    use finance_query::TimeRange;
    match parse_range(s) {
        TimeRange::OneDay => "ONE_DAY",
        TimeRange::FiveDays => "FIVE_DAYS",
        TimeRange::OneMonth => "ONE_MONTH",
        TimeRange::ThreeMonths => "THREE_MONTHS",
        TimeRange::SixMonths => "SIX_MONTHS",
        TimeRange::OneYear => "ONE_YEAR",
        TimeRange::TwoYears => "TWO_YEARS",
        TimeRange::FiveYears => "FIVE_YEARS",
        TimeRange::TenYears => "TEN_YEARS",
        TimeRange::YearToDate => "YEAR_TO_DATE",
        TimeRange::Max => "MAX",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const VALID: &[&str] = &["symbol", "shortName", "regularMarketPrice"];
    const COMPOSITE: &[(&str, &str)] = &[("meta", "{ currency }")];

    #[test]
    fn build_rest_selection_uses_requested_valid_fields() {
        let sel = build_rest_selection(Some("symbol,shortName"), VALID);
        assert!(sel.contains("symbol"));
        assert!(sel.contains("shortName"));
        assert!(!sel.contains("regularMarketPrice"));
    }

    #[test]
    fn build_rest_selection_falls_back_to_all_when_omitted() {
        let sel = build_rest_selection(None, VALID);
        for f in VALID {
            assert!(sel.contains(f));
        }
    }

    #[test]
    fn build_rest_selection_falls_back_to_all_when_every_requested_field_is_unknown() {
        // Regression: previously produced a bare "{ }", which is invalid
        // GraphQL syntax and surfaced as a confusing parser error to callers.
        let sel = build_rest_selection(Some("bogus1,bogus2"), VALID);
        assert_ne!(sel, "{ }");
        for f in VALID {
            assert!(sel.contains(f));
        }
    }

    #[test]
    fn build_rest_composite_selection_falls_back_to_all_when_every_requested_field_is_unknown() {
        let sel = build_rest_composite_selection(Some("nope,also_nope"), VALID, COMPOSITE);
        assert_ne!(sel, "{ }");
        for f in VALID {
            assert!(sel.contains(f));
        }
    }
}
