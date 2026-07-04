use async_graphql::{Name, Variables};
use axum::{
    extract::{Extension, Path, Query},
    http::StatusCode,
    response::{IntoResponse, Json},
};
use finance_query_server::graphql::{
    self,
    fields::{
        DIVIDENDS_COMPOSITE_FIELDS, GQL_DIVIDENDS_VALID_FIELDS, GQL_SPLIT_VALID_FIELDS,
        gql_string_list_literal, unwrap_field, unwrap_ticker_field,
    },
};
use serde::Deserialize;
use tracing::info;

use super::gql_bridge::{
    build_rest_composite_selection, build_rest_selection, execute_gql_rest, range_to_gql,
};

fn default_max_range() -> String {
    "max".to_string()
}

#[derive(Deserialize)]
pub(crate) struct RangeQuery {
    #[serde(default = "default_max_range")]
    range: String,
    /// Comma-separated list of fields to include in response
    fields: Option<String>,
}

#[derive(Deserialize)]
pub(crate) struct BatchDividendsQuery {
    /// Comma-separated symbols (required)
    symbols: String,
    #[serde(default = "default_max_range")]
    range: String,
    /// Comma-separated list of fields to include in response
    fields: Option<String>,
}

#[derive(Deserialize)]
pub(crate) struct BatchSplitsQuery {
    /// Comma-separated symbols (required)
    symbols: String,
    #[serde(default = "default_max_range")]
    range: String,
    /// Comma-separated list of fields to include in response
    fields: Option<String>,
}

#[derive(Deserialize)]
pub(crate) struct BatchCapitalGainsQuery {
    /// Comma-separated symbols (required)
    symbols: String,
    #[serde(default = "default_max_range")]
    range: String,
    /// Comma-separated list of fields to include in response
    fields: Option<String>,
}

/// GET /v2/dividends/{symbol}
///
/// Query: `range` (str, default "max")
pub(crate) async fn get_dividends(
    Extension(schema): Extension<graphql::FinanceSchema>,
    Path(symbol): Path<String>,
    Query(params): Query<RangeQuery>,
) -> impl IntoResponse {
    let gql_range = range_to_gql(&params.range);
    let selection = build_rest_composite_selection(
        params.fields.as_deref(),
        GQL_DIVIDENDS_VALID_FIELDS,
        DIVIDENDS_COMPOSITE_FIELDS,
    );
    let query = format!(
        "query GetDivs($symbol: String!) {{ ticker(symbol: $symbol) {{ dividends(range: {gql_range}) {selection} }} }}"
    );
    info!(
        "Fetching dividends for {} (range={:?})",
        symbol, params.range
    );

    let mut vars = Variables::default();
    vars.insert(Name::new("symbol"), symbol.clone().into());
    let data = match execute_gql_rest(&schema, &query, vars).await {
        Ok(d) => d,
        Err(resp) => return resp,
    };
    (StatusCode::OK, Json(unwrap_ticker_field(data, "dividends"))).into_response()
}

/// GET /v2/splits/{symbol}
pub(crate) async fn get_splits(
    Extension(schema): Extension<graphql::FinanceSchema>,
    Path(symbol): Path<String>,
    Query(params): Query<RangeQuery>,
) -> impl IntoResponse {
    let gql_range = range_to_gql(&params.range);
    let selection = build_rest_selection(params.fields.as_deref(), GQL_SPLIT_VALID_FIELDS);
    let query = format!(
        "query GetSplits($symbol: String!) {{ ticker(symbol: $symbol) {{ splits(range: {gql_range}) {selection} }} }}"
    );
    info!("Fetching splits for {} (range={:?})", symbol, params.range);

    let mut vars = Variables::default();
    vars.insert(Name::new("symbol"), symbol.clone().into());
    let data = match execute_gql_rest(&schema, &query, vars).await {
        Ok(d) => d,
        Err(resp) => return resp,
    };
    (StatusCode::OK, Json(unwrap_ticker_field(data, "splits"))).into_response()
}

/// GET /v2/capital-gains/{symbol}
pub(crate) async fn get_capital_gains(
    Extension(schema): Extension<graphql::FinanceSchema>,
    Path(symbol): Path<String>,
    Query(params): Query<RangeQuery>,
) -> impl IntoResponse {
    let gql_range = range_to_gql(&params.range);
    let selection = build_rest_selection(params.fields.as_deref(), &["timestamp", "amount"]);
    let query = format!(
        "query GetCG($symbol: String!) {{ ticker(symbol: $symbol) {{ capitalGains(range: {gql_range}) {selection} }} }}"
    );
    info!(
        "Fetching capital gains for {} (range={:?})",
        symbol, params.range
    );

    let mut vars = Variables::default();
    vars.insert(Name::new("symbol"), symbol.clone().into());
    let data = match execute_gql_rest(&schema, &query, vars).await {
        Ok(d) => d,
        Err(resp) => return resp,
    };
    (
        StatusCode::OK,
        Json(unwrap_ticker_field(data, "capitalGains")),
    )
        .into_response()
}

/// GET /v2/dividends?symbols=<csv>&range=<str>
pub(crate) async fn get_batch_dividends(
    Extension(schema): Extension<graphql::FinanceSchema>,
    Query(params): Query<BatchDividendsQuery>,
) -> impl IntoResponse {
    let symbols: Vec<&str> = params.symbols.split(',').map(|s| s.trim()).collect();
    let gql_range = range_to_gql(&params.range);
    // Top-level wrapper fields are "symbol"/"dividends" (GqlSymbolDividends); "dividends" is a
    // plain Vec<GqlDividend> list (no per-symbol analytics, unlike single-symbol GqlDividends).
    let want_dividends = params
        .fields
        .as_deref()
        .map(|f| f.split(',').any(|x| x.trim() == "dividends"))
        .unwrap_or(true);
    let selection = if want_dividends {
        "{ symbol dividends { timestamp amount } }".to_string()
    } else {
        "{ symbol }".to_string()
    };
    let syms_literal = finance_query_server::graphql::fields::gql_string_list_literal(&symbols);
    let query = format!(
        "query {{ dividendsBatch(symbols: [{}], range: {}) {{ dividends {} errors {{ symbol message }} }} }}",
        syms_literal, gql_range, selection
    );
    info!(
        "Fetching batch dividends for {} symbols (range={})",
        symbols.len(),
        params.range
    );

    let data = match execute_gql_rest(&schema, &query, Variables::default()).await {
        Ok(d) => d,
        Err(resp) => return resp,
    };
    (StatusCode::OK, Json(unwrap_field(data, "dividendsBatch"))).into_response()
}

/// GET /v2/splits?symbols=<csv>&range=<str>
pub(crate) async fn get_batch_splits(
    Extension(schema): Extension<graphql::FinanceSchema>,
    Query(params): Query<BatchSplitsQuery>,
) -> impl IntoResponse {
    let symbols: Vec<&str> = params.symbols.split(',').map(|s| s.trim()).collect();
    let gql_range = range_to_gql(&params.range);
    let syms_literal = gql_string_list_literal(&symbols);
    let item_selection = build_rest_selection(params.fields.as_deref(), GQL_SPLIT_VALID_FIELDS);

    let query = format!(
        "query {{ splitsBatch(symbols: [{}], range: {}) {{ splits {{ symbol splits {} }} errors {{ symbol message }} }} }}",
        syms_literal, gql_range, item_selection
    );

    info!(
        "Fetching batch splits for {} symbols (range={})",
        symbols.len(),
        params.range
    );

    let data = match execute_gql_rest(&schema, &query, Variables::default()).await {
        Ok(d) => d,
        Err(resp) => return resp,
    };
    (StatusCode::OK, Json(unwrap_field(data, "splitsBatch"))).into_response()
}

/// GET /v2/capital-gains?symbols=<csv>&range=<str>
pub(crate) async fn get_batch_capital_gains(
    Extension(schema): Extension<graphql::FinanceSchema>,
    Query(params): Query<BatchCapitalGainsQuery>,
) -> impl IntoResponse {
    let symbols: Vec<&str> = params.symbols.split(',').map(|s| s.trim()).collect();
    let gql_range = range_to_gql(&params.range);
    let syms_literal = gql_string_list_literal(&symbols);
    let item_selection = build_rest_selection(params.fields.as_deref(), &["timestamp", "amount"]);

    let query = format!(
        "query {{ capitalGainsBatch(symbols: [{}], range: {}) {{ capitalGains {{ symbol capitalGains {} }} errors {{ symbol message }} }} }}",
        syms_literal, gql_range, item_selection
    );

    info!(
        "Fetching batch capital gains for {} symbols (range={})",
        symbols.len(),
        params.range
    );

    let data = match execute_gql_rest(&schema, &query, Variables::default()).await {
        Ok(d) => d,
        Err(resp) => return resp,
    };
    (
        StatusCode::OK,
        Json(unwrap_field(data, "capitalGainsBatch")),
    )
        .into_response()
}
