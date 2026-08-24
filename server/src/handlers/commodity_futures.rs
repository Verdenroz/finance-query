use async_graphql::{Name, Variables};
use axum::{
    extract::{Extension, Path, Query},
    http::StatusCode,
    response::{IntoResponse, Json},
};
use finance_query_server::graphql::{
    self,
    fields::{
        GQL_COMMODITY_QUOTE_VALID_FIELDS, GQL_FUTURES_QUOTE_VALID_FIELDS,
        GQL_INDEX_CONSTITUENT_VALID_FIELDS, unwrap_field,
    },
};
use finance_query_server::params::{CommodityQuery, FuturesQuery, IndexConstituentsQuery};
use tracing::info;

use super::gql_bridge::{build_rest_selection, execute_gql_rest};

/// GET /v2/commodities/{symbol}
///
/// A commodity's current quote (e.g. gold, silver, crude oil),
/// provider-routed via `Providers::commodity()` (Yahoo, keyless).
pub(crate) async fn get_commodity(
    Extension(schema): Extension<graphql::FinanceSchema>,
    Path(symbol): Path<String>,
    Query(params): Query<CommodityQuery>,
) -> impl IntoResponse {
    let selection =
        build_rest_selection(params.fields.as_deref(), GQL_COMMODITY_QUOTE_VALID_FIELDS);
    let query = format!(
        "query GetCommodity($symbol: String!) {{ commodity(symbol: $symbol) {selection} }}"
    );
    info!(
        "Fetching commodity quote for {} (fields={:?})",
        symbol, params.fields
    );
    let mut vars = Variables::default();
    vars.insert(Name::new("symbol"), symbol.clone().into());
    let data = match execute_gql_rest(&schema, &query, vars).await {
        Ok(d) => d,
        Err(resp) => return *resp,
    };
    (StatusCode::OK, Json(unwrap_field(data, "commodity"))).into_response()
}

/// GET /v2/futures/{symbol}
///
/// A futures contract's current quote, provider-routed via
/// `Providers::futures()` (Yahoo, keyless).
pub(crate) async fn get_futures(
    Extension(schema): Extension<graphql::FinanceSchema>,
    Path(symbol): Path<String>,
    Query(params): Query<FuturesQuery>,
) -> impl IntoResponse {
    let selection = build_rest_selection(params.fields.as_deref(), GQL_FUTURES_QUOTE_VALID_FIELDS);
    let query =
        format!("query GetFutures($symbol: String!) {{ futures(symbol: $symbol) {selection} }}");
    info!(
        "Fetching futures quote for {} (fields={:?})",
        symbol, params.fields
    );
    let mut vars = Variables::default();
    vars.insert(Name::new("symbol"), symbol.clone().into());
    let data = match execute_gql_rest(&schema, &query, vars).await {
        Ok(d) => d,
        Err(resp) => return *resp,
    };
    (StatusCode::OK, Json(unwrap_field(data, "futures"))).into_response()
}

/// GET /v2/index-constituents/{symbol}
///
/// An index's current constituent list, provider-routed via
/// `Providers::index()` (Wikipedia, S&P 500 only).
pub(crate) async fn get_index_constituents(
    Extension(schema): Extension<graphql::FinanceSchema>,
    Path(symbol): Path<String>,
    Query(params): Query<IndexConstituentsQuery>,
) -> impl IntoResponse {
    let selection =
        build_rest_selection(params.fields.as_deref(), GQL_INDEX_CONSTITUENT_VALID_FIELDS);
    let query = format!(
        "query GetIndexConstituents($symbol: String!) {{ indexConstituents(symbol: $symbol) {selection} }}"
    );
    info!(
        "Fetching index constituents for {} (fields={:?})",
        symbol, params.fields
    );
    let mut vars = Variables::default();
    vars.insert(Name::new("symbol"), symbol.clone().into());
    let data = match execute_gql_rest(&schema, &query, vars).await {
        Ok(d) => d,
        Err(resp) => return *resp,
    };
    (
        StatusCode::OK,
        Json(unwrap_field(data, "indexConstituents")),
    )
        .into_response()
}
