use async_graphql::{Name, Variables};
use axum::{
    extract::{Extension, Path, Query},
    http::StatusCode,
    response::{IntoResponse, Json},
};
use finance_query_server::graphql::{self, fields::unwrap_ticker_field};
use serde::Deserialize;
use tracing::info;

use super::gql_bridge::{build_rest_selection, execute_gql_rest, interval_to_gql, range_to_gql};
use super::support::{default_interval, default_range};

/// Query parameters for /v2/risk/{symbol}
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct RiskQuery {
    #[serde(default = "default_interval")]
    interval: String,
    #[serde(default = "default_range")]
    range: String,
    /// Optional benchmark symbol for beta calculation (e.g., "SPY")
    benchmark: Option<String>,
    /// Comma-separated list of fields to include in response
    fields: Option<String>,
}

/// GET /v2/risk/{symbol}
///
/// Query: `interval` (str, default "1d"), `range` (str, default "1y"), `benchmark` (str, optional)
pub(crate) async fn get_risk(
    Extension(schema): Extension<graphql::FinanceSchema>,
    Path(symbol): Path<String>,
    Query(params): Query<RiskQuery>,
) -> impl IntoResponse {
    let gql_interval = interval_to_gql(&params.interval);
    let gql_range = range_to_gql(&params.range);
    let has_benchmark = params.benchmark.as_deref().is_some_and(|b| !b.is_empty());
    let bench_arg = if has_benchmark {
        ", benchmark: $benchmark"
    } else {
        ""
    };
    // GraphQL rejects a declared operation variable that's never referenced
    // in the query body, so $benchmark can only be declared when it's used.
    let benchmark_decl = if has_benchmark {
        ", $benchmark: String"
    } else {
        ""
    };
    let selection = build_rest_selection(
        params.fields.as_deref(),
        &[
            "var95",
            "var99",
            "parametricVar95",
            "sharpe",
            "sortino",
            "calmar",
            "beta",
            "maxDrawdown",
            "maxDrawdownRecoveryPeriods",
        ],
    );
    let query = format!(
        "query GetRisk($symbol: String!{benchmark_decl}) {{ ticker(symbol: $symbol) {{ risk(interval: {gql_interval}, range: {gql_range}{bench_arg}) {selection} }} }}"
    );
    info!(
        "Fetching risk analytics for {} (interval={}, range={}, benchmark={:?})",
        symbol, params.interval, params.range, params.benchmark
    );

    let mut vars = Variables::default();
    vars.insert(Name::new("symbol"), symbol.clone().into());
    if let Some(b) = params.benchmark.as_deref().filter(|b| !b.is_empty()) {
        vars.insert(Name::new("benchmark"), b.into());
    }
    let data = match execute_gql_rest(&schema, &query, vars).await {
        Ok(d) => d,
        Err(resp) => return resp,
    };
    (StatusCode::OK, Json(unwrap_ticker_field(data, "risk"))).into_response()
}
