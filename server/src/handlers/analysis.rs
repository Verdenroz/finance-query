use async_graphql::{Name, Variables};
use axum::{
    extract::{Extension, Path, Query},
    http::StatusCode,
    response::{IntoResponse, Json},
};
use finance_query_server::graphql::{
    self,
    fields::{
        GQL_COMPANY_PROFILE_VALID_FIELDS, GQL_EARNINGS_ESTIMATE_COMPOSITE,
        GQL_EARNINGS_ESTIMATE_VALID_FIELDS, GQL_EARNINGS_HISTORY_COMPOSITE,
        GQL_EARNINGS_HISTORY_VALID_FIELDS, GQL_EARNINGS_SURPRISES_COMPOSITE,
        GQL_EARNINGS_SURPRISES_VALID_FIELDS, GQL_EARNINGS_TRANSCRIPT_VALID_FIELDS,
        GQL_ETF_COUNTRY_WEIGHTING_COMPOSITE, GQL_ETF_HOLDING_COMPOSITE,
        GQL_ETF_PROFILE_VALID_FIELDS, GQL_ETF_SECTOR_WEIGHTING_COMPOSITE,
        GQL_GRADING_HISTORY_COMPOSITE, GQL_GRADING_HISTORY_VALID_FIELDS,
        GQL_PRICE_TARGET_CONSENSUS_VALID_FIELDS, GQL_RATING_CONSENSUS_VALID_FIELDS,
        GQL_RECOMMENDATION_TREND_COMPOSITE, GQL_RECOMMENDATION_TREND_VALID_FIELDS,
        GQL_RECOMMENDATION_VALID_FIELDS, RECOMMENDATION_COMPOSITE_FIELDS, gql_string_list_literal,
        unwrap_field, unwrap_ticker_field,
    },
    pagination::{build_connection_selection, unwrap_nested_connection},
};
use finance_query_server::params::{
    AnalysisQuery, AnalysisType, BatchRecommendationsQuery, EarningsTranscriptV2Query,
    RecommendationsQuery,
};
use tracing::info;

use super::gql_bridge::{
    RestTypeSpec, build_rest_composite_selection, build_rest_selection, execute_gql_rest,
};

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
        Err(resp) => return *resp,
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
        Err(resp) => return *resp,
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
        Err(resp) => return *resp,
    };
    (StatusCode::OK, Json(unwrap_ticker_field(data, gql_field))).into_response()
}

/// GET /v2/company-profile/{symbol}
///
/// Company identity/classification profile. Currently Alpha Vantage only —
/// requires `ALPHAVANTAGE_API_KEY` to be configured.
pub(crate) async fn get_company_profile(
    Extension(schema): Extension<graphql::FinanceSchema>,
    Path(symbol): Path<String>,
    Query(params): Query<AnalysisQuery>,
) -> impl IntoResponse {
    let selection =
        build_rest_selection(params.fields.as_deref(), GQL_COMPANY_PROFILE_VALID_FIELDS);
    let query = format!(
        "query GetCompanyProfile($symbol: String!) {{ ticker(symbol: $symbol) {{ companyProfile {selection} }} }}"
    );
    info!(
        "Fetching company profile for {} (fields={:?})",
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
        Json(unwrap_ticker_field(data, "companyProfile")),
    )
        .into_response()
}

/// GET /v2/earnings-surprises/{symbol}
///
/// Earnings-surprise history. Currently FMP and Alpha Vantage.
pub(crate) async fn get_earnings_surprises(
    Extension(schema): Extension<graphql::FinanceSchema>,
    Path(symbol): Path<String>,
    Query(params): Query<AnalysisQuery>,
) -> impl IntoResponse {
    let selection = build_rest_composite_selection(
        params.fields.as_deref(),
        GQL_EARNINGS_SURPRISES_VALID_FIELDS,
        &[("surprises", GQL_EARNINGS_SURPRISES_COMPOSITE)],
    );
    let query = format!(
        "query GetEarningsSurprises($symbol: String!) {{ ticker(symbol: $symbol) {{ earningsSurprises {selection} }} }}"
    );
    info!(
        "Fetching earnings surprises for {} (fields={:?})",
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
        Json(unwrap_ticker_field(data, "earningsSurprises")),
    )
        .into_response()
}

/// GET /v2/rating-consensus/{symbol}
///
/// Consensus rating rollup (analyst grade distribution + headline label).
pub(crate) async fn get_rating_consensus(
    Extension(schema): Extension<graphql::FinanceSchema>,
    Path(symbol): Path<String>,
    Query(params): Query<AnalysisQuery>,
) -> impl IntoResponse {
    let selection =
        build_rest_selection(params.fields.as_deref(), GQL_RATING_CONSENSUS_VALID_FIELDS);
    let query = format!(
        "query GetRatingConsensus($symbol: String!) {{ ticker(symbol: $symbol) {{ ratingConsensus {selection} }} }}"
    );
    info!(
        "Fetching rating consensus for {} (fields={:?})",
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
        Json(unwrap_ticker_field(data, "ratingConsensus")),
    )
        .into_response()
}

/// GET /v2/price-target-consensus/{symbol}
///
/// Consensus analyst price target.
pub(crate) async fn get_price_target_consensus(
    Extension(schema): Extension<graphql::FinanceSchema>,
    Path(symbol): Path<String>,
    Query(params): Query<AnalysisQuery>,
) -> impl IntoResponse {
    let selection = build_rest_selection(
        params.fields.as_deref(),
        GQL_PRICE_TARGET_CONSENSUS_VALID_FIELDS,
    );
    let query = format!(
        "query GetPriceTargetConsensus($symbol: String!) {{ ticker(symbol: $symbol) {{ priceTargetConsensus {selection} }} }}"
    );
    info!(
        "Fetching price target consensus for {} (fields={:?})",
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
        Json(unwrap_ticker_field(data, "priceTargetConsensus")),
    )
        .into_response()
}

/// GET /v2/etf-profile/{symbol}
///
/// ETF profile and holdings (only meaningful for ETF symbols).
pub(crate) async fn get_etf_profile(
    Extension(schema): Extension<graphql::FinanceSchema>,
    Path(symbol): Path<String>,
    Query(params): Query<AnalysisQuery>,
) -> impl IntoResponse {
    let selection = build_rest_composite_selection(
        params.fields.as_deref(),
        GQL_ETF_PROFILE_VALID_FIELDS,
        &[
            ("holdings", GQL_ETF_HOLDING_COMPOSITE),
            ("sectorWeightings", GQL_ETF_SECTOR_WEIGHTING_COMPOSITE),
            ("countryWeightings", GQL_ETF_COUNTRY_WEIGHTING_COMPOSITE),
        ],
    );
    let query = format!(
        "query GetEtfProfile($symbol: String!) {{ ticker(symbol: $symbol) {{ etfProfile {selection} }} }}"
    );
    info!(
        "Fetching ETF profile for {} (fields={:?})",
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
        Json(unwrap_ticker_field(data, "etfProfile")),
    )
        .into_response()
}

/// GET /v2/earnings-transcript/{symbol}
///
/// Earnings call transcript, provider-neutral shape (Yahoo or Alpha
/// Vantage), distinct from `/v2/transcripts/{symbol}`'s richer Yahoo-only
/// shape. Query: `quarter`, `year` (both required for Alpha Vantage; Yahoo
/// defaults to latest when omitted).
pub(crate) async fn get_earnings_transcript(
    Extension(schema): Extension<graphql::FinanceSchema>,
    Path(symbol): Path<String>,
    Query(params): Query<EarningsTranscriptV2Query>,
) -> impl IntoResponse {
    let quarter_arg = params
        .quarter
        .map(|q| format!("quarter: \"{}\"", q.as_str()));
    let year_arg = params.year.map(|y| format!("year: {y}"));
    let args: Vec<String> = [quarter_arg, year_arg].into_iter().flatten().collect();
    let args_str = if args.is_empty() {
        String::new()
    } else {
        format!("({})", args.join(", "))
    };
    let selection = build_rest_selection(
        params.fields.as_deref(),
        GQL_EARNINGS_TRANSCRIPT_VALID_FIELDS,
    );
    let query = format!(
        "query GetEarningsTranscript($symbol: String!) {{ ticker(symbol: $symbol) {{ earningsTranscript{args_str} {selection} }} }}"
    );
    info!(
        "Fetching earnings transcript for {} (quarter={:?}, year={:?})",
        symbol, params.quarter, params.year
    );
    let mut vars = Variables::default();
    vars.insert(Name::new("symbol"), symbol.clone().into());
    let data = match execute_gql_rest(&schema, &query, vars).await {
        Ok(d) => d,
        Err(resp) => return *resp,
    };
    (
        StatusCode::OK,
        Json(unwrap_ticker_field(data, "earningsTranscript")),
    )
        .into_response()
}
