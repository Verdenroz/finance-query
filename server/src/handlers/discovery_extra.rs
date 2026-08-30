//! TVL, symbol detail, index constituent changes and sector history.

use async_graphql::{Name, Variables};
use axum::{
    Extension, Json,
    extract::{Path, Query},
    http::StatusCode,
    response::IntoResponse,
};
use finance_query_server::graphql::fields::{
    GQL_INDEX_CONSTITUENT_CHANGE_VALID_FIELDS, GQL_PROTOCOL_TVL_VALID_FIELDS,
    GQL_SECTOR_PERFORMANCE_HISTORY_VALID_FIELDS, GQL_SYMBOL_DETAILS_VALID_FIELDS,
    GQL_TVL_POINT_VALID_FIELDS, PROTOCOL_TVL_COMPOSITE_FIELDS,
    SECTOR_PERFORMANCE_HISTORY_COMPOSITE_FIELDS, escape_gql_string, unwrap_field,
};
use finance_query_server::graphql::pagination::build_connection_selection;
use finance_query_server::params::{AnalysisQuery, FilingsQuery};
use tracing::info;

use super::gql_bridge::{
    build_rest_composite_selection, build_rest_selection, connection_args, execute_gql_rest,
    unwrap_connection,
};
use crate::graphql;

fn conn_args_str(limit: Option<u32>, cursor: Option<&str>) -> String {
    let args = connection_args(limit, cursor);
    match args.is_empty() {
        true => String::new(),
        false => format!("({})", args.join(", ")),
    }
}

/// GET /v2/crypto/coins/{id}/tvl
pub(crate) async fn get_protocol_tvl(
    Extension(schema): Extension<graphql::FinanceSchema>,
    Path(id): Path<String>,
    Query(params): Query<AnalysisQuery>,
) -> impl IntoResponse {
    let selection = build_rest_composite_selection(
        params.fields.as_deref(),
        GQL_PROTOCOL_TVL_VALID_FIELDS,
        PROTOCOL_TVL_COMPOSITE_FIELDS,
    );
    let query = format!(
        "query GetTvl {{ cryptoCoin(id: \"{}\") {{ tvl {selection} }} }}",
        escape_gql_string(&id)
    );
    info!("Fetching protocol TVL for {}", id);
    let data = match execute_gql_rest(&schema, &query, Variables::default()).await {
        Ok(d) => d,
        Err(resp) => return *resp,
    };
    let result = unwrap_field(unwrap_field(data, "cryptoCoin"), "tvl");
    (StatusCode::OK, Json(result)).into_response()
}

/// GET /v2/crypto/coins/{id}/tvl-history
pub(crate) async fn get_protocol_tvl_history(
    Extension(schema): Extension<graphql::FinanceSchema>,
    Path(id): Path<String>,
    Query(params): Query<FilingsQuery>,
) -> impl IntoResponse {
    let inner = build_rest_selection(params.fields.as_deref(), GQL_TVL_POINT_VALID_FIELDS);
    let selection = build_connection_selection(&inner);
    let args = conn_args_str(params.limit, params.cursor.as_deref());
    let query = format!(
        "query GetTvlHistory {{ cryptoCoin(id: \"{}\") {{ tvlHistory{args} {selection} }} }}",
        escape_gql_string(&id)
    );
    info!("Fetching protocol TVL history for {}", id);
    let data = match execute_gql_rest(&schema, &query, Variables::default()).await {
        Ok(d) => d,
        Err(resp) => return *resp,
    };
    let paginated = params.limit.is_some() || params.cursor.is_some();
    let result = unwrap_connection(
        unwrap_field(unwrap_field(data, "cryptoCoin"), "tvlHistory"),
        paginated,
    );
    (StatusCode::OK, Json(result)).into_response()
}

/// GET /v2/symbol-details/{symbol}
pub(crate) async fn get_symbol_details(
    Extension(schema): Extension<graphql::FinanceSchema>,
    Path(symbol): Path<String>,
    Query(params): Query<AnalysisQuery>,
) -> impl IntoResponse {
    let selection = build_rest_selection(params.fields.as_deref(), GQL_SYMBOL_DETAILS_VALID_FIELDS);
    let query = format!(
        "query GetSymbolDetails($symbol: String!) {{ symbolDetails(symbol: $symbol) {selection} }}"
    );
    info!("Fetching symbol details for {}", symbol);
    let mut vars = Variables::default();
    vars.insert(Name::new("symbol"), symbol.clone().into());
    let data = match execute_gql_rest(&schema, &query, vars).await {
        Ok(d) => d,
        Err(resp) => return *resp,
    };
    (StatusCode::OK, Json(unwrap_field(data, "symbolDetails"))).into_response()
}

/// GET /v2/indices/{symbol}/constituent-changes
pub(crate) async fn get_index_constituent_changes(
    Extension(schema): Extension<graphql::FinanceSchema>,
    Path(symbol): Path<String>,
    Query(params): Query<FilingsQuery>,
) -> impl IntoResponse {
    let inner = build_rest_selection(
        params.fields.as_deref(),
        GQL_INDEX_CONSTITUENT_CHANGE_VALID_FIELDS,
    );
    let selection = build_connection_selection(&inner);
    let args = conn_args_str(params.limit, params.cursor.as_deref());
    let inner_args = args.trim_start_matches('(').trim_end_matches(')');
    let arg_suffix = match inner_args.is_empty() {
        true => String::new(),
        false => format!(", {inner_args}"),
    };
    let query = format!(
        "query GetChanges($symbol: String!) {{ indexConstituentChanges(symbol: $symbol{arg_suffix}) {selection} }}"
    );
    info!("Fetching index constituent changes for {}", symbol);
    let mut vars = Variables::default();
    vars.insert(Name::new("symbol"), symbol.clone().into());
    let data = match execute_gql_rest(&schema, &query, vars).await {
        Ok(d) => d,
        Err(resp) => return *resp,
    };
    let paginated = params.limit.is_some() || params.cursor.is_some();
    let result = unwrap_connection(unwrap_field(data, "indexConstituentChanges"), paginated);
    (StatusCode::OK, Json(result)).into_response()
}

/// GET /v2/sector-performance/history
pub(crate) async fn get_sector_performance_history(
    Extension(schema): Extension<graphql::FinanceSchema>,
    Query(params): Query<FilingsQuery>,
) -> impl IntoResponse {
    let selection = build_rest_composite_selection(
        params.fields.as_deref(),
        GQL_SECTOR_PERFORMANCE_HISTORY_VALID_FIELDS,
        SECTOR_PERFORMANCE_HISTORY_COMPOSITE_FIELDS,
    );
    let limit = params.limit.unwrap_or(30);
    let query = format!(
        "query GetSectorHistory {{ sectorPerformanceHistory(limit: {limit}) {selection} }}"
    );
    info!("Fetching sector performance history (limit={limit})");
    let data = match execute_gql_rest(&schema, &query, Variables::default()).await {
        Ok(d) => d,
        Err(resp) => return *resp,
    };
    (
        StatusCode::OK,
        Json(unwrap_field(data, "sectorPerformanceHistory")),
    )
        .into_response()
}
