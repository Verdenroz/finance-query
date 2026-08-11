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
use finance_query_server::params::{BatchFinancialsQuery, FinancialsQuery};
use tracing::info;

use super::gql_bridge::{build_rest_composite_selection, execute_gql_rest};

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

/// GET /v2/financials/{symbol}/{statement}
///
/// Path params:
/// - `statement`: income, balance, or cashflow
///
/// Query: `frequency` (annual|quarterly, default: annual), `metrics` (comma-separated metric names to include)
pub(crate) async fn get_financials(
    Extension(schema): Extension<graphql::FinanceSchema>,
    Path((symbol, statement)): Path<(String, StatementType)>,
    Query(params): Query<FinancialsQuery>,
) -> impl IntoResponse {
    let selection = build_rest_composite_selection(
        params.fields.as_deref(),
        GQL_FINANCIAL_LINE_ITEM_VALID_FIELDS,
        FINANCIAL_LINE_ITEM_COMPOSITE_FIELDS,
    );
    let query = format!(
        "query GetFin($symbol: String!) {{ ticker(symbol: $symbol) {{ financials(statement: {}, frequency: {}{}) {} }} }}",
        statement_to_gql(statement),
        frequency_to_gql(params.frequency),
        metrics_arg(params.metrics.as_deref()),
        selection
    );

    let mut vars = Variables::default();
    vars.insert(Name::new("symbol"), symbol.clone().into());

    info!(
        "Fetching financials for {} (fields={:?}, metrics={:?})",
        symbol, params.fields, params.metrics
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
        statement_to_gql(params.statement),
        frequency_to_gql(params.frequency),
        metrics_arg(params.metrics.as_deref()),
        item_selection
    );

    info!(
        "Fetching batch financials for {} symbols (metrics={:?})",
        symbols.len(),
        params.metrics
    );

    let data = match execute_gql_rest(&schema, &query, Variables::default()).await {
        Ok(d) => d,
        Err(resp) => return resp,
    };
    (StatusCode::OK, Json(unwrap_field(data, "financialsBatch"))).into_response()
}
