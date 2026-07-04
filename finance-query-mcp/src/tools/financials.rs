use finance_query_server::graphql::FinanceSchema;
use finance_query_server::graphql::fields::{
    FINANCIAL_LINE_ITEM_COMPOSITE_FIELDS as SHARED_FINANCIAL_LINE_ITEM_COMPOSITE_FIELDS,
    GQL_FINANCIAL_LINE_ITEM_VALID_FIELDS,
};
use rmcp::{ErrorData as McpError, model::CallToolResult};

use crate::error::{invalid_params, ser_err};
use crate::tools::gql::{
    build_type_spec_selection, execute_query, gql_string_list_literal, parse_fields, unwrap_field,
    unwrap_ticker_field,
};
use crate::tools::helpers::{parse_frequency, parse_statement_type};

/// Valid/default fields for `GqlFinancialLineItem` (`{ metric values }`);
/// `values` (`Vec<GqlFinancialDataPoint>`, composite) needs its nested
/// sub-selection, expanded via `build_type_spec_selection`.
const FINANCIAL_LINE_ITEM_FIELDS: &[&str] = GQL_FINANCIAL_LINE_ITEM_VALID_FIELDS;
const FINANCIAL_LINE_ITEM_COMPOSITE_FIELDS: &[(&str, &str)] =
    SHARED_FINANCIAL_LINE_ITEM_COMPOSITE_FIELDS;

pub async fn get_financials(
    schema: &FinanceSchema,
    symbol: String,
    statement: String,
    frequency: Option<String>,
    metrics: Option<String>,
    fields: Option<String>,
) -> Result<CallToolResult, McpError> {
    let st = parse_statement_type(&statement).ok_or_else(|| {
        invalid_params(format!(
            "Invalid statement type: '{statement}'. Use: income, balance, cashflow"
        ))
    })?;
    let freq = parse_frequency(frequency.as_deref().unwrap_or("annual"));
    let st_str = st.as_str();
    let freq_str = freq.as_str();
    let gql_st = match st_str {
        "income" => "INCOME",
        "balance" => "BALANCE",
        "cashflow" => "CASH_FLOW",
        _ => unreachable!(),
    };
    let gql_freq = match freq_str {
        "annual" => "ANNUAL",
        "quarterly" => "QUARTERLY",
        _ => "ANNUAL",
    };
    let metric_list = parse_fields(metrics);
    let metrics_arg = match &metric_list {
        Some(list) if !list.is_empty() => {
            format!(", metrics: [{}]", gql_string_list_literal(list))
        }
        _ => String::new(),
    };
    let field_list = parse_fields(fields);
    let selection = build_type_spec_selection(
        field_list.as_deref(),
        FINANCIAL_LINE_ITEM_FIELDS,
        FINANCIAL_LINE_ITEM_FIELDS,
        FINANCIAL_LINE_ITEM_COMPOSITE_FIELDS,
    );

    let query = format!(
        "query GetFin($symbol: String!) {{ ticker(symbol: $symbol) {{ financials(statement: {gql_st}, frequency: {gql_freq}{metrics_arg}) {selection} }} }}"
    );
    let mut variables = async_graphql::Variables::default();
    variables.insert(async_graphql::Name::new("symbol"), symbol.into());
    let json = execute_query(schema, &query, variables).await?;
    let data = unwrap_ticker_field(json, "financials");
    Ok(CallToolResult::success(vec![rmcp::model::Content::text(
        serde_json::to_string(&data).map_err(ser_err)?,
    )]))
}

pub async fn get_batch_financials(
    schema: &FinanceSchema,
    symbols: String,
    statement: String,
    frequency: Option<String>,
    metrics: Option<String>,
    fields: Option<String>,
) -> Result<CallToolResult, McpError> {
    let st = parse_statement_type(&statement).ok_or_else(|| {
        invalid_params(format!(
            "Invalid statement type: '{statement}'. Use: income, balance, cashflow"
        ))
    })?;
    let freq = parse_frequency(frequency.as_deref().unwrap_or("annual"));
    let st_str = st.as_str();
    let freq_str = freq.as_str();
    let gql_st = match st_str {
        "income" => "INCOME",
        "balance" => "BALANCE",
        "cashflow" => "CASH_FLOW",
        _ => unreachable!(),
    };
    let gql_freq = match freq_str {
        "annual" => "ANNUAL",
        "quarterly" => "QUARTERLY",
        _ => "ANNUAL",
    };
    let metric_list = parse_fields(metrics);
    let metrics_arg = match &metric_list {
        Some(list) if !list.is_empty() => {
            format!(", metrics: [{}]", gql_string_list_literal(list))
        }
        _ => String::new(),
    };
    let syms: Vec<String> = symbols.split(',').map(|s| s.trim().to_string()).collect();
    let syms_literal = gql_string_list_literal(&syms);
    let field_list = parse_fields(fields);
    // "statement" (`GqlSymbolFinancials`) is a list of composite
    // `GqlFinancialLineItem` and needs its own nested sub-selection.
    let item_selection = build_type_spec_selection(
        field_list.as_deref(),
        FINANCIAL_LINE_ITEM_FIELDS,
        FINANCIAL_LINE_ITEM_FIELDS,
        FINANCIAL_LINE_ITEM_COMPOSITE_FIELDS,
    );

    let query = format!(
        "query {{ financialsBatch(symbols: [{syms_literal}], statement: {gql_st}, frequency: {gql_freq}{metrics_arg}) {{ financials {{ symbol statement {item_selection} }} errors {{ symbol message }} }} }}"
    );
    let json = execute_query(schema, &query, async_graphql::Variables::default()).await?;
    let data = unwrap_field(json, "financialsBatch");
    Ok(CallToolResult::success(vec![rmcp::model::Content::text(
        serde_json::to_string(&data).map_err(ser_err)?,
    )]))
}
