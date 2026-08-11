use finance_query::{Interval, TimeRange};
use finance_query_server::graphql::FinanceSchema;
use rmcp::{ErrorData as McpError, model::CallToolResult};

use crate::error::ser_err;
use crate::tools::gql::{
    DEFAULT_MCP_PAGE_SIZE, GQL_INDICATORS_DEFAULT_FIELDS, GQL_INDICATORS_VALID_FIELDS,
    INDICATOR_COMPOSITE_FIELDS, build_connection_selection, build_type_spec_selection,
    escape_gql_string, execute_query, gql_string_list_literal, parse_fields, unwrap_field,
    unwrap_ticker_field, wrap_nested_connection,
};
use crate::tools::helpers::{interval_to_gql, range_to_gql};

/// Split a comma-separated `symbols` param into trimmed, non-empty symbols.
fn split_symbols(symbols: &str) -> Vec<String> {
    symbols
        .split(',')
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect()
}

/// Whether the `indicators` composite sub-field should be expanded for the
/// batch item selection: true when the caller didn't restrict `fields` at
/// all, or explicitly asked for `indicators`.
fn wants_indicators_field(field_list: Option<&[String]>) -> bool {
    field_list
        .map(|fs| fs.iter().any(|f| f == "indicators"))
        .unwrap_or(true)
}

/// Build the `{ symbol [indicators { ... }] }` item selection for
/// `indicatorsBatch`, omitting the nested `indicators` sub-selection
/// entirely when the caller's `fields` excludes it.
fn build_indicators_batch_item_selection(field_list: Option<&[String]>) -> String {
    let mut item_selection = String::from("{ symbol ");
    if wants_indicators_field(field_list) {
        item_selection.push_str("indicators ");
        item_selection.push_str(&build_type_spec_selection(
            field_list,
            GQL_INDICATORS_VALID_FIELDS,
            GQL_INDICATORS_DEFAULT_FIELDS,
            INDICATOR_COMPOSITE_FIELDS,
        ));
        item_selection.push(' ');
    }
    item_selection.push('}');
    item_selection
}

/// Build the `first`/`after` connection argument list shared by paginated
/// batch queries in this file.
fn build_connection_args(limit: Option<u32>, cursor: Option<&str>) -> String {
    let mut conn_args = vec![format!("first: {}", limit.unwrap_or(DEFAULT_MCP_PAGE_SIZE))];
    if let Some(c) = cursor {
        conn_args.push(format!("after: \"{}\"", escape_gql_string(c)));
    }
    conn_args.join(", ")
}

/// Accepts one or more comma-separated symbols: a single symbol returns the
/// flat indicators shape, multiple symbols return the batch `{indicators, errors}` shape.
pub async fn get_indicators(
    schema: &FinanceSchema,
    symbols: String,
    interval: Option<Interval>,
    range: Option<TimeRange>,
    fields: Option<String>,
    limit: Option<u32>,
    cursor: Option<String>,
) -> Result<CallToolResult, McpError> {
    let syms = split_symbols(&symbols);
    if syms.len() == 1 {
        get_one_indicators(
            schema,
            syms.into_iter().next().unwrap(),
            interval,
            range,
            fields,
        )
        .await
    } else {
        get_many_indicators(schema, syms, interval, range, fields, limit, cursor).await
    }
}

async fn get_one_indicators(
    schema: &FinanceSchema,
    symbol: String,
    interval: Option<Interval>,
    range: Option<TimeRange>,
    fields: Option<String>,
) -> Result<CallToolResult, McpError> {
    let gql_interval = interval_to_gql(interval.unwrap_or(Interval::OneDay));
    let gql_range = range_to_gql(range.unwrap_or(TimeRange::OneYear));
    let field_list = parse_fields(fields);
    let selection = build_type_spec_selection(
        field_list.as_deref(),
        GQL_INDICATORS_VALID_FIELDS,
        GQL_INDICATORS_DEFAULT_FIELDS,
        INDICATOR_COMPOSITE_FIELDS,
    );
    let query = format!(
        "query GetIndicators($symbol: String!) {{ ticker(symbol: $symbol) {{ indicators(interval: {gql_interval}, range: {gql_range}) {selection} }} }}"
    );
    let mut variables = async_graphql::Variables::default();
    variables.insert(async_graphql::Name::new("symbol"), symbol.into());
    let json = execute_query(schema, &query, variables).await?;
    let data = unwrap_ticker_field(json, "indicators");
    Ok(CallToolResult::success(vec![rmcp::model::Content::text(
        serde_json::to_string(&data).map_err(ser_err)?,
    )]))
}

async fn get_many_indicators(
    schema: &FinanceSchema,
    syms: Vec<String>,
    interval: Option<Interval>,
    range: Option<TimeRange>,
    fields: Option<String>,
    limit: Option<u32>,
    cursor: Option<String>,
) -> Result<CallToolResult, McpError> {
    let gql_interval = interval_to_gql(interval.unwrap_or(Interval::OneDay));
    let gql_range = range_to_gql(range.unwrap_or(TimeRange::OneYear));
    let syms_literal = gql_string_list_literal(&syms);
    let field_list = parse_fields(fields);
    // "indicators" (`GqlIndicatorsSummary`) is composite and needs its own
    // nested sub-selection, not a bare field name.
    let item_selection = build_indicators_batch_item_selection(field_list.as_deref());
    let selection = build_connection_selection(&item_selection);

    let conn_args = build_connection_args(limit, cursor.as_deref());

    let query = format!(
        "query {{ indicatorsBatch(symbols: [{}], interval: {gql_interval}, range: {gql_range}) {{ indicators({conn_args}) {} errors {{ symbol message }} }} }}",
        syms_literal, selection
    );
    let json = execute_query(schema, &query, async_graphql::Variables::default()).await?;
    let data = wrap_nested_connection(unwrap_field(json, "indicatorsBatch"), "indicators");
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
    fn wants_indicators_field_defaults_true_when_fields_is_none() {
        assert!(wants_indicators_field(None));
    }

    #[test]
    fn wants_indicators_field_false_when_fields_is_empty() {
        // Unlike `None` (no restriction at all), an explicit empty list is a
        // restriction that excludes everything, `indicators` included.
        let empty: Vec<String> = Vec::new();
        assert!(!wants_indicators_field(Some(&empty)));
    }

    #[test]
    fn wants_indicators_field_true_when_explicitly_requested() {
        let fields = vec!["symbol".to_string(), "indicators".to_string()];
        assert!(wants_indicators_field(Some(&fields)));
    }

    #[test]
    fn wants_indicators_field_false_when_fields_excludes_it() {
        let fields = vec!["symbol".to_string()];
        assert!(!wants_indicators_field(Some(&fields)));
    }

    #[test]
    fn build_indicators_batch_item_selection_includes_indicators_by_default() {
        let sel = build_indicators_batch_item_selection(None);
        assert!(sel.starts_with("{ symbol indicators"));
        assert!(sel.contains("sma10"));
        assert!(sel.ends_with('}'));
    }

    #[test]
    fn build_indicators_batch_item_selection_omits_indicators_when_excluded() {
        let fields = vec!["symbol".to_string()];
        let sel = build_indicators_batch_item_selection(Some(&fields));
        assert_eq!(sel, "{ symbol }");
        assert!(!sel.contains("indicators"));
    }

    #[test]
    fn build_indicators_batch_item_selection_expands_nested_fields_when_requested() {
        let fields = vec!["indicators".to_string(), "rsi14".to_string()];
        let sel = build_indicators_batch_item_selection(Some(&fields));
        assert!(sel.contains("indicators"));
        assert!(sel.contains("rsi14"));
        assert!(!sel.contains("sma10"));
    }

    #[test]
    fn build_connection_args_uses_default_page_size_without_cursor() {
        assert_eq!(
            build_connection_args(None, None),
            format!("first: {}", DEFAULT_MCP_PAGE_SIZE)
        );
    }

    #[test]
    fn build_connection_args_uses_given_limit() {
        assert_eq!(build_connection_args(Some(5), None), "first: 5");
    }

    #[test]
    fn build_connection_args_appends_escaped_cursor_when_present() {
        assert_eq!(
            build_connection_args(Some(10), Some("abc\"123")),
            "first: 10, after: \"abc\\\"123\""
        );
    }
}
