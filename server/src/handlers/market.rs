use async_graphql::Variables;
use axum::{
    extract::{Extension, Query},
    http::{HeaderMap, StatusCode},
    response::{IntoResponse, Json},
};
use finance_query::ValueFormat;
use finance_query_server::graphql::{
    self,
    fields::{
        GQL_FEAR_AND_GREED_VALID_FIELDS, GQL_MARKET_SUMMARY_VALID_FIELDS, GQL_QUOTE_VALID_FIELDS,
        GQL_TRENDING_VALID_FIELDS, escape_gql_string, unwrap_field,
    },
};
use finance_query_server::lang;
use serde::Deserialize;
use tracing::info;

use super::gql_bridge::{build_rest_selection, execute_gql_rest};
use super::support::parse_format;

/// Map a REST `region` string (world-indices region, e.g. "americas",
/// "asia-pacific") to a `GqlIndicesRegion` enum literal. Returns `None` for
/// unrecognized input, matching `IndicesRegion::parse`'s permissive-but-strict
/// behavior (invalid region == no filter, not an error).
fn indices_region_to_gql(s: &str) -> Option<&'static str> {
    match s.to_lowercase().replace(['-', '_'], "").as_str() {
        "americas" | "america" => Some("AMERICAS"),
        "europe" | "eu" => Some("EUROPE"),
        "asiapacific" | "asia" | "apac" => Some("ASIA_PACIFIC"),
        "middleeastafrica" | "mea" | "emea" => Some("MIDDLE_EAST_AFRICA"),
        "currencies" | "currency" | "fx" => Some("CURRENCIES"),
        _ => None,
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

/// Query parameters for /v2/indices
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct IndicesQuery {
    /// Region filter: americas, europe, asia-pacific, middle-east-africa, currencies
    region: Option<String>,
    /// Value format: raw, pretty, or both (default: raw)
    format: Option<String>,
    /// Comma-separated list of fields to include in response
    fields: Option<String>,
}

/// Query parameters for /v2/market-summary
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct MarketSummaryQuery {
    /// Region code for localization (e.g., "US", "JP", "GB")
    region: Option<String>,
    /// Value format: raw, pretty, or both (default: raw)
    format: Option<String>,
    /// Comma-separated list of fields to include in response
    fields: Option<String>,
    /// Target language for translated text fields (BCP 47, e.g. "ja", "zh-Hant");
    /// falls back to the Accept-Language header
    lang: Option<String>,
}

/// Query parameters for /v2/trending
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct TrendingQuery {
    /// Region code for localization (e.g., "US", "JP", "GB")
    region: Option<String>,
    /// Comma-separated list of fields to include in response
    fields: Option<String>,
}

/// GET /v2/indices
///
/// Returns quotes for world market indices, optionally filtered by region.
pub(crate) async fn get_indices(
    Extension(schema): Extension<graphql::FinanceSchema>,
    Query(params): Query<IndicesQuery>,
) -> impl IntoResponse {
    let format = parse_format(params.format.as_deref());
    let selection = build_rest_selection(params.fields.as_deref(), GQL_QUOTE_VALID_FIELDS);
    let region_arg = params
        .region
        .as_deref()
        .and_then(indices_region_to_gql)
        .map(|r| format!("region: {r}, "));
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
        Err(resp) => return resp,
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
    let format = parse_format(params.format.as_deref());
    let lang = lang::resolve_lang(params.lang.as_deref(), &headers);
    let selection = build_rest_selection(params.fields.as_deref(), GQL_MARKET_SUMMARY_VALID_FIELDS);
    let region_arg = params
        .region
        .as_deref()
        .filter(|r| !r.is_empty())
        .map(|r| format!("region: \"{}\", ", escape_gql_string(r)));
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
        Err(resp) => return resp,
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
        .as_deref()
        .filter(|r| !r.is_empty())
        .map(|r| format!("(region: \"{}\")", escape_gql_string(r)));
    let args_str = region_arg.unwrap_or_default();
    let query = format!("query {{ trending{args_str} {selection} }}");

    info!(
        "Fetching trending tickers (region={:?}, fields={:?})",
        params.region, params.fields
    );

    let data = match execute_gql_rest(&schema, &query, Variables::default()).await {
        Ok(d) => d,
        Err(resp) => return resp,
    };
    (StatusCode::OK, Json(unwrap_field(data, "trending"))).into_response()
}

/// Query parameters for /v2/fear-and-greed
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct FearAndGreedQuery {
    /// Comma-separated list of fields to include in response
    fields: Option<String>,
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
        Err(resp) => return resp,
    };
    (StatusCode::OK, Json(unwrap_field(data, "fearAndGreed"))).into_response()
}
