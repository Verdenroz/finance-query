use finance_query_server::graphql::FinanceSchema;
use rmcp::{ErrorData as McpError, model::CallToolResult};

use crate::error::ser_err;
use crate::tools::gql::{
    DEFAULT_MCP_PAGE_SIZE, DIVIDENDS_COMPOSITE_FIELDS, GQL_DIVIDENDS_DEFAULT_FIELDS,
    GQL_DIVIDENDS_VALID_FIELDS, build_paginated_composite_selection, execute_query,
    gql_string_list_literal, parse_fields, unwrap_field, unwrap_ticker_field,
    wrap_nested_connection,
};

/// Split a comma-separated symbols param into a trimmed, non-empty list.
fn split_symbols(symbols: &str) -> Vec<String> {
    symbols
        .split(',')
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect()
}

/// Accepts one or more comma-separated symbols: a single symbol returns the
/// flat shape with dividend `analytics` (CAGR, average payment, etc.) and
/// paginated dividend history; multiple symbols return the batch
/// `{dividends, errors}` shape (dividend entries only, no per-symbol analytics
/// — the underlying batch query doesn't compute it).
pub async fn get_dividends(
    schema: &FinanceSchema,
    symbols: String,
    range: Option<String>,
    fields: Option<String>,
    limit: Option<u32>,
    cursor: Option<String>,
) -> Result<CallToolResult, McpError> {
    let syms = split_symbols(&symbols);
    if syms.len() == 1 {
        get_one_dividends(
            schema,
            syms.into_iter().next().unwrap(),
            range,
            fields,
            limit,
            cursor,
        )
        .await
    } else {
        get_many_dividends(schema, syms, range, fields).await
    }
}

/// Build the `dividends(...)` selection for the single-symbol path: the
/// `dividends` list is the one paginated composite field, everything else
/// (e.g. `analytics`) uses the plain default/valid field lists.
fn build_dividends_selection(
    field_list: Option<&[String]>,
    limit: u32,
    cursor: Option<&str>,
) -> String {
    let dividends_item_selection = DIVIDENDS_COMPOSITE_FIELDS
        .iter()
        .find(|(name, _)| *name == "dividends")
        .map(|(_, sel)| *sel)
        .unwrap_or("{ timestamp amount }");
    let fields_csv = field_list.map(|fs| fs.join(","));
    build_paginated_composite_selection(
        fields_csv.as_deref(),
        GQL_DIVIDENDS_VALID_FIELDS,
        GQL_DIVIDENDS_DEFAULT_FIELDS,
        DIVIDENDS_COMPOSITE_FIELDS,
        "dividends",
        dividends_item_selection,
        Some(limit),
        cursor,
    )
}

async fn get_one_dividends(
    schema: &FinanceSchema,
    symbol: String,
    range: Option<String>,
    fields: Option<String>,
    limit: Option<u32>,
    cursor: Option<String>,
) -> Result<CallToolResult, McpError> {
    let field_list = parse_fields(fields);
    let selection = build_dividends_selection(
        field_list.as_deref(),
        limit.unwrap_or(DEFAULT_MCP_PAGE_SIZE),
        cursor.as_deref(),
    );
    let r = range.as_deref().unwrap_or("max").to_lowercase();
    let gql_range = crate::tools::helpers::range_to_gql(&r);

    let query = format!(
        "query GetDivs($symbol: String!) {{ ticker(symbol: $symbol) {{ dividends(range: {gql_range}) {selection} }} }}"
    );
    let mut variables = async_graphql::Variables::default();
    variables.insert(async_graphql::Name::new("symbol"), symbol.into());

    let json = execute_query(schema, &query, variables).await?;
    let data = wrap_nested_connection(unwrap_ticker_field(json, "dividends"), "dividends");
    let text = serde_json::to_string(&data).map_err(ser_err)?;
    Ok(CallToolResult::success(vec![rmcp::model::Content::text(
        text,
    )]))
}

/// Build the outer `{ symbol [dividends { timestamp amount }] }` selection
/// for a batch dividends response, omitting the nested `dividends` block
/// entirely when not requested.
fn build_batch_dividends_selection(want_dividends: bool) -> String {
    let mut selection = String::from("{ symbol ");
    if want_dividends {
        selection.push_str("dividends { timestamp amount } ");
    }
    selection.push('}');
    selection
}

async fn get_many_dividends(
    schema: &FinanceSchema,
    syms: Vec<String>,
    range: Option<String>,
    fields: Option<String>,
) -> Result<CallToolResult, McpError> {
    let r = range.as_deref().unwrap_or("1y").to_lowercase();
    let gql_range = crate::tools::helpers::range_to_gql(&r);

    let field_list = parse_fields(fields);
    // "dividends" here is Vec<GqlDividend> (no per-symbol analytics, unlike
    // single-symbol GqlDividends) — needs its own nested sub-selection.
    let want_dividends = field_list
        .as_ref()
        .map(|fs| fs.iter().any(|f| f == "dividends"))
        .unwrap_or(true);
    let selection = build_batch_dividends_selection(want_dividends);

    let syms_literal = gql_string_list_literal(&syms);

    let query = format!(
        "query {{ dividendsBatch(symbols: [{}], range: {gql_range}) {{ dividends {} errors {{ symbol message }} }} }}",
        syms_literal, selection
    );

    let json = execute_query(schema, &query, async_graphql::Variables::default()).await?;
    let data = unwrap_field(json, "dividendsBatch");
    let text = serde_json::to_string(&data).map_err(ser_err)?;
    Ok(CallToolResult::success(vec![rmcp::model::Content::text(
        text,
    )]))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fields(names: &[&str]) -> Vec<String> {
        names.iter().map(|s| s.to_string()).collect()
    }

    #[test]
    fn split_symbols_trims_and_filters_empty_entries() {
        assert_eq!(split_symbols("AAPL"), vec!["AAPL".to_string()]);
        assert_eq!(
            split_symbols("AAPL, MSFT ,GOOG"),
            vec!["AAPL".to_string(), "MSFT".to_string(), "GOOG".to_string()]
        );
        assert_eq!(
            split_symbols("AAPL,,MSFT,"),
            vec!["AAPL".to_string(), "MSFT".to_string()]
        );
    }

    #[test]
    fn split_symbols_returns_empty_vec_for_blank_input() {
        assert_eq!(split_symbols(""), Vec::<String>::new());
        assert_eq!(split_symbols("   "), Vec::<String>::new());
        assert_eq!(split_symbols(","), Vec::<String>::new());
    }

    #[test]
    fn build_dividends_selection_defaults_include_dividends_and_analytics() {
        let selection = build_dividends_selection(None, 25, None);

        assert!(selection.contains("dividends"));
        assert!(selection.contains("analytics"));
        assert!(selection.contains("edges"));
        assert!(selection.contains("pageInfo"));
        assert!(selection.contains("first: 25"));
    }

    #[test]
    fn build_dividends_selection_includes_cursor_when_present() {
        let selection = build_dividends_selection(None, 10, Some("abc123"));

        assert!(selection.contains("first: 10"));
        assert!(selection.contains("after: \"abc123\""));
    }

    #[test]
    fn build_dividends_selection_respects_explicit_field_list() {
        let requested = fields(&["analytics"]);
        let selection = build_dividends_selection(Some(&requested), 25, None);

        assert!(selection.contains("analytics"));
        assert!(selection.contains("totalPaid"));
        assert!(!selection.contains("edges"));
    }

    #[test]
    fn build_batch_dividends_selection_includes_dividends_block_when_requested() {
        let selection = build_batch_dividends_selection(true);

        assert_eq!(selection, "{ symbol dividends { timestamp amount } }");
    }

    #[test]
    fn build_batch_dividends_selection_omits_dividends_block_when_not_requested() {
        let selection = build_batch_dividends_selection(false);

        assert_eq!(selection, "{ symbol }");
    }
}
