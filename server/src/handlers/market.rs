use async_graphql::Variables;
use axum::{
    extract::{Extension, Query},
    http::{HeaderMap, StatusCode},
    response::{IntoResponse, Json},
};
use finance_query::{IndicesRegion, ValueFormat};
use finance_query_server::graphql::{
    self,
    fields::{
        GQL_FEAR_AND_GREED_VALID_FIELDS, GQL_MARKET_CALENDAR_VALID_FIELDS,
        GQL_MARKET_SECTOR_PE_VALID_FIELDS, GQL_MARKET_SECTOR_PERFORMANCE_VALID_FIELDS,
        GQL_MARKET_SUMMARY_VALID_FIELDS, GQL_QUOTE_VALID_FIELDS, GQL_TRENDING_VALID_FIELDS,
        MARKET_CALENDAR_DETAIL_UNION_SELECTION, escape_gql_string, unwrap_field,
    },
};
use finance_query_server::lang;
use finance_query_server::params::{
    FearAndGreedQuery, IndicesQuery, MarketCalendarQuery, MarketSummaryQuery, SectorPeQuery,
    SectorPerformanceQuery, TrendingQuery,
};
use tracing::info;

use super::gql_bridge::{build_rest_selection, build_rest_union_selection, execute_gql_rest};
use super::support::parse_format;

/// Map an `IndicesRegion` to a `GqlIndicesRegion` enum literal.
fn indices_region_to_gql(region: IndicesRegion) -> &'static str {
    match region {
        IndicesRegion::Americas => "AMERICAS",
        IndicesRegion::Europe => "EUROPE",
        IndicesRegion::AsiaPacific => "ASIA_PACIFIC",
        IndicesRegion::MiddleEastAfrica => "MIDDLE_EAST_AFRICA",
        IndicesRegion::Currencies => "CURRENCIES",
    }
}

/// Map a REST `format` string to the `GqlValueFormat` enum literal.
fn format_to_gql(format: ValueFormat) -> &'static str {
    match format {
        ValueFormat::Raw => "RAW",
        ValueFormat::Pretty => "PRETTY",
        ValueFormat::Both => "BOTH",
    }
}

/// GET /v2/indices
///
/// Returns quotes for world market indices, optionally filtered by region.
pub(crate) async fn get_indices(
    Extension(schema): Extension<graphql::FinanceSchema>,
    Query(params): Query<IndicesQuery>,
) -> impl IntoResponse {
    let format = parse_format(params.format);
    let selection = build_rest_selection(params.fields.as_deref(), GQL_QUOTE_VALID_FIELDS);
    let region_arg = params
        .region
        .map(|r| format!("region: {}, ", indices_region_to_gql(r)));
    let args = region_arg.unwrap_or_default();
    let query = format!(
        "query {{ indices({}format: {}) {} }}",
        args,
        format_to_gql(format),
        selection
    );

    info!(
        "Fetching indices (region={:?}, format={}, fields={:?})",
        params.region,
        format.as_str(),
        params.fields
    );

    let data = match execute_gql_rest(&schema, &query, Variables::default()).await {
        Ok(d) => d,
        Err(resp) => return *resp,
    };
    (StatusCode::OK, Json(unwrap_field(data, "indices"))).into_response()
}

/// GET /v2/market-summary
///
/// Returns market summary with major indices, currencies, and commodities.
pub(crate) async fn get_market_summary(
    Extension(schema): Extension<graphql::FinanceSchema>,
    Query(params): Query<MarketSummaryQuery>,
    headers: HeaderMap,
) -> impl IntoResponse {
    let format = parse_format(params.format);
    let lang = lang::resolve_lang(params.lang.as_deref(), &headers);
    let selection = build_rest_selection(params.fields.as_deref(), GQL_MARKET_SUMMARY_VALID_FIELDS);
    let region_arg = params
        .region
        .map(|r| format!("region: \"{}\", ", escape_gql_string(r.region())));
    let lang_arg = match &lang {
        Some(l) => format!("lang: \"{}\", ", escape_gql_string(l)),
        None => String::new(),
    };
    let query = format!(
        "query {{ marketSummary({}{}format: {}) {} }}",
        region_arg.unwrap_or_default(),
        lang_arg,
        format_to_gql(format),
        selection
    );

    info!(
        "Fetching market summary (region={:?}, format={}, fields={:?})",
        params.region,
        format.as_str(),
        params.fields
    );

    let data = match execute_gql_rest(&schema, &query, Variables::default()).await {
        Ok(d) => d,
        Err(resp) => return *resp,
    };
    (StatusCode::OK, Json(unwrap_field(data, "marketSummary"))).into_response()
}

/// GET /v2/trending
///
/// Returns trending tickers for a region.
pub(crate) async fn get_trending(
    Extension(schema): Extension<graphql::FinanceSchema>,
    Query(params): Query<TrendingQuery>,
) -> impl IntoResponse {
    let selection = build_rest_selection(params.fields.as_deref(), GQL_TRENDING_VALID_FIELDS);
    // Parens must be omitted entirely when there's no argument — `trending()`
    // with empty parens is invalid GraphQL syntax, not "no arguments".
    let region_arg = params
        .region
        .map(|r| format!("(region: \"{}\")", escape_gql_string(r.region())));
    let args_str = region_arg.unwrap_or_default();
    let query = format!("query {{ trending{args_str} {selection} }}");

    info!(
        "Fetching trending tickers (region={:?}, fields={:?})",
        params.region, params.fields
    );

    let data = match execute_gql_rest(&schema, &query, Variables::default()).await {
        Ok(d) => d,
        Err(resp) => return *resp,
    };
    (StatusCode::OK, Json(unwrap_field(data, "trending"))).into_response()
}

/// GET /v2/fear-and-greed
///
/// Returns the CNN Fear & Greed index from alternative.me.
pub(crate) async fn get_fear_and_greed(
    Extension(schema): Extension<graphql::FinanceSchema>,
    Query(params): Query<FearAndGreedQuery>,
) -> impl IntoResponse {
    let selection = build_rest_selection(params.fields.as_deref(), GQL_FEAR_AND_GREED_VALID_FIELDS);
    let query = format!("query {{ fearAndGreed {selection} }}");

    info!("Fetching Fear & Greed index (fields={:?})", params.fields);

    let data = match execute_gql_rest(&schema, &query, Variables::default()).await {
        Ok(d) => d,
        Err(resp) => return *resp,
    };
    (StatusCode::OK, Json(unwrap_field(data, "fearAndGreed"))).into_response()
}

/// Build the `marketCalendar { ... }` selection set, expanding `detail` with
/// its full union inline-fragment selection.
fn build_rest_market_calendar_selection(fields: Option<&str>) -> String {
    build_rest_union_selection(
        fields,
        GQL_MARKET_CALENDAR_VALID_FIELDS,
        "detail",
        &["symbol", "date"],
        MARKET_CALENDAR_DETAIL_UNION_SELECTION,
    )
}

/// GET /v2/market-calendar?kind=<str>&from=<date>&to=<date>
pub(crate) async fn get_market_calendar(
    Extension(schema): Extension<graphql::FinanceSchema>,
    Query(params): Query<MarketCalendarQuery>,
) -> impl IntoResponse {
    let selection = build_rest_market_calendar_selection(params.fields.as_deref());
    let gql_kind = params.kind.as_gql_str();
    let query = format!(
        "query {{ marketCalendar(kind: {gql_kind}, from: \"{}\", to: \"{}\") {selection} }}",
        escape_gql_string(&params.from),
        escape_gql_string(&params.to)
    );

    info!(
        "Fetching market calendar (kind={:?}, from={}, to={})",
        params.kind, params.from, params.to
    );

    let data = match execute_gql_rest(&schema, &query, Variables::default()).await {
        Ok(d) => d,
        Err(resp) => return *resp,
    };
    (StatusCode::OK, Json(unwrap_field(data, "marketCalendar"))).into_response()
}

/// GET /v2/sector-performance
///
/// Aggregate performance for every sector, provider-routed via
/// `Providers::market()` (Yahoo screener fan-out, keyless).
pub(crate) async fn get_sector_performance(
    Extension(schema): Extension<graphql::FinanceSchema>,
    Query(params): Query<SectorPerformanceQuery>,
) -> impl IntoResponse {
    let selection = build_rest_selection(
        params.fields.as_deref(),
        GQL_MARKET_SECTOR_PERFORMANCE_VALID_FIELDS,
    );
    let query = format!("query {{ sectorPerformance {selection} }}");

    info!("Fetching sector performance (fields={:?})", params.fields);

    let data = match execute_gql_rest(&schema, &query, Variables::default()).await {
        Ok(d) => d,
        Err(resp) => return *resp,
    };
    (
        StatusCode::OK,
        Json(unwrap_field(data, "sectorPerformance")),
    )
        .into_response()
}

/// GET /v2/sector-pe
///
/// Price/earnings ratios by sector, provider-routed via `Providers::market()`
/// (Yahoo screener fan-out, keyless).
pub(crate) async fn get_sector_pe(
    Extension(schema): Extension<graphql::FinanceSchema>,
    Query(params): Query<SectorPeQuery>,
) -> impl IntoResponse {
    let selection =
        build_rest_selection(params.fields.as_deref(), GQL_MARKET_SECTOR_PE_VALID_FIELDS);
    let query = format!("query {{ sectorPe {selection} }}");

    info!("Fetching sector PE (fields={:?})", params.fields);

    let data = match execute_gql_rest(&schema, &query, Variables::default()).await {
        Ok(d) => d,
        Err(resp) => return *resp,
    };
    (StatusCode::OK, Json(unwrap_field(data, "sectorPe"))).into_response()
}
