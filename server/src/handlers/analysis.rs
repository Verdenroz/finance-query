use async_graphql::{Name, Variables};
use axum::{
    extract::{Extension, Path, Query},
    http::StatusCode,
    response::{IntoResponse, Json},
};
use finance_query_server::graphql::{
    self,
    fields::{
        GQL_EARNINGS_ESTIMATE_COMPOSITE, GQL_EARNINGS_ESTIMATE_VALID_FIELDS,
        GQL_EARNINGS_HISTORY_COMPOSITE, GQL_EARNINGS_HISTORY_VALID_FIELDS,
        GQL_GRADING_HISTORY_COMPOSITE, GQL_GRADING_HISTORY_VALID_FIELDS,
        GQL_RECOMMENDATION_TREND_COMPOSITE, GQL_RECOMMENDATION_TREND_VALID_FIELDS,
        GQL_RECOMMENDATION_VALID_FIELDS, RECOMMENDATION_COMPOSITE_FIELDS, gql_string_list_literal,
        unwrap_field, unwrap_ticker_field,
    },
};
use serde::Deserialize;
use tracing::info;

use super::gql_bridge::{RestTypeSpec, build_rest_composite_selection, execute_gql_rest};

fn default_limit() -> u32 {
    std::env::var("RECOMMENDATIONS_LIMIT")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(5)
}

fn default_recommendations_limit() -> u32 {
    10
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct RecommendationsQuery {
    #[serde(default = "default_limit")]
    limit: u32,
    /// Comma-separated list of fields to include in response
    fields: Option<String>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct BatchRecommendationsQuery {
    /// Comma-separated symbols (required)
    symbols: String,
    #[serde(default = "default_recommendations_limit")]
    limit: u32,
    /// Comma-separated list of fields to include in response
    fields: Option<String>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct AnalysisQuery {
    /// Comma-separated list of fields to include in response
    fields: Option<String>,
}

/// (GraphQL field name -> (VALID, composite sub-field map)) per analysis type.
/// The first element must stay in sync with every `services::analysis::AnalysisType`
/// variant and its corresponding GraphQL field.
const ANALYSIS_TYPE_REST_SPECS: &[RestTypeSpec] = &[
    (
        "recommendations",
        "recommendationTrend",
        GQL_RECOMMENDATION_TREND_VALID_FIELDS,
        &[("trend", GQL_RECOMMENDATION_TREND_COMPOSITE)],
    ),
    (
        "upgrades-downgrades",
        "gradingHistory",
        GQL_GRADING_HISTORY_VALID_FIELDS,
        &[("history", GQL_GRADING_HISTORY_COMPOSITE)],
    ),
    (
        "earnings-estimate",
        "earningsEstimate",
        GQL_EARNINGS_ESTIMATE_VALID_FIELDS,
        &[("trend", GQL_EARNINGS_ESTIMATE_COMPOSITE)],
    ),
    (
        "earnings-history",
        "earningsHistory",
        GQL_EARNINGS_HISTORY_VALID_FIELDS,
        &[("history", GQL_EARNINGS_HISTORY_COMPOSITE)],
    ),
];

/// GET /v2/recommendations/{symbol}
///
/// Query: `limit` (u32, default via `RECOMMENDATIONS_LIMIT` or server default)
pub(crate) async fn get_recommendations(
    Extension(schema): Extension<graphql::FinanceSchema>,
    Path(symbol): Path<String>,
    Query(params): Query<RecommendationsQuery>,
) -> impl IntoResponse {
    let selection = build_rest_composite_selection(
        params.fields.as_deref(),
        GQL_RECOMMENDATION_VALID_FIELDS,
        RECOMMENDATION_COMPOSITE_FIELDS,
    );
    let query = format!(
        "query GetRecs($symbol: String!) {{ ticker(symbol: $symbol) {{ recommendations(limit: {}) {selection} }} }}",
        params.limit
    );
    let mut vars = Variables::default();
    vars.insert(Name::new("symbol"), symbol.clone().into());

    info!(
        "Fetching recommendations for {} (limit={}, fields={:?})",
        symbol, params.limit, params.fields
    );

    let data = match execute_gql_rest(&schema, &query, vars).await {
        Ok(d) => d,
        Err(resp) => return resp,
    };
    (
        StatusCode::OK,
        Json(unwrap_ticker_field(data, "recommendations")),
    )
        .into_response()
}

/// GET /v2/recommendations?symbols=<csv>&limit=<u32>
pub(crate) async fn get_batch_recommendations(
    Extension(schema): Extension<graphql::FinanceSchema>,
    Query(params): Query<BatchRecommendationsQuery>,
) -> impl IntoResponse {
    let symbols: Vec<&str> = params.symbols.split(',').map(|s| s.trim()).collect();
    let syms_literal = gql_string_list_literal(&symbols);
    let item_selection = build_rest_composite_selection(
        params.fields.as_deref(),
        GQL_RECOMMENDATION_VALID_FIELDS,
        RECOMMENDATION_COMPOSITE_FIELDS,
    );
    let query = format!(
        "query {{ recommendationsBatch(symbols: [{}], limit: {}) {{ recommendations {} errors {{ symbol message }} }} }}",
        syms_literal, params.limit, item_selection
    );

    info!(
        "Fetching batch recommendations for {} symbols (limit={})",
        symbols.len(),
        params.limit
    );

    let data = match execute_gql_rest(&schema, &query, Variables::default()).await {
        Ok(d) => d,
        Err(resp) => return resp,
    };
    (
        StatusCode::OK,
        Json(unwrap_field(data, "recommendationsBatch")),
    )
        .into_response()
}

/// GET /v2/analysis/{symbol}/{analysis_type}
pub(crate) async fn get_analysis(
    Extension(schema): Extension<graphql::FinanceSchema>,
    Path((symbol, analysis_type)): Path<(String, String)>,
    Query(params): Query<AnalysisQuery>,
) -> impl IntoResponse {
    let key = analysis_type.to_lowercase();
    let Some((_, gql_field, valid_fields, composite_fields)) = ANALYSIS_TYPE_REST_SPECS
        .iter()
        .find(|(k, ..)| *k == key.as_str())
    else {
        return (StatusCode::BAD_REQUEST, Json(serde_json::json!({
            "error": format!("Invalid analysis type: '{}'. Valid: recommendations, upgrades-downgrades, earnings-estimate, earnings-history", analysis_type),
            "status": 400
        }))).into_response();
    };
    let selection =
        build_rest_composite_selection(params.fields.as_deref(), valid_fields, composite_fields);
    let query = format!(
        "query GetAnalysis($symbol: String!) {{ ticker(symbol: $symbol) {{ {gql_field} {selection} }} }}"
    );
    info!(
        "Fetching {} analysis for {} (fields={:?})",
        analysis_type, symbol, params.fields
    );
    let mut vars = Variables::default();
    vars.insert(Name::new("symbol"), symbol.clone().into());
    let data = match execute_gql_rest(&schema, &query, vars).await {
        Ok(d) => d,
        Err(resp) => return resp,
    };
    (StatusCode::OK, Json(unwrap_ticker_field(data, gql_field))).into_response()
}
