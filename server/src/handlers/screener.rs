use async_graphql::Variables;
use axum::{
    extract::{Extension, Path, Query},
    http::StatusCode,
    response::{IntoResponse, Json},
};
use finance_query::{Screener, ValueFormat};
use finance_query_server::graphql::{
    self,
    fields::{
        GQL_SCREENER_QUOTE_FIELDS, GQL_SCREENER_RESULTS_VALID_FIELDS,
        SCREENER_RESULTS_COMPOSITE_FIELDS, unwrap_field,
    },
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

/// Names usable in `fields` to narrow each quote, taken from the quote selection.
fn quote_field_names() -> impl Iterator<Item = &'static str> {
    GQL_SCREENER_QUOTE_FIELDS
        .split_whitespace()
        .filter(|t| *t != "{" && *t != "}")
}

/// Selection for a screener response. `fields` may name result fields
/// (`quotes`, `total`) or per-quote fields (`symbol`); per-quote names narrow
/// the `quotes` objects, and imply `quotes` when no result field is named.
fn build_screener_selection(fields: Option<&str>) -> String {
    let requested: Vec<&str> = fields
        .unwrap_or_default()
        .split(',')
        .map(str::trim)
        .filter(|f| !f.is_empty())
        .collect();
    let quote_fields: Vec<&str> = requested
        .iter()
        .copied()
        .filter(|f| quote_field_names().any(|q| q == *f))
        .collect();
    if quote_fields.is_empty() {
        return build_rest_composite_selection(
            fields,
            GQL_SCREENER_RESULTS_VALID_FIELDS,
            SCREENER_RESULTS_COMPOSITE_FIELDS,
        );
    }

    let mut result_fields: Vec<&str> = requested
        .iter()
        .copied()
        .filter(|f| GQL_SCREENER_RESULTS_VALID_FIELDS.contains(f) && *f != "quotes")
        .collect();
    result_fields.insert(0, "quotes");
    let page_info = SCREENER_RESULTS_COMPOSITE_FIELDS
        .iter()
        .find(|(name, _)| *name == "pageInfo")
        .map_or("", |(_, nested)| *nested);
    let body: Vec<String> = result_fields
        .iter()
        .map(|f| match *f {
            "quotes" => format!("quotes {{ {} }}", quote_fields.join(" ")),
            "pageInfo" => format!("pageInfo {page_info}"),
            other => other.to_string(),
        })
        .collect();
    format!("{{ {} }}", body.join(" "))
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
    let selection = build_screener_selection(params.fields.as_deref());

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
        Err(resp) => return *resp,
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
    let selection = build_screener_selection(body.fields.as_deref());

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
        Err(resp) => return *resp,
    };
    (StatusCode::OK, Json(unwrap_field(data, "customScreener"))).into_response()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn omitted_fields_select_the_whole_result() {
        let sel = build_screener_selection(None);
        assert!(sel.contains("quotes {"));
        assert!(sel.contains("regularMarketPrice"));
        assert!(sel.contains("total"));
    }

    #[test]
    fn per_quote_fields_narrow_the_quotes() {
        let sel = build_screener_selection(Some("symbol,regularMarketPrice"));
        assert_eq!(sel, "{ quotes { symbol regularMarketPrice } }");
    }

    #[test]
    fn result_fields_ride_along_with_per_quote_fields() {
        let sel = build_screener_selection(Some("symbol,total,pageInfo"));
        assert!(sel.starts_with("{ quotes { symbol } total pageInfo { hasNextPage"));
    }

    #[test]
    fn result_fields_alone_keep_the_full_quote_selection() {
        let sel = build_screener_selection(Some("quotes,total"));
        assert!(sel.contains("shortName"));
        assert!(sel.contains("total"));
        assert!(!sel.contains("pageInfo"));
    }

    #[test]
    fn unknown_fields_fall_back_to_the_whole_result() {
        assert_eq!(
            build_screener_selection(Some("nope")),
            build_screener_selection(None)
        );
    }
}
