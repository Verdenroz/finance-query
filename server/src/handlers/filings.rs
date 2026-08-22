use async_graphql::{Name, Variables};
use axum::{
    extract::{Extension, Path, Query},
    http::StatusCode,
    response::{IntoResponse, Json},
};
use finance_query_server::graphql::{
    self,
    fields::{
        GQL_CONGRESSIONAL_TRADE_VALID_FIELDS, GQL_FAIL_TO_DELIVER_VALID_FIELDS, unwrap_ticker_field,
    },
    pagination::build_connection_selection,
};
use finance_query_server::params::FilingsQuery;
use tracing::info;

use super::gql_bridge::{
    build_rest_selection, connection_args, execute_gql_rest, unwrap_connection,
};

/// GET /v2/filings/{symbol}/congressional-trades
///
/// Congressional (senate) trading disclosures for a symbol. Currently FMP
/// only — requires `FMP_API_KEY`.
pub(crate) async fn get_congressional_trades(
    Extension(schema): Extension<graphql::FinanceSchema>,
    Path(symbol): Path<String>,
    Query(params): Query<FilingsQuery>,
) -> impl IntoResponse {
    let inner_selection = build_rest_selection(
        params.fields.as_deref(),
        GQL_CONGRESSIONAL_TRADE_VALID_FIELDS,
    );
    let selection = build_connection_selection(&inner_selection);
    let conn_args = connection_args(params.limit, params.cursor.as_deref());
    let conn_args_str = if conn_args.is_empty() {
        String::new()
    } else {
        format!("({})", conn_args.join(", "))
    };
    let query = format!(
        "query GetCongressionalTrades($symbol: String!) {{ ticker(symbol: $symbol) {{ congressionalTrades{conn_args_str} {selection} }} }}"
    );

    info!("Fetching congressional trades for {}", symbol);

    let mut vars = Variables::default();
    vars.insert(Name::new("symbol"), symbol.clone().into());
    let data = match execute_gql_rest(&schema, &query, vars).await {
        Ok(d) => d,
        Err(resp) => return *resp,
    };
    let paginated = params.limit.is_some() || params.cursor.is_some();
    let result = unwrap_connection(unwrap_ticker_field(data, "congressionalTrades"), paginated);
    (StatusCode::OK, Json(result)).into_response()
}

/// GET /v2/filings/{symbol}/fails-to-deliver
///
/// Fails-to-deliver records for a symbol. Currently FMP only — requires
/// `FMP_API_KEY`.
pub(crate) async fn get_fails_to_deliver(
    Extension(schema): Extension<graphql::FinanceSchema>,
    Path(symbol): Path<String>,
    Query(params): Query<FilingsQuery>,
) -> impl IntoResponse {
    let inner_selection =
        build_rest_selection(params.fields.as_deref(), GQL_FAIL_TO_DELIVER_VALID_FIELDS);
    let selection = build_connection_selection(&inner_selection);
    let conn_args = connection_args(params.limit, params.cursor.as_deref());
    let conn_args_str = if conn_args.is_empty() {
        String::new()
    } else {
        format!("({})", conn_args.join(", "))
    };
    let query = format!(
        "query GetFailsToDeliver($symbol: String!) {{ ticker(symbol: $symbol) {{ failsToDeliver{conn_args_str} {selection} }} }}"
    );

    info!("Fetching fails-to-deliver records for {}", symbol);

    let mut vars = Variables::default();
    vars.insert(Name::new("symbol"), symbol.clone().into());
    let data = match execute_gql_rest(&schema, &query, vars).await {
        Ok(d) => d,
        Err(resp) => return *resp,
    };
    let paginated = params.limit.is_some() || params.cursor.is_some();
    let result = unwrap_connection(unwrap_ticker_field(data, "failsToDeliver"), paginated);
    (StatusCode::OK, Json(result)).into_response()
}
