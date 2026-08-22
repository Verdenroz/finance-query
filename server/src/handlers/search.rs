use async_graphql::Variables;
use axum::{
    extract::{Extension, Query},
    http::{HeaderMap, StatusCode},
    response::{IntoResponse, Json},
};
use finance_query::LookupType;
use finance_query_server::graphql::{
    self,
    fields::{
        GQL_LOOKUP_RESULTS_VALID_FIELDS, GQL_SEARCH_RESULTS_VALID_FIELDS,
        LOOKUP_RESULTS_COMPOSITE_FIELDS, SEARCH_RESULTS_COMPOSITE_FIELDS, escape_gql_string,
        unwrap_field,
    },
    pagination::{build_paginated_composite_selection, unwrap_nested_connection},
};
use finance_query_server::lang;
use finance_query_server::params::{LookupQuery, SearchQuery};
use tracing::info;

use super::gql_bridge::{build_rest_composite_selection, execute_gql_rest};

fn lookup_type_to_gql(lookup_type: LookupType) -> &'static str {
    match lookup_type {
        LookupType::All => "ALL",
        LookupType::Equity => "EQUITY",
        LookupType::MutualFund => "MUTUAL_FUND",
        LookupType::Etf => "ETF",
        LookupType::Index => "INDEX",
        LookupType::Future => "FUTURE",
        LookupType::Currency => "CURRENCY",
        LookupType::Cryptocurrency => "CRYPTOCURRENCY",
    }
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
    let quotes_item_selection = SEARCH_RESULTS_COMPOSITE_FIELDS
        .iter()
        .find(|(name, _)| *name == "quotes")
        .map(|(_, sel)| *sel)
        .unwrap_or("{ symbol }");
    let selection = build_paginated_composite_selection(
        params.fields.as_deref(),
        GQL_SEARCH_RESULTS_VALID_FIELDS,
        GQL_SEARCH_RESULTS_VALID_FIELDS,
        SEARCH_RESULTS_COMPOSITE_FIELDS,
        "quotes",
        quotes_item_selection,
        params.limit,
        params.cursor.as_deref(),
    );
    let region_arg = params
        .region
        .map(|r| format!(", region: \"{}\"", escape_gql_string(r.region())));
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
        Err(resp) => return *resp,
    };
    let paginated = params.limit.is_some() || params.cursor.is_some();
    let result = unwrap_nested_connection(unwrap_field(data, "search"), "quotes", paginated);
    (StatusCode::OK, Json(result)).into_response()
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
        .map(|r| format!(", region: \"{}\"", escape_gql_string(r.region())));
    let lang_arg = match &lang {
        Some(l) => format!(", lang: \"{}\"", escape_gql_string(l)),
        None => String::new(),
    };
    let query = format!(
        "query {{ lookup(query: \"{}\", type: {}, count: {}, logo: {}{}{}) {} }}",
        escape_gql_string(&params.q),
        lookup_type_to_gql(params.lookup_type),
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
        Err(resp) => return *resp,
    };
    (StatusCode::OK, Json(unwrap_field(data, "lookup"))).into_response()
}
