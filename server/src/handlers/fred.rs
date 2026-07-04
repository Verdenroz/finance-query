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
};
use serde::Deserialize;
use tracing::info;

use super::gql_bridge::{build_rest_composite_selection, build_rest_selection, execute_gql_rest};

#[derive(Deserialize)]
pub(crate) struct FredSeriesQuery {
    /// Comma-separated list of fields to include in response
    fields: Option<String>,
}

/// Query parameters for /v2/fred/treasury-yields
#[derive(Deserialize)]
pub(crate) struct TreasuryYieldsQuery {
    /// Calendar year (default: current year)
    year: Option<u32>,
    /// Comma-separated list of fields to include in response
    fields: Option<String>,
}

/// GET /v2/fred/series/{id}
///
/// Fetch observations for a FRED data series. Requires `FRED_API_KEY` to be set.
pub(crate) async fn get_fred_series(
    Extension(schema): Extension<graphql::FinanceSchema>,
    Path(series_id): Path<String>,
    Query(params): Query<FredSeriesQuery>,
) -> impl IntoResponse {
    let selection = build_rest_composite_selection(
        params.fields.as_deref(),
        GQL_MACRO_SERIES_VALID_FIELDS,
        MACRO_SERIES_COMPOSITE_FIELDS,
    );
    let query = format!("query GetFredSeries($id: String!) {{ fredSeries(id: $id) {selection} }}");
    let mut vars = Variables::default();
    vars.insert(Name::new("id"), series_id.clone().into());

    info!("Fetching FRED series: {}", series_id);

    let data = match execute_gql_rest(&schema, &query, vars).await {
        Ok(d) => d,
        Err(resp) => return resp,
    };
    (StatusCode::OK, Json(unwrap_field(data, "fredSeries"))).into_response()
}

/// GET /v2/fred/treasury-yields
///
/// Query: `year` (u32, default: current year)
pub(crate) async fn get_fred_treasury_yields(
    Extension(schema): Extension<graphql::FinanceSchema>,
    Query(params): Query<TreasuryYieldsQuery>,
) -> impl IntoResponse {
    let selection = build_rest_selection(params.fields.as_deref(), GQL_TREASURY_YIELD_VALID_FIELDS);
    let year_arg = match params.year {
        Some(y) => format!("(year: {y})"),
        None => String::new(),
    };
    let query = format!("query {{ treasuryYields{year_arg} {selection} }}");

    info!("Fetching Treasury yields for year {:?}", params.year);

    let data = match execute_gql_rest(&schema, &query, Variables::default()).await {
        Ok(d) => d,
        Err(resp) => return resp,
    };
    (StatusCode::OK, Json(unwrap_field(data, "treasuryYields"))).into_response()
}
