use async_graphql::{Name, Variables};
use axum::{
    extract::{Extension, Path, Query},
    http::StatusCode,
    response::{IntoResponse, Json},
};
use finance_query_server::graphql::{
    self,
    fields::{
        GQL_CONGRESSIONAL_TRADE_VALID_FIELDS, GQL_FAIL_TO_DELIVER_VALID_FIELDS,
        GQL_FILING_SECTION_VALID_FIELDS, GQL_RISK_FACTOR_VALID_FIELDS, unwrap_ticker_field,
    },
    pagination::build_connection_selection,
};
use finance_query_server::params::{FilingSectionsQuery, FilingsQuery, RiskFactorsQuery};
use tracing::info;

use super::gql_bridge::{
    build_rest_selection, connection_args, execute_gql_rest, unwrap_connection,
};

/// GET /v2/filings/{symbol}/congressional-trades
///
/// Congressional (House and Senate) trading disclosures for a symbol. FMP
/// when `FMP_API_KEY` is set; House and Senate PTR filings otherwise
/// (keyless, merged when both are compiled in).
pub(crate) async fn get_congressional_trades(
    Extension(schema): Extension<graphql::FinanceSchema>,
    Path(symbol): Path<String>,
    Query(params): Query<FilingsQuery>,
) -> impl IntoResponse {
    let inner_selection = build_rest_selection(
        params.fields.as_deref(),
        GQL_CONGRESSIONAL_TRADE_VALID_FIELDS,
    );
    let selection = build_connection_selection(&inner_selection);
    let conn_args = connection_args(params.limit, params.cursor.as_deref());
    let conn_args_str = if conn_args.is_empty() {
        String::new()
    } else {
        format!("({})", conn_args.join(", "))
    };
    let query = format!(
        "query GetCongressionalTrades($symbol: String!) {{ ticker(symbol: $symbol) {{ congressionalTrades{conn_args_str} {selection} }} }}"
    );

    info!("Fetching congressional trades for {}", symbol);

    let mut vars = Variables::default();
    vars.insert(Name::new("symbol"), symbol.clone().into());
    let data = match execute_gql_rest(&schema, &query, vars).await {
        Ok(d) => d,
        Err(resp) => return *resp,
    };
    let paginated = params.limit.is_some() || params.cursor.is_some();
    let result = unwrap_connection(unwrap_ticker_field(data, "congressionalTrades"), paginated);
    (StatusCode::OK, Json(result)).into_response()
}

/// GET /v2/filings/{symbol}/fails-to-deliver
///
/// Fails-to-deliver records for a symbol. Routes through FMP when
/// `FMP_API_KEY` is set, EDGAR otherwise (keyless).
pub(crate) async fn get_fails_to_deliver(
    Extension(schema): Extension<graphql::FinanceSchema>,
    Path(symbol): Path<String>,
    Query(params): Query<FilingsQuery>,
) -> impl IntoResponse {
    let inner_selection =
        build_rest_selection(params.fields.as_deref(), GQL_FAIL_TO_DELIVER_VALID_FIELDS);
    let selection = build_connection_selection(&inner_selection);
    let conn_args = connection_args(params.limit, params.cursor.as_deref());
    let conn_args_str = if conn_args.is_empty() {
        String::new()
    } else {
        format!("({})", conn_args.join(", "))
    };
    let query = format!(
        "query GetFailsToDeliver($symbol: String!) {{ ticker(symbol: $symbol) {{ failsToDeliver{conn_args_str} {selection} }} }}"
    );

    info!("Fetching fails-to-deliver records for {}", symbol);

    let mut vars = Variables::default();
    vars.insert(Name::new("symbol"), symbol.clone().into());
    let data = match execute_gql_rest(&schema, &query, vars).await {
        Ok(d) => d,
        Err(resp) => return *resp,
    };
    let paginated = params.limit.is_some() || params.cursor.is_some();
    let result = unwrap_connection(unwrap_ticker_field(data, "failsToDeliver"), paginated);
    (StatusCode::OK, Json(result)).into_response()
}

/// GET /v2/filings/{symbol}/sections?accessionNumber=<str>&form=<ten-k|eight-k>
///
/// Sectioned text of one filing. Routes through EDGAR (best-effort HTML
/// extraction) or Polygon when configured.
pub(crate) async fn get_filing_sections(
    Extension(schema): Extension<graphql::FinanceSchema>,
    Path(symbol): Path<String>,
    Query(params): Query<FilingSectionsQuery>,
) -> impl IntoResponse {
    let selection = build_rest_selection(params.fields.as_deref(), GQL_FILING_SECTION_VALID_FIELDS);
    let gql_form = match params.form {
        finance_query_server::params::FilingSectionFormParam::TenK => "TEN_K",
        finance_query_server::params::FilingSectionFormParam::EightK => "EIGHT_K",
    };
    let query = format!(
        "query GetFilingSections($symbol: String!, $accession: String!) {{ ticker(symbol: $symbol) {{ filingSections(accessionNumber: $accession, form: {gql_form}) {selection} }} }}"
    );

    info!(
        "Fetching filing sections for {} (accession={}, form={:?})",
        symbol, params.accession_number, params.form
    );

    let mut vars = Variables::default();
    vars.insert(Name::new("symbol"), symbol.clone().into());
    vars.insert(Name::new("accession"), params.accession_number.into());
    let data = match execute_gql_rest(&schema, &query, vars).await {
        Ok(d) => d,
        Err(resp) => return *resp,
    };
    (
        StatusCode::OK,
        Json(unwrap_ticker_field(data, "filingSections")),
    )
        .into_response()
}

/// GET /v2/filings/{symbol}/risk-factors
///
/// Risk factors extracted from this symbol's SEC filings. Routes through
/// EDGAR (best-effort HTML extraction) or Polygon when configured.
pub(crate) async fn get_risk_factors(
    Extension(schema): Extension<graphql::FinanceSchema>,
    Path(symbol): Path<String>,
    Query(params): Query<RiskFactorsQuery>,
) -> impl IntoResponse {
    let selection = build_rest_selection(params.fields.as_deref(), GQL_RISK_FACTOR_VALID_FIELDS);
    let query = format!(
        "query GetRiskFactors($symbol: String!) {{ ticker(symbol: $symbol) {{ riskFactors {selection} }} }}"
    );

    info!("Fetching risk factors for {}", symbol);

    let mut vars = Variables::default();
    vars.insert(Name::new("symbol"), symbol.clone().into());
    let data = match execute_gql_rest(&schema, &query, vars).await {
        Ok(d) => d,
        Err(resp) => return *resp,
    };
    (
        StatusCode::OK,
        Json(unwrap_ticker_field(data, "riskFactors")),
    )
        .into_response()
}
