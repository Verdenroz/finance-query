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
use crate::tools::helpers::{
    frequency_to_gql, parse_frequency, parse_statement_type, statement_to_gql,
};

/// Valid/default fields for `GqlFinancialLineItem` (`{ metric values }`);
/// `values` (`Vec<GqlFinancialDataPoint>`, composite) needs its nested
/// sub-selection, expanded via `build_type_spec_selection`.
const FINANCIAL_LINE_ITEM_FIELDS: &[&str] = GQL_FINANCIAL_LINE_ITEM_VALID_FIELDS;
const FINANCIAL_LINE_ITEM_COMPOSITE_FIELDS: &[(&str, &str)] =
    SHARED_FINANCIAL_LINE_ITEM_COMPOSITE_FIELDS;

/// Split a comma-separated `symbols` param into trimmed, non-empty symbols.
fn split_symbols(symbols: &str) -> Vec<String> {
    symbols
        .split(',')
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect()
}

/// Build the `, metrics: [...]` argument suffix, or empty when no metrics
/// were requested — mirrors the leading comma needed because it's always
/// spliced after an already-present `statement`/`frequency` argument.
fn build_metrics_arg(metric_list: Option<&[String]>) -> String {
    match metric_list {
        Some(list) if !list.is_empty() => {
            format!(", metrics: [{}]", gql_string_list_literal(list))
        }
        _ => String::new(),
    }
}

/// Accepts one or more comma-separated symbols: a single symbol returns the
/// flat statement shape, multiple symbols return the batch `{financials, errors}` shape.
pub async fn get_financials(
    schema: &FinanceSchema,
    symbols: String,
    statement: String,
    frequency: Option<String>,
    metrics: Option<String>,
    fields: Option<String>,
) -> Result<CallToolResult, McpError> {
    let syms = split_symbols(&symbols);
    if syms.len() == 1 {
        get_one_financials(
            schema,
            syms.into_iter().next().unwrap(),
            statement,
            frequency,
            metrics,
            fields,
        )
        .await
    } else {
        get_many_financials(schema, syms, statement, frequency, metrics, fields).await
    }
}

async fn get_one_financials(
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
    let freq_input = frequency.as_deref().unwrap_or("annual");
    let freq = parse_frequency(freq_input).ok_or_else(|| {
        invalid_params(format!(
            "Invalid frequency: '{freq_input}'. Use: annual, quarterly"
        ))
    })?;
    let gql_st = statement_to_gql(st);
    let gql_freq = frequency_to_gql(freq);
    let metric_list = parse_fields(metrics);
    let metrics_arg = build_metrics_arg(metric_list.as_deref());
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

async fn get_many_financials(
    schema: &FinanceSchema,
    syms: Vec<String>,
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
    let freq_input = frequency.as_deref().unwrap_or("annual");
    let freq = parse_frequency(freq_input).ok_or_else(|| {
        invalid_params(format!(
            "Invalid frequency: '{freq_input}'. Use: annual, quarterly"
        ))
    })?;
    let gql_st = statement_to_gql(st);
    let gql_freq = frequency_to_gql(freq);
    let metric_list = parse_fields(metrics);
    let metrics_arg = build_metrics_arg(metric_list.as_deref());
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn split_symbols_returns_single_symbol_for_one_input() {
        assert_eq!(split_symbols("AAPL"), vec!["AAPL".to_string()]);
    }

    #[test]
    fn split_symbols_trims_whitespace_around_each_symbol() {
        assert_eq!(
            split_symbols("AAPL, MSFT , GOOG"),
            vec!["AAPL".to_string(), "MSFT".to_string(), "GOOG".to_string()]
        );
    }

    #[test]
    fn split_symbols_drops_empty_entries_from_repeated_commas() {
        assert_eq!(
            split_symbols("AAPL,,MSFT,"),
            vec!["AAPL".to_string(), "MSFT".to_string()]
        );
    }

    #[test]
    fn split_symbols_returns_empty_vec_for_empty_string() {
        assert_eq!(split_symbols(""), Vec::<String>::new());
    }

    #[test]
    fn split_symbols_returns_empty_vec_for_blank_input() {
        assert_eq!(split_symbols("   "), Vec::<String>::new());
    }

    #[test]
    fn build_metrics_arg_formats_a_gql_list_when_metrics_present() {
        let metrics = vec!["revenue".to_string(), "netIncome".to_string()];
        assert_eq!(
            build_metrics_arg(Some(&metrics)),
            ", metrics: [\"revenue\", \"netIncome\"]"
        );
    }

    #[test]
    fn build_metrics_arg_is_empty_when_metrics_is_none() {
        assert_eq!(build_metrics_arg(None), "");
    }

    #[test]
    fn build_metrics_arg_is_empty_when_metrics_list_is_empty() {
        let empty: Vec<String> = Vec::new();
        assert_eq!(build_metrics_arg(Some(&empty)), "");
    }
}
