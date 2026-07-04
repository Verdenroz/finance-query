use async_graphql::{Name, Variables};
use axum::{
    extract::{Extension, Path, Query},
    http::StatusCode,
    response::{IntoResponse, Json},
};
use finance_query_server::graphql::{
    self,
    fields::{
        GQL_INDICATORS_VALID_FIELDS, INDICATOR_COMPOSITE_FIELDS, gql_string_list_literal,
        unwrap_field, unwrap_ticker_field,
    },
};
use serde::Deserialize;
use tracing::info;

use super::gql_bridge::{
    build_rest_composite_selection, execute_gql_rest, interval_to_gql, range_to_gql,
};
use super::support::{default_interval, default_range};

#[derive(Deserialize)]
pub(crate) struct BatchIndicatorsQuery {
    /// Comma-separated symbols (required)
    symbols: String,
    #[serde(default = "default_interval")]
    interval: String,
    #[serde(default = "default_range")]
    range: String,
    /// Comma-separated list of fields to include in response
    fields: Option<String>,
}

/// Query params for /v2/indicators/{symbol}, shared with the chart-range shape.
#[derive(Deserialize)]
pub(crate) struct IndicatorsQuery {
    #[serde(default = "default_interval")]
    interval: String,
    #[serde(default = "default_range")]
    range: String,
    /// Comma-separated list of fields to include in response
    fields: Option<String>,
}

/// GET /v2/indicators/{symbol}
pub(crate) async fn get_indicators(
    Extension(schema): Extension<graphql::FinanceSchema>,
    Path(symbol): Path<String>,
    Query(params): Query<IndicatorsQuery>,
) -> impl IntoResponse {
    let gql_interval = interval_to_gql(&params.interval);
    let gql_range = range_to_gql(&params.range);
    let selection = build_rest_indicators_selection(params.fields.as_deref());
    let query = format!(
        "query GetIndicators($symbol: String!) {{ ticker(symbol: $symbol) {{ indicators(interval: {gql_interval}, range: {gql_range}) {selection} }} }}"
    );
    info!(
        "Calculating indicators for {} with interval={}, range={}",
        symbol, params.interval, params.range
    );
    let mut vars = Variables::default();
    vars.insert(Name::new("symbol"), symbol.clone().into());
    let data = match execute_gql_rest(&schema, &query, vars).await {
        Ok(d) => d,
        Err(resp) => return resp,
    };
    (
        StatusCode::OK,
        Json(unwrap_ticker_field(data, "indicators")),
    )
        .into_response()
}

/// GET /v2/indicators?symbols=<csv>&interval=<str>&range=<str>
pub(crate) async fn get_batch_indicators(
    Extension(schema): Extension<graphql::FinanceSchema>,
    Query(params): Query<BatchIndicatorsQuery>,
) -> impl IntoResponse {
    let gql_interval = interval_to_gql(&params.interval);
    let gql_range = range_to_gql(&params.range);
    let syms: Vec<&str> = params.symbols.split(',').map(|s| s.trim()).collect();
    let syms_literal = gql_string_list_literal(&syms);
    // Top-level batch wrapper fields are "symbol"/"indicators" (GqlSymbolIndicators);
    // "indicators" is itself composite and needs its own nested sub-selection.
    let want_indicators = params
        .fields
        .as_deref()
        .map(|f| f.split(',').any(|x| x.trim() == "indicators"))
        .unwrap_or(true);
    let selection = if want_indicators {
        format!(
            "{{ symbol indicators {} }}",
            build_rest_indicators_selection(params.fields.as_deref())
        )
    } else {
        "{ symbol }".to_string()
    };
    let query = format!(
        "query {{ indicatorsBatch(symbols: [{}], interval: {gql_interval}, range: {gql_range}) {{ indicators {} errors {{ symbol message }} }} }}",
        syms_literal, selection
    );
    info!(
        "Fetching batch indicators for {} symbols (interval={}, range={})",
        syms.len(),
        params.interval,
        params.range
    );
    let data = match execute_gql_rest(&schema, &query, Variables::default()).await {
        Ok(d) => d,
        Err(resp) => return resp,
    };
    (StatusCode::OK, Json(unwrap_field(data, "indicatorsBatch"))).into_response()
}

/// Build the `indicators { ... }` selection set, expanding any composite
/// field (stochastic, macd, aroon, ...) with its required nested
/// sub-selection — mirrors `build_indicators_selection` in
/// finance-query-mcp/src/tools/gql.rs.
fn build_rest_indicators_selection(fields: Option<&str>) -> String {
    build_rest_composite_selection(
        fields,
        GQL_INDICATORS_VALID_FIELDS,
        INDICATOR_COMPOSITE_FIELDS,
    )
}
