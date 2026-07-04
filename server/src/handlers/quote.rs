use async_graphql::{Name, Variables};
use axum::{
    extract::{Extension, Path, Query},
    http::HeaderMap,
    response::{IntoResponse, Json},
};
use finance_query::ValueFormat;
use serde::Deserialize;
use tracing::info;

use axum::http::StatusCode;
use finance_query_server::graphql::{
    self,
    fields::{GQL_QUOTE_VALID_FIELDS, unwrap_field, unwrap_ticker_field},
};
use finance_query_server::lang;

use super::gql_bridge::{build_rest_selection, execute_gql_rest};
use super::support::parse_format;

#[derive(Deserialize)]
pub(crate) struct QuoteQuery {
    /// Whether to include company logo URL (default: false)
    #[serde(default)]
    logo: bool,
    /// Value format: raw, pretty, or both (default: raw)
    format: Option<String>,
    /// Comma-separated list of fields to include in response
    fields: Option<String>,
    /// Target language for translated text fields (BCP 47, e.g. "ja", "zh-Hant");
    /// falls back to the Accept-Language header
    lang: Option<String>,
}

#[derive(Deserialize)]
pub(crate) struct QuotesQuery {
    symbols: String, // Comma-separated symbols
    /// Whether to include company logo URLs (default: false)
    #[serde(default)]
    logo: bool,
    /// Value format: raw, pretty, or both (default: raw)
    format: Option<String>,
    /// Comma-separated list of fields to include in response
    fields: Option<String>,
    /// Target language for translated text fields (BCP 47, e.g. "ja", "zh-Hant");
    /// falls back to the Accept-Language header
    lang: Option<String>,
}

/// GET /v2/quote/{symbol}
///
/// Query: `logo` (bool, default: false), `format` (raw|pretty|both), `fields` (comma-separated)
pub(crate) async fn get_quote(
    Extension(schema): Extension<graphql::FinanceSchema>,
    Path(symbol): Path<String>,
    Query(params): Query<QuoteQuery>,
    headers: HeaderMap,
) -> impl IntoResponse {
    let format = parse_format(params.format.as_deref());
    let lang = lang::resolve_lang(params.lang.as_deref(), &headers);

    // Map REST format to GraphQL enum value string.
    let gql_format = match format {
        ValueFormat::Raw => "RAW",
        ValueFormat::Pretty => "PRETTY",
        ValueFormat::Both => "BOTH",
    };

    let selection = build_rest_selection(params.fields.as_deref(), GQL_QUOTE_VALID_FIELDS);

    let lang_arg = match &lang {
        Some(l) => format!(", lang: \"{}\"", l),
        None => String::new(),
    };

    let query = format!(
        "query GetQuote($symbol: String!, $logo: Boolean) {{ ticker(symbol: $symbol) {{ quote(logo: $logo, format: {}{}) {} }} }}",
        gql_format, lang_arg, selection
    );

    let mut variables = Variables::default();
    variables.insert(Name::new("symbol"), symbol.clone().into());
    variables.insert(Name::new("logo"), params.logo.into());

    info!(
        "Received quote request for symbol: {} (logo={}, format={}, fields={:?})",
        symbol,
        params.logo,
        format.as_str(),
        params.fields
    );

    let data = match execute_gql_rest(&schema, &query, variables).await {
        Ok(d) => d,
        Err(resp) => return resp,
    };

    (StatusCode::OK, Json(unwrap_ticker_field(data, "quote"))).into_response()
}

/// GET /v2/quotes
///
/// Query: `symbols` (comma-separated, required), `logo` (bool, default: false),
///        `format` (raw|pretty|both), `fields` (comma-separated)
///
/// Uses batch fetching via Tickers for optimal performance (single API call).
pub(crate) async fn get_quotes(
    Extension(schema): Extension<graphql::FinanceSchema>,
    Query(params): Query<QuotesQuery>,
    headers: HeaderMap,
) -> impl IntoResponse {
    let format = parse_format(params.format.as_deref());
    let lang = lang::resolve_lang(params.lang.as_deref(), &headers);

    let gql_format = match format {
        ValueFormat::Raw => "RAW",
        ValueFormat::Pretty => "PRETTY",
        ValueFormat::Both => "BOTH",
    };
    let lang_arg = match &lang {
        Some(l) => format!(", lang: \"{}\"", l),
        None => String::new(),
    };
    let logo_arg = if params.logo { ", logo: true" } else { "" };
    let selection = build_rest_selection(params.fields.as_deref(), GQL_QUOTE_VALID_FIELDS);

    let syms: Vec<&str> = params.symbols.split(',').map(|s| s.trim()).collect();
    let syms_literal: String = syms
        .iter()
        .map(|s| format!("\"{}\"", s))
        .collect::<Vec<_>>()
        .join(", ");

    let query = format!(
        "query {{ quotes(symbols: [{}], format: {}{}{}) {{ quotes {} errors {{ symbol message }} }} }}",
        syms_literal, gql_format, logo_arg, lang_arg, selection
    );

    info!(
        "Fetching batch quotes for {} symbols (logo={}, format={}, fields={:?})",
        syms.len(),
        params.logo,
        format.as_str(),
        params.fields
    );

    let data = match execute_gql_rest(&schema, &query, Variables::default()).await {
        Ok(d) => d,
        Err(resp) => return resp,
    };

    (StatusCode::OK, Json(unwrap_field(data, "quotes"))).into_response()
}
