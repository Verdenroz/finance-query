use async_graphql::{Name, Variables};
use axum::{
    extract::{Extension, Path, Query},
    http::{HeaderMap, StatusCode},
    response::{IntoResponse, Json},
};
use finance_query_server::graphql::{
    self,
    fields::{
        GQL_TRANSCRIPT_VALID_FIELDS, TRANSCRIPT_COMPOSITE_FIELDS, escape_gql_string,
        unwrap_ticker_field,
    },
};
use finance_query_server::lang;
use serde::Deserialize;
use tracing::info;

use super::gql_bridge::{build_rest_composite_selection, execute_gql_rest};

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct EarningsTranscriptQuery {
    /// Fiscal quarter (Q1, Q2, Q3, Q4). If not provided, returns latest.
    quarter: Option<String>,
    /// Fiscal year. If not provided with quarter, returns latest.
    year: Option<i32>,
    /// Target language for translated text fields (BCP 47, e.g. "ja", "zh-Hant");
    /// falls back to the Accept-Language header
    lang: Option<String>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct EarningsTranscriptsQuery {
    /// Maximum number of transcripts to return. If not provided, returns all.
    limit: Option<usize>,
    /// Target language for translated text fields (BCP 47, e.g. "ja", "zh-Hant");
    /// falls back to the Accept-Language header
    lang: Option<String>,
}

/// GET /v2/transcripts/{symbol}
///
/// Returns earnings transcript for a symbol.
/// Query params:
/// - `quarter` (optional): Fiscal quarter (Q1, Q2, Q3, Q4). Defaults to latest.
/// - `year` (optional): Fiscal year. Defaults to latest.
pub(crate) async fn get_transcript(
    Extension(schema): Extension<graphql::FinanceSchema>,
    Path(symbol): Path<String>,
    Query(params): Query<EarningsTranscriptQuery>,
    headers: HeaderMap,
) -> impl IntoResponse {
    let lang = lang::resolve_lang(params.lang.as_deref(), &headers);
    let quarter_arg = params
        .quarter
        .as_deref()
        .map(|q| format!("quarter: \"{}\"", escape_gql_string(q)));
    let year_arg = params.year.map(|y| format!("year: {y}"));
    let lang_arg = lang.as_ref().map(|l| format!("lang: \"{}\"", l));
    let args: Vec<String> = [quarter_arg, year_arg, lang_arg]
        .into_iter()
        .flatten()
        .collect();
    let args_str = if args.is_empty() {
        String::new()
    } else {
        format!("({})", args.join(", "))
    };
    let selection = build_rest_composite_selection(
        None,
        GQL_TRANSCRIPT_VALID_FIELDS,
        TRANSCRIPT_COMPOSITE_FIELDS,
    );
    let query = format!(
        "query GetTranscript($symbol: String!) {{ ticker(symbol: $symbol) {{ transcript{args_str} {selection} }} }}"
    );
    info!(
        "Fetching transcript for {} (quarter={:?}, year={:?})",
        symbol, params.quarter, params.year
    );

    let mut vars = Variables::default();
    vars.insert(Name::new("symbol"), symbol.clone().into());
    let data = match execute_gql_rest(&schema, &query, vars).await {
        Ok(d) => d,
        Err(resp) => return resp,
    };
    (
        StatusCode::OK,
        Json(unwrap_ticker_field(data, "transcript")),
    )
        .into_response()
}

/// GET /v2/transcripts/{symbol}/all
///
/// Returns all earnings transcripts for a symbol.
/// Query params:
/// - `limit` (optional): Maximum number of transcripts to return.
pub(crate) async fn get_transcripts(
    Extension(schema): Extension<graphql::FinanceSchema>,
    Path(symbol): Path<String>,
    Query(params): Query<EarningsTranscriptsQuery>,
    headers: HeaderMap,
) -> impl IntoResponse {
    let lang = lang::resolve_lang(params.lang.as_deref(), &headers);
    let limit_arg = params.limit.map(|l| format!("limit: {l}"));
    let lang_arg = lang.as_ref().map(|l| format!("lang: \"{}\"", l));
    let args: Vec<String> = [limit_arg, lang_arg].into_iter().flatten().collect();
    let args_str = if args.is_empty() {
        String::new()
    } else {
        format!("({})", args.join(", "))
    };
    let selection = build_rest_composite_selection(
        None,
        GQL_TRANSCRIPT_VALID_FIELDS,
        TRANSCRIPT_COMPOSITE_FIELDS,
    );
    let query = format!(
        "query GetTranscripts($symbol: String!) {{ ticker(symbol: $symbol) {{ transcripts{args_str} {selection} }} }}"
    );
    info!(
        "Fetching all transcripts for {} (limit={:?})",
        symbol, params.limit
    );
    let mut vars = Variables::default();
    vars.insert(Name::new("symbol"), symbol.clone().into());
    let data = match execute_gql_rest(&schema, &query, vars).await {
        Ok(d) => d,
        Err(resp) => return resp,
    };
    (
        StatusCode::OK,
        Json(unwrap_ticker_field(data, "transcripts")),
    )
        .into_response()
}
