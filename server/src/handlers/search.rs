use async_graphql::Variables;
use axum::{
    extract::{Extension, Query},
    http::{HeaderMap, StatusCode},
    response::{IntoResponse, Json},
};
use finance_query_server::graphql::{
    self,
    fields::{
        GQL_LOOKUP_RESULTS_VALID_FIELDS, GQL_SEARCH_RESULTS_VALID_FIELDS,
        LOOKUP_RESULTS_COMPOSITE_FIELDS, SEARCH_RESULTS_COMPOSITE_FIELDS, escape_gql_string,
        unwrap_field,
    },
};
use finance_query_server::lang;
use serde::Deserialize;
use tracing::info;

use super::gql_bridge::{build_rest_composite_selection, execute_gql_rest};

fn lookup_type_to_gql(s: &str) -> &'static str {
    match s.to_lowercase().as_str() {
        "equity" => "EQUITY",
        "mutualfund" => "MUTUAL_FUND",
        "etf" => "ETF",
        "index" => "INDEX",
        "future" => "FUTURE",
        "currency" => "CURRENCY",
        "cryptocurrency" => "CRYPTOCURRENCY",
        _ => "ALL",
    }
}

fn default_hits() -> u32 {
    std::env::var("SEARCH_HITS")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(10)
}

fn default_logo() -> bool {
    true
}

fn default_lookup_type() -> String {
    "all".to_string()
}

fn default_lookup_count() -> u32 {
    25
}

#[derive(Deserialize)]
pub(crate) struct SearchQuery {
    /// Search query string (required)
    q: String,
    /// Maximum number of quote results (default: 6)
    #[serde(default = "default_hits")]
    quotes: u32,
    /// Maximum number of news results (default: 0 = disabled)
    #[serde(default)]
    news: u32,
    /// Enable fuzzy matching for typos (default: false)
    #[serde(default)]
    fuzzy: bool,
    /// Enable logo URLs in results (default: true)
    #[serde(default = "default_logo")]
    logo: bool,
    /// Enable research reports (default: false)
    #[serde(default)]
    research: bool,
    /// Enable cultural assets/NFT indices (default: false)
    #[serde(default)]
    cultural: bool,
    /// Region code for lang/region settings (e.g., "US", "JP", "GB")
    region: Option<String>,
    /// Comma-separated list of fields to include in response
    fields: Option<String>,
    /// Target language for translated text fields (BCP 47, e.g. "ja", "zh-Hant");
    /// falls back to the Accept-Language header
    lang: Option<String>,
}

#[derive(Deserialize)]
pub(crate) struct LookupQuery {
    /// Lookup query string (required)
    q: String,
    /// Asset type filter: all, equity, mutualfund, etf, index, future, currency, cryptocurrency
    #[serde(default = "default_lookup_type")]
    #[serde(rename = "type")]
    lookup_type: String,
    /// Maximum number of results (default: 25)
    #[serde(default = "default_lookup_count")]
    count: u32,
    /// Include logo URLs (requires additional API call, default: false)
    #[serde(default)]
    logo: bool,
    /// Region code for lang/region settings (e.g., "US", "JP", "GB")
    region: Option<String>,
    /// Comma-separated list of fields to include in response
    fields: Option<String>,
    /// Target language for translated text fields (BCP 47, e.g. "ja", "zh-Hant");
    /// falls back to the Accept-Language header
    lang: Option<String>,
}

/// GET /v2/search
///
/// Search for quotes, news, and research reports
///
/// Query parameters:
/// - `q` (string, required): Search query
/// - `quotes` (u32, default: 6): Maximum quote results
/// - `news` (u32, default: 0): Maximum news results
/// - `fuzzy` (bool, default: false): Enable fuzzy matching for typos
/// - `logo` (bool, default: true): Include logo URLs
/// - `research` (bool, default: false): Include research reports
/// - `cultural` (bool, default: false): Include cultural assets (NFT indices)
/// - `region` (string, optional): Region code for lang/localization (e.g., "US", "JP")
pub(crate) async fn search(
    Extension(schema): Extension<graphql::FinanceSchema>,
    Query(params): Query<SearchQuery>,
    headers: HeaderMap,
) -> impl IntoResponse {
    let lang = lang::resolve_lang(params.lang.as_deref(), &headers);
    let selection = build_rest_composite_selection(
        params.fields.as_deref(),
        GQL_SEARCH_RESULTS_VALID_FIELDS,
        SEARCH_RESULTS_COMPOSITE_FIELDS,
    );
    let region_arg = params
        .region
        .as_deref()
        .filter(|r| !r.is_empty())
        .map(|r| format!(", region: \"{}\"", escape_gql_string(r)));
    let lang_arg = match &lang {
        Some(l) => format!(", lang: \"{}\"", escape_gql_string(l)),
        None => String::new(),
    };
    let query = format!(
        "query {{ search(query: \"{}\", quotes: {}, news: {}, fuzzy: {}, logo: {}, research: {}, cultural: {}{}{}) {} }}",
        escape_gql_string(&params.q),
        params.quotes,
        params.news,
        params.fuzzy,
        params.logo,
        params.research,
        params.cultural,
        region_arg.unwrap_or_default(),
        lang_arg,
        selection
    );

    info!(
        "Searching for: {} (quotes={}, news={}, logo={}, research={}, cultural={}, region={:?})",
        params.q,
        params.quotes,
        params.news,
        params.logo,
        params.research,
        params.cultural,
        params.region
    );

    let data = match execute_gql_rest(&schema, &query, Variables::default()).await {
        Ok(d) => d,
        Err(resp) => return resp,
    };
    (StatusCode::OK, Json(unwrap_field(data, "search"))).into_response()
}

/// GET /v2/lookup
///
/// Type-filtered symbol lookup. Unlike search, lookup specializes in discovering tickers
/// filtered by asset type (equity, ETF, mutual fund, index, future, currency, cryptocurrency).
///
/// Query parameters:
/// - `q` (string, required): Lookup query
/// - `type` (string, default: "all"): Asset type filter
/// - `count` (u32, default: 25): Maximum results
/// - `logo` (bool, default: false): Include logo URLs (requires extra API call)
/// - `region` (string, optional): Region code for lang/localization (e.g., "US", "JP")
pub(crate) async fn lookup(
    Extension(schema): Extension<graphql::FinanceSchema>,
    Query(params): Query<LookupQuery>,
    headers: HeaderMap,
) -> impl IntoResponse {
    let lang = lang::resolve_lang(params.lang.as_deref(), &headers);
    let selection = build_rest_composite_selection(
        params.fields.as_deref(),
        GQL_LOOKUP_RESULTS_VALID_FIELDS,
        LOOKUP_RESULTS_COMPOSITE_FIELDS,
    );
    let region_arg = params
        .region
        .as_deref()
        .filter(|r| !r.is_empty())
        .map(|r| format!(", region: \"{}\"", escape_gql_string(r)));
    let lang_arg = match &lang {
        Some(l) => format!(", lang: \"{}\"", escape_gql_string(l)),
        None => String::new(),
    };
    let query = format!(
        "query {{ lookup(query: \"{}\", type: {}, count: {}, logo: {}{}{}) {} }}",
        escape_gql_string(&params.q),
        lookup_type_to_gql(&params.lookup_type),
        params.count,
        params.logo,
        region_arg.unwrap_or_default(),
        lang_arg,
        selection
    );

    info!(
        "Looking up: {} (type={}, count={}, logo={}, region={:?})",
        params.q, params.lookup_type, params.count, params.logo, params.region
    );

    let data = match execute_gql_rest(&schema, &query, Variables::default()).await {
        Ok(d) => d,
        Err(resp) => return resp,
    };
    (StatusCode::OK, Json(unwrap_field(data, "lookup"))).into_response()
}
