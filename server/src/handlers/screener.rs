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
use finance_query_server::params::{CustomScreenerRequest, ScreenersQuery};
use tracing::info;

use super::gql_bridge::{build_rest_composite_selection, execute_gql_rest};
use super::support::parse_format;

// Delegates to the shared `parse_format` so an omitted `format` resolves to the
// same default every other endpoint uses.
fn format_to_gql(format: Option<ValueFormat>) -> &'static str {
    match parse_format(format) {
        ValueFormat::Raw => "RAW",
        ValueFormat::Pretty => "PRETTY",
        ValueFormat::Both => "BOTH",
    }
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
    Path(screener): Path<Screener>,
    Query(params): Query<ScreenersQuery>,
) -> impl IntoResponse {
    let gql_type = screener.as_scr_id().to_uppercase();
    let gql_format = format_to_gql(params.format);
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
        "Fetching {:?} screener (count={}, format={:?}, fields={:?})",
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
    let sort_ascending = body.sort_type == Some(finance_query::SortType::Asc);
    let gql_format = format_to_gql(body.format);
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
