use async_graphql::{Name, Variables};
use axum::{
    extract::{Extension, Path, Query},
    http::StatusCode,
    response::{IntoResponse, Json},
};
use finance_query_server::graphql::{
    self,
    fields::{
        EDGAR_FACTS_COMPOSITE_FIELDS, GQL_EDGAR_FACTS_VALID_FIELDS, gql_string_list_literal,
        unwrap_field, unwrap_ticker_field,
    },
};
use serde::Deserialize;
use tracing::info;

use super::gql_bridge::build_rest_composite_selection;
use super::gql_bridge::build_rest_selection;
use super::gql_bridge::execute_gql_rest;

const GQL_EDGAR_CIK_VALID_FIELDS: &[&str] = &["symbol", "cik"];

const GQL_EDGAR_SEARCH_VALID_FIELDS: &[&str] = &["totalHits", "hits"];
const EDGAR_SEARCH_COMPOSITE_FIELDS: &[(&str, &str)] =
    &[("hits", "{ fileDate form adsh displayNames ciks }")];

#[derive(Deserialize)]
pub(crate) struct EdgarSearchQuery {
    /// Search query string (required)
    q: String,
    /// Comma-separated form types (e.g., "10-K,10-Q")
    forms: Option<String>,
    /// Start date in YYYY-MM-DD format
    start_date: Option<String>,
    /// End date in YYYY-MM-DD format
    end_date: Option<String>,
    /// Pagination offset (default: 0)
    from: Option<usize>,
    /// Page size (default: 100, max: 100)
    size: Option<usize>,
    /// Comma-separated list of fields to include in response
    fields: Option<String>,
}

#[derive(Deserialize)]
pub(crate) struct EdgarFieldsQuery {
    /// Comma-separated list of fields to include in response
    fields: Option<String>,
}

/// Query parameters for /v2/edgar/facts/{symbol}
#[derive(Deserialize)]
pub(crate) struct EdgarFactsQuery {
    /// Comma-separated list of fields to include in response
    fields: Option<String>,
    /// XBRL taxonomy (default: "us-gaap"); also try "ifrs-full" or "dei"
    taxonomy: Option<String>,
    /// Comma-separated XBRL concept names to filter to; omitted = curated defaults
    concepts: Option<String>,
}

/// GET /v2/edgar/cik/{symbol}
///
/// Resolve a ticker symbol to its SEC CIK number.
/// Requires EDGAR_EMAIL environment variable to be set.
pub(crate) async fn get_edgar_cik(
    Extension(schema): Extension<graphql::FinanceSchema>,
    Path(symbol): Path<String>,
    params: Query<EdgarFieldsQuery>,
) -> impl IntoResponse {
    let selection = build_rest_selection(params.fields.as_deref(), GQL_EDGAR_CIK_VALID_FIELDS);
    let query =
        format!("query GetEdgarCik($symbol: String!) {{ edgarCik(symbol: $symbol) {selection} }}");
    let mut vars = Variables::default();
    vars.insert(Name::new("symbol"), symbol.clone().into());

    info!("Resolving CIK for symbol: {}", symbol);

    let data = match execute_gql_rest(&schema, &query, vars).await {
        Ok(d) => d,
        Err(resp) => return resp,
    };
    (StatusCode::OK, Json(unwrap_field(data, "edgarCik"))).into_response()
}

/// GET /v2/edgar/submissions/{symbol}
///
/// Fetch SEC filing history and company metadata.
/// Requires EDGAR_EMAIL environment variable to be set.
pub(crate) async fn get_edgar_submissions(
    Extension(schema): Extension<graphql::FinanceSchema>,
    Path(symbol): Path<String>,
    _params: Query<EdgarFieldsQuery>,
) -> impl IntoResponse {
    info!("Fetching EDGAR submissions for symbol: {}", symbol);
    let query = "query GetEdgarSubmissions($symbol: String!) { ticker(symbol: $symbol) { edgarSubmissions { cik name tickers exchanges sic sicDescription fiscalYearEnd category filings { accessionNumber filingDate reportDate form size primaryDocument primaryDocDescription } } } }";
    let mut vars = Variables::default();
    vars.insert(Name::new("symbol"), symbol.clone().into());
    let data = match execute_gql_rest(&schema, query, vars).await {
        Ok(d) => d,
        Err(resp) => return resp,
    };
    (
        StatusCode::OK,
        Json(unwrap_ticker_field(data, "edgarSubmissions")),
    )
        .into_response()
}

/// GET /v2/edgar/facts/{symbol}
pub(crate) async fn get_edgar_facts(
    Extension(schema): Extension<graphql::FinanceSchema>,
    Path(symbol): Path<String>,
    params: Query<EdgarFactsQuery>,
) -> impl IntoResponse {
    info!("Fetching EDGAR company facts for symbol: {}", symbol);
    let selection = build_rest_composite_selection(
        params.fields.as_deref(),
        GQL_EDGAR_FACTS_VALID_FIELDS,
        EDGAR_FACTS_COMPOSITE_FIELDS,
    );
    let taxonomy_arg = match &params.taxonomy {
        Some(t) if !t.trim().is_empty() => "taxonomy: $taxonomy".to_string(),
        _ => String::new(),
    };
    let concept_list: Option<Vec<&str>> = params.concepts.as_deref().map(|c| {
        c.split(',')
            .map(|s| s.trim())
            .filter(|s| !s.is_empty())
            .collect()
    });
    let concepts_arg = match &concept_list {
        Some(list) if !list.is_empty() => {
            format!("concepts: [{}]", gql_string_list_literal(list))
        }
        _ => String::new(),
    };
    let args: Vec<&str> = [taxonomy_arg.as_str(), concepts_arg.as_str()]
        .into_iter()
        .filter(|s| !s.is_empty())
        .collect();
    let args_str = if args.is_empty() {
        String::new()
    } else {
        format!("({})", args.join(", "))
    };
    let query = format!(
        "query GetEdgarFacts($symbol: String!, $taxonomy: String) {{ ticker(symbol: $symbol) {{ edgarFacts{args_str} {selection} }} }}"
    );
    let mut vars = Variables::default();
    vars.insert(Name::new("symbol"), symbol.clone().into());
    if let Some(t) = &params.taxonomy {
        vars.insert(Name::new("taxonomy"), t.clone().into());
    }
    let data = match execute_gql_rest(&schema, &query, vars).await {
        Ok(d) => d,
        Err(resp) => return resp,
    };
    (
        StatusCode::OK,
        Json(unwrap_ticker_field(data, "edgarFacts")),
    )
        .into_response()
}

/// GET /v2/edgar/search
pub(crate) async fn get_edgar_search(
    Extension(schema): Extension<graphql::FinanceSchema>,
    Query(params): Query<EdgarSearchQuery>,
) -> impl IntoResponse {
    info!(
        "EDGAR search: q={}, from={:?}, size={:?}, fields={:?}",
        params.q, params.from, params.size, params.fields
    );
    let selection = build_rest_composite_selection(
        params.fields.as_deref(),
        GQL_EDGAR_SEARCH_VALID_FIELDS,
        EDGAR_SEARCH_COMPOSITE_FIELDS,
    );
    let query_str = format!(
        "query GetEdgarSearch($query: String!, $forms: String, $startDate: String, $endDate: String, $from: Int, $size: Int) {{ edgarSearch(query: $query, forms: $forms, startDate: $startDate, endDate: $endDate, from: $from, size: $size) {selection} }}"
    );
    let mut vars = Variables::default();
    vars.insert(Name::new("query"), params.q.clone().into());
    if let Some(f) = params.forms.as_deref().filter(|f| !f.trim().is_empty()) {
        vars.insert(Name::new("forms"), f.into());
    }
    if let Some(d) = params.start_date.as_deref().filter(|d| !d.is_empty()) {
        vars.insert(Name::new("startDate"), d.into());
    }
    if let Some(d) = params.end_date.as_deref().filter(|d| !d.is_empty()) {
        vars.insert(Name::new("endDate"), d.into());
    }
    if let Some(from) = params.from {
        vars.insert(Name::new("from"), (from as i64).into());
    }
    if let Some(size) = params.size {
        vars.insert(Name::new("size"), (size as i64).into());
    }
    let data = match execute_gql_rest(&schema, &query_str, vars).await {
        Ok(d) => d,
        Err(resp) => return resp,
    };
    (StatusCode::OK, Json(unwrap_field(data, "edgarSearch"))).into_response()
}
