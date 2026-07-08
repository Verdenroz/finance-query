use async_graphql::Variables;
use axum::{
    extract::{Extension, Path, Query},
    http::StatusCode,
    response::{IntoResponse, Json},
};
use finance_query::{Screener, ValueFormat};
use finance_query_server::graphql::{
    self,
    fields::{GQL_SCREENER_RESULTS_VALID_FIELDS, SCREENER_RESULTS_COMPOSITE_FIELDS, unwrap_field},
};
use serde::Deserialize;
use tracing::info;

use super::gql_bridge::{build_rest_composite_selection, execute_gql_rest};
use super::support::parse_format;

fn default_screeners_count() -> u32 {
    std::env::var("SCREENERS_COUNT")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(25)
}

// Delegates to the shared `parse_format` (same `fmt`/`full` aliases every other
// endpoint accepts) instead of re-matching the raw string.
fn format_to_gql(format: Option<&str>) -> &'static str {
    match parse_format(format) {
        ValueFormat::Raw => "RAW",
        ValueFormat::Pretty => "PRETTY",
        ValueFormat::Both => "BOTH",
    }
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ScreenersQuery {
    #[serde(default = "default_screeners_count")]
    count: u32,
    /// Value format: raw, pretty, or both (default: raw)
    format: Option<String>,
    /// Comma-separated list of fields to include in response
    fields: Option<String>,
}

/// Request body for custom screener endpoint
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct CustomScreenerRequest {
    /// Number of results (default: 25, max: 250)
    #[serde(default = "default_screeners_count")]
    size: u32,
    /// Pagination offset (default: 0)
    #[serde(default)]
    offset: u32,
    /// Sort direction: "ASC" or "DESC" (default: DESC)
    #[serde(default)]
    sort_type: Option<String>,
    /// Field to sort by (default: intradaymarketcap)
    sort_field: Option<String>,
    /// Quote type: EQUITY, ETF, MUTUALFUND, etc. (default: EQUITY)
    quote_type: Option<String>,
    /// Filter conditions
    #[serde(default)]
    filters: Vec<FilterCondition>,
    /// Value format: raw, pretty, or both (default: raw)
    format: Option<String>,
    /// Comma-separated list of fields to include in response
    fields: Option<String>,
}

/// A single filter condition
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct FilterCondition {
    /// Field name (e.g., "region", "avgdailyvol3m", "intradaymarketcap")
    field: String,
    /// Operator: eq, gt, gte, lt, lte, btwn
    operator: String,
    /// Value(s) for the condition
    value: serde_json::Value,
}

/// GET /v2/screeners/{screener}
///
/// Path params:
/// - `screener`: One of 15 predefined screener identifiers (kebab-case)
///   - Equity: aggressive-small-caps, day-gainers, day-losers, growth-technology-stocks,
///     most-actives, most-shorted-stocks, small-cap-gainers, undervalued-growth-stocks,
///     undervalued-large-caps
///   - Fund: conservative-foreign-funds, high-yield-bond, portfolio-anchors,
///     solid-large-growth-funds, solid-midcap-growth-funds, top-mutual-funds
///
/// Query: `count` (u32, default 25, max 250), `format` (raw|pretty|both), `fields` (comma-separated)
pub(crate) async fn get_screeners(
    Extension(schema): Extension<graphql::FinanceSchema>,
    Path(screener): Path<String>,
    Query(params): Query<ScreenersQuery>,
) -> impl IntoResponse {
    let st = match screener.parse::<Screener>() {
        Ok(t) => t,
        Err(_) => {
            let error = serde_json::json!({
                "error": format!("Invalid screener: '{}'. Valid types: {}", screener, Screener::valid_types()),
                "status": 400
            });
            return (StatusCode::BAD_REQUEST, Json(error)).into_response();
        }
    };
    let gql_type = st.as_scr_id().to_uppercase();
    let gql_format = format_to_gql(params.format.as_deref());
    let selection = build_rest_composite_selection(
        params.fields.as_deref(),
        GQL_SCREENER_RESULTS_VALID_FIELDS,
        SCREENER_RESULTS_COMPOSITE_FIELDS,
    );

    let query = format!(
        "query {{ screener(type: {}, count: {}, format: {}) {} }}",
        gql_type, params.count, gql_format, selection
    );

    info!(
        "Fetching {} screener (count={}, format={:?}, fields={:?})",
        screener, params.count, params.format, params.fields
    );

    let data = match execute_gql_rest(&schema, &query, Variables::default()).await {
        Ok(d) => d,
        Err(resp) => return resp,
    };
    (StatusCode::OK, Json(unwrap_field(data, "screener"))).into_response()
}

/// POST /v2/screeners/custom
///
/// Execute a custom screener query with flexible filtering.
///
/// Request body:
/// ```json
/// {
///   "size": 25,
///   "offset": 0,
///   "sortType": "DESC",
///   "sortField": "intradaymarketcap",
///   "quoteType": "EQUITY",
///   "filters": [
///     {"field": "region", "operator": "eq", "value": "us"},
///     {"field": "avgdailyvol3m", "operator": "gt", "value": 200000}
///   ],
///   "format": "raw",
///   "fields": "symbol,shortName,regularMarketPrice"
/// }
/// ```
pub(crate) async fn post_custom_screener(
    Extension(schema): Extension<graphql::FinanceSchema>,
    Json(body): Json<CustomScreenerRequest>,
) -> impl IntoResponse {
    let sort_ascending = body
        .sort_type
        .as_deref()
        .map(|s| s.to_lowercase() == "asc")
        .unwrap_or(false);
    let gql_format = format_to_gql(body.format.as_deref());
    let selection = build_rest_composite_selection(
        body.fields.as_deref(),
        GQL_SCREENER_RESULTS_VALID_FIELDS,
        SCREENER_RESULTS_COMPOSITE_FIELDS,
    );

    let filter_count = body.filters.len();
    let filters_json: Vec<serde_json::Value> = body
        .filters
        .iter()
        .map(|f| {
            serde_json::json!({
                "field": f.field,
                "operator": f.operator,
                "value": f.value,
            })
        })
        .collect();

    let vars_json = serde_json::json!({
        "input": {
            "size": body.size,
            "offset": body.offset,
            "sortAscending": sort_ascending,
            "sortField": body.sort_field,
            "quoteType": body.quote_type,
            "filters": filters_json,
        }
    });
    let variables = Variables::from_json(vars_json);

    let query = format!(
        "query($input: GqlCustomScreenerInput!) {{ customScreener(input: $input, format: {}) {} }}",
        gql_format, selection
    );

    info!(
        "Executing custom screener (size={}, filters={})",
        body.size, filter_count
    );

    let data = match execute_gql_rest(&schema, &query, variables).await {
        Ok(d) => d,
        Err(resp) => return resp,
    };
    (StatusCode::OK, Json(unwrap_field(data, "customScreener"))).into_response()
}
