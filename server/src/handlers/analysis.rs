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
    pagination::{build_connection_selection, unwrap_nested_connection},
};
use finance_query_server::params::{
    AnalysisQuery, AnalysisType, BatchRecommendationsQuery, RecommendationsQuery,
};
use tracing::info;

use super::gql_bridge::{RestTypeSpec, build_rest_composite_selection, execute_gql_rest};

/// (GraphQL field name -> (VALID, composite sub-field map)) per analysis type.
/// The first element must stay in sync with every `services::analysis` per-type
/// fn and its corresponding GraphQL field.
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
    let selection = build_connection_selection(&item_selection);

    let mut conn_args = Vec::new();
    if let Some(limit) = params.page_limit {
        conn_args.push(format!("first: {limit}"));
    }
    if let Some(cursor) = params.page_cursor.as_deref() {
        conn_args.push(format!(
            "after: \"{}\"",
            cursor.replace('\\', "\\\\").replace('"', "\\\"")
        ));
    }
    let conn_args_str = if conn_args.is_empty() {
        String::new()
    } else {
        format!("({})", conn_args.join(", "))
    };

    let query = format!(
        "query {{ recommendationsBatch(symbols: [{}], limit: {}) {{ recommendations{} {} errors {{ symbol message }} }} }}",
        syms_literal, params.limit, conn_args_str, selection
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
    let paginated = params.page_limit.is_some() || params.page_cursor.is_some();
    let result = unwrap_nested_connection(
        unwrap_field(data, "recommendationsBatch"),
        "recommendations",
        paginated,
    );
    (StatusCode::OK, Json(result)).into_response()
}

/// GET /v2/analysis/{symbol}/{analysis_type}
pub(crate) async fn get_analysis(
    Extension(schema): Extension<graphql::FinanceSchema>,
    Path((symbol, analysis_type)): Path<(String, AnalysisType)>,
    Query(params): Query<AnalysisQuery>,
) -> impl IntoResponse {
    let (_, gql_field, valid_fields, composite_fields) = ANALYSIS_TYPE_REST_SPECS
        .iter()
        .find(|(k, ..)| *k == analysis_type.as_str())
        .expect("ANALYSIS_TYPE_REST_SPECS covers every AnalysisType variant");
    let selection =
        build_rest_composite_selection(params.fields.as_deref(), valid_fields, composite_fields);
    let query = format!(
        "query GetAnalysis($symbol: String!) {{ ticker(symbol: $symbol) {{ {gql_field} {selection} }} }}"
    );
    info!(
        "Fetching {:?} analysis for {} (fields={:?})",
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
