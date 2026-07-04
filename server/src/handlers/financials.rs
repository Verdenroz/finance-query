use async_graphql::{Name, Variables};
use axum::{
    extract::{Extension, Path, Query},
    http::StatusCode,
    response::{IntoResponse, Json},
};
use finance_query::{Frequency, StatementType};
use finance_query_server::graphql::{
    self,
    fields::{
        FINANCIAL_LINE_ITEM_COMPOSITE_FIELDS, GQL_FINANCIAL_LINE_ITEM_VALID_FIELDS,
        gql_string_list_literal, unwrap_field, unwrap_ticker_field,
    },
};
use serde::Deserialize;
use tracing::info;

use super::gql_bridge::{build_rest_composite_selection, execute_gql_rest};

fn default_frequency() -> String {
    "annual".to_string()
}

fn parse_frequency(s: &str) -> Frequency {
    match s.to_lowercase().as_str() {
        "quarterly" | "q" => Frequency::Quarterly,
        _ => Frequency::Annual,
    }
}

fn parse_statement_type(s: &str) -> Option<StatementType> {
    match s.to_lowercase().as_str() {
        "income" => Some(StatementType::Income),
        "balance" => Some(StatementType::Balance),
        "cashflow" | "cash-flow" => Some(StatementType::CashFlow),
        _ => None,
    }
}

fn statement_to_gql(statement: StatementType) -> &'static str {
    match statement {
        StatementType::Income => "INCOME",
        StatementType::Balance => "BALANCE",
        StatementType::CashFlow => "CASH_FLOW",
    }
}

fn frequency_to_gql(frequency: Frequency) -> &'static str {
    match frequency {
        Frequency::Annual => "ANNUAL",
        Frequency::Quarterly => "QUARTERLY",
    }
}

// Build the `, metrics: [...]` argument fragment from a comma-separated list.
fn metrics_arg(metrics: Option<&str>) -> String {
    let list: Vec<&str> = metrics
        .map(|raw| {
            raw.split(',')
                .map(|m| m.trim())
                .filter(|m| !m.is_empty())
                .collect()
        })
        .unwrap_or_default();
    if list.is_empty() {
        String::new()
    } else {
        format!(", metrics: [{}]", gql_string_list_literal(&list))
    }
}

fn invalid_statement_response(statement: &str) -> axum::response::Response {
    let error = serde_json::json!({
        "error": format!("Invalid statement type: '{}'. Valid types: income, balance, cashflow", statement),
        "status": 400
    });
    (StatusCode::BAD_REQUEST, Json(error)).into_response()
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct FinancialsQuery {
    /// Frequency: annual or quarterly (default: annual)
    #[serde(default = "default_frequency")]
    frequency: String,
    /// Comma-separated list of fields to include in response
    fields: Option<String>,
    /// Comma-separated list of metric names to include in the statement (e.g. "TotalRevenue,NetIncome")
    metrics: Option<String>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct BatchFinancialsQuery {
    /// Comma-separated symbols (required)
    symbols: String,
    /// Statement type (required): income, balance, cashflow
    statement: String,
    #[serde(default = "default_frequency")]
    frequency: String,
    /// Comma-separated list of fields to include in response
    fields: Option<String>,
    /// Comma-separated list of metric names to include in the statement (e.g. "TotalRevenue,NetIncome")
    metrics: Option<String>,
}

/// GET /v2/financials/{symbol}/{statement}
///
/// Path params:
/// - `statement`: income, balance, or cashflow
///
/// Query: `frequency` (annual|quarterly, default: annual), `metrics` (comma-separated metric names to include)
pub(crate) async fn get_financials(
    Extension(schema): Extension<graphql::FinanceSchema>,
    Path((symbol, statement)): Path<(String, String)>,
    Query(params): Query<FinancialsQuery>,
) -> impl IntoResponse {
    let frequency = parse_frequency(&params.frequency);
    let statement_type = match parse_statement_type(&statement) {
        Some(st) => st,
        None => return invalid_statement_response(&statement),
    };

    let selection = build_rest_composite_selection(
        params.fields.as_deref(),
        GQL_FINANCIAL_LINE_ITEM_VALID_FIELDS,
        FINANCIAL_LINE_ITEM_COMPOSITE_FIELDS,
    );
    let query = format!(
        "query GetFin($symbol: String!) {{ ticker(symbol: $symbol) {{ financials(statement: {}, frequency: {}{}) {} }} }}",
        statement_to_gql(statement_type),
        frequency_to_gql(frequency),
        metrics_arg(params.metrics.as_deref()),
        selection
    );

    let mut vars = Variables::default();
    vars.insert(Name::new("symbol"), symbol.clone().into());

    info!(
        "Fetching {} {} financials for {} (fields={:?}, metrics={:?})",
        params.frequency, statement, symbol, params.fields, params.metrics
    );

    let data = match execute_gql_rest(&schema, &query, vars).await {
        Ok(d) => d,
        Err(resp) => return resp,
    };
    (
        StatusCode::OK,
        Json(unwrap_ticker_field(data, "financials")),
    )
        .into_response()
}

/// GET /v2/financials?symbols=<csv>&statement=<str>&frequency=<str>&metrics=<csv>
pub(crate) async fn get_batch_financials(
    Extension(schema): Extension<graphql::FinanceSchema>,
    Query(params): Query<BatchFinancialsQuery>,
) -> impl IntoResponse {
    let statement_type = match parse_statement_type(&params.statement) {
        Some(st) => st,
        None => return invalid_statement_response(&params.statement),
    };
    let frequency = parse_frequency(&params.frequency);

    let symbols: Vec<&str> = params.symbols.split(',').map(|s| s.trim()).collect();
    let syms_literal = gql_string_list_literal(&symbols);
    let item_selection = build_rest_composite_selection(
        params.fields.as_deref(),
        GQL_FINANCIAL_LINE_ITEM_VALID_FIELDS,
        FINANCIAL_LINE_ITEM_COMPOSITE_FIELDS,
    );

    let query = format!(
        "query {{ financialsBatch(symbols: [{}], statement: {}, frequency: {}{}) {{ financials {{ symbol statement {} }} errors {{ symbol message }} }} }}",
        syms_literal,
        statement_to_gql(statement_type),
        frequency_to_gql(frequency),
        metrics_arg(params.metrics.as_deref()),
        item_selection
    );

    info!(
        "Fetching batch financials for {} symbols (statement={}, frequency={}, metrics={:?})",
        symbols.len(),
        params.statement,
        params.frequency,
        params.metrics
    );

    let data = match execute_gql_rest(&schema, &query, Variables::default()).await {
        Ok(d) => d,
        Err(resp) => return resp,
    };
    (StatusCode::OK, Json(unwrap_field(data, "financialsBatch"))).into_response()
}
