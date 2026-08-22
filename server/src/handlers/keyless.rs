//! REST handlers for the keyless providers: GDELT global news search and
//! CFTC Commitments of Traders. Neither needs an API key.

use async_graphql::{Name, Variables};
use axum::{
    extract::{Extension, Path, Query},
    http::StatusCode,
    response::{IntoResponse, Json},
};
use finance_query_server::graphql::{
    self,
    fields::{
        COT_COMPOSITE_FIELDS, GQL_COT_VALID_FIELDS, GQL_NEWS_VALID_FIELDS, NEWS_COMPOSITE_FIELDS,
        unwrap_field,
    },
    pagination::{
        build_connection_selection, build_paginated_composite_selection, unwrap_nested_connection,
    },
};
use finance_query_server::params::{CotQuery, GdeltNewsQuery};
use tracing::info;

use super::gql_bridge::{
    build_rest_composite_selection, connection_args, execute_gql_rest, unwrap_connection,
};

/// GET /v2/gdelt/news/{symbol}
pub(crate) async fn get_gdelt_news(
    Extension(schema): Extension<graphql::FinanceSchema>,
    Path(symbol): Path<String>,
    Query(params): Query<GdeltNewsQuery>,
) -> impl IntoResponse {
    let inner_selection = build_rest_composite_selection(
        params.fields.as_deref(),
        GQL_NEWS_VALID_FIELDS,
        NEWS_COMPOSITE_FIELDS,
    );
    let selection = build_connection_selection(&inner_selection);
    let conn_args = connection_args(params.limit, params.cursor.as_deref());
    let conn_args_str = if conn_args.is_empty() {
        String::new()
    } else {
        format!(", {}", conn_args.join(", "))
    };
    let query = format!(
        "query GdeltNews($symbol: String!) {{ gdeltNews(symbol: $symbol{conn_args_str}) {selection} }}"
    );
    let mut vars = Variables::default();
    vars.insert(Name::new("symbol"), symbol.clone().into());

    info!("Fetching GDELT news for: {symbol}");

    let data = match execute_gql_rest(&schema, &query, vars).await {
        Ok(d) => d,
        Err(resp) => return *resp,
    };
    let paginated = params.limit.is_some() || params.cursor.is_some();
    let result = unwrap_connection(unwrap_field(data, "gdeltNews"), paginated);
    (StatusCode::OK, Json(result)).into_response()
}

/// GET /v2/cftc/cot/{symbol}
pub(crate) async fn get_commitments_of_traders(
    Extension(schema): Extension<graphql::FinanceSchema>,
    Path(symbol): Path<String>,
    Query(params): Query<CotQuery>,
) -> impl IntoResponse {
    let observations_item_selection = COT_COMPOSITE_FIELDS
        .iter()
        .find(|(name, _)| *name == "observations")
        .map(|(_, sel)| *sel)
        .unwrap_or("{ reportDate openInterest }");
    let selection = build_paginated_composite_selection(
        params.fields.as_deref(),
        GQL_COT_VALID_FIELDS,
        GQL_COT_VALID_FIELDS,
        COT_COMPOSITE_FIELDS,
        "observations",
        observations_item_selection,
        params.limit,
        params.cursor.as_deref(),
    );
    let query = format!(
        "query Cot($symbol: String!) {{ commitmentsOfTraders(symbol: $symbol) {selection} }}"
    );
    let mut vars = Variables::default();
    vars.insert(Name::new("symbol"), symbol.clone().into());

    info!("Fetching CFTC Commitments of Traders for: {symbol}");

    let data = match execute_gql_rest(&schema, &query, vars).await {
        Ok(d) => d,
        Err(resp) => return *resp,
    };
    let paginated = params.limit.is_some() || params.cursor.is_some();
    let result = unwrap_nested_connection(
        unwrap_field(data, "commitmentsOfTraders"),
        "observations",
        paginated,
    );
    (StatusCode::OK, Json(result)).into_response()
}
