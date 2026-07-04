use async_graphql::{Name, Variables};
use axum::{
    extract::{Extension, Path, Query},
    http::StatusCode,
    response::{IntoResponse, Json},
};
use finance_query_server::graphql::{
    self,
    fields::{
        GQL_MACRO_SERIES_VALID_FIELDS, GQL_TREASURY_YIELD_VALID_FIELDS,
        MACRO_SERIES_COMPOSITE_FIELDS, unwrap_field,
    },
    pagination::{
        build_connection_selection, build_paginated_composite_selection, unwrap_nested_connection,
    },
};
use serde::Deserialize;
use tracing::info;

use super::gql_bridge::{build_rest_selection, execute_gql_rest, unwrap_connection};

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct FredSeriesQuery {
    /// Comma-separated list of fields to include in response
    fields: Option<String>,
    /// Max observations per page; omitted (with cursor also omitted) = every
    /// matching observation as a bare array, unchanged from pre-pagination behavior
    limit: Option<u32>,
    /// Opaque continuation cursor from a previous response's `pageInfo.endCursor`
    cursor: Option<String>,
}

/// Query parameters for /v2/fred/treasury-yields
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct TreasuryYieldsQuery {
    /// Calendar year (default: current year)
    year: Option<u32>,
    /// Comma-separated list of fields to include in response
    fields: Option<String>,
    /// Max rows per page; omitted (with cursor also omitted) = every matching
    /// row as a bare array, unchanged from pre-pagination behavior
    limit: Option<u32>,
    /// Opaque continuation cursor from a previous response's `pageInfo.endCursor`
    cursor: Option<String>,
}

/// GET /v2/fred/series/{id}
///
/// Fetch observations for a FRED data series. Requires `FRED_API_KEY` to be set.
pub(crate) async fn get_fred_series(
    Extension(schema): Extension<graphql::FinanceSchema>,
    Path(series_id): Path<String>,
    Query(params): Query<FredSeriesQuery>,
) -> impl IntoResponse {
    let observations_item_selection = MACRO_SERIES_COMPOSITE_FIELDS
        .iter()
        .find(|(name, _)| *name == "observations")
        .map(|(_, sel)| *sel)
        .unwrap_or("{ date value }");
    let selection = build_paginated_composite_selection(
        params.fields.as_deref(),
        GQL_MACRO_SERIES_VALID_FIELDS,
        GQL_MACRO_SERIES_VALID_FIELDS,
        MACRO_SERIES_COMPOSITE_FIELDS,
        "observations",
        observations_item_selection,
        params.limit,
        params.cursor.as_deref(),
    );
    let query = format!("query GetFredSeries($id: String!) {{ fredSeries(id: $id) {selection} }}");
    let mut vars = Variables::default();
    vars.insert(Name::new("id"), series_id.clone().into());

    info!("Fetching FRED series: {}", series_id);

    let data = match execute_gql_rest(&schema, &query, vars).await {
        Ok(d) => d,
        Err(resp) => return resp,
    };
    let paginated = params.limit.is_some() || params.cursor.is_some();
    let result =
        unwrap_nested_connection(unwrap_field(data, "fredSeries"), "observations", paginated);
    (StatusCode::OK, Json(result)).into_response()
}

/// GET /v2/fred/treasury-yields
///
/// Query: `year` (u32, default: current year)
pub(crate) async fn get_fred_treasury_yields(
    Extension(schema): Extension<graphql::FinanceSchema>,
    Query(params): Query<TreasuryYieldsQuery>,
) -> impl IntoResponse {
    let inner_selection =
        build_rest_selection(params.fields.as_deref(), GQL_TREASURY_YIELD_VALID_FIELDS);
    let selection = build_connection_selection(&inner_selection);
    let mut args = Vec::new();
    if let Some(y) = params.year {
        args.push(format!("year: {y}"));
    }
    if let Some(limit) = params.limit {
        args.push(format!("first: {limit}"));
    }
    if let Some(cursor) = params.cursor.as_deref() {
        args.push(format!(
            "after: \"{}\"",
            cursor.replace('\\', "\\\\").replace('"', "\\\"")
        ));
    }
    let args_str = if args.is_empty() {
        String::new()
    } else {
        format!("({})", args.join(", "))
    };
    let query = format!("query {{ treasuryYields{args_str} {selection} }}");

    info!("Fetching Treasury yields for year {:?}", params.year);

    let data = match execute_gql_rest(&schema, &query, Variables::default()).await {
        Ok(d) => d,
        Err(resp) => return resp,
    };
    let paginated = params.limit.is_some() || params.cursor.is_some();
    let result = unwrap_connection(unwrap_field(data, "treasuryYields"), paginated);
    (StatusCode::OK, Json(result)).into_response()
}
