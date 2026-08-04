use finance_query_server::graphql::FinanceSchema;
use rmcp::{ErrorData as McpError, model::CallToolResult};

use crate::error::ser_err;
use crate::tools::gql::{
    DEFAULT_MCP_PAGE_SIZE, GQL_OPTIONS_VALID_FIELDS, OPTIONS_COMPOSITE_FIELDS,
    build_connection_selection, escape_gql_string, execute_query, parse_fields,
    unwrap_ticker_field, wrap_nested_connection,
};

/// Build the options `{ ... }` selection, expanding `calls`/`puts` as paginated
/// Connections sharing the same `first`/`after` args — mirrors
/// `build_options_selection` in `server/src/handlers/options.rs`.
fn build_options_selection(fields: Option<&[String]>, limit: u32, cursor: Option<&str>) -> String {
    let chosen: Vec<&str> = match fields {
        Some(fs) if !fs.is_empty() => fs
            .iter()
            .map(|f| f.trim())
            .filter(|f| GQL_OPTIONS_VALID_FIELDS.contains(f))
            .collect(),
        _ => GQL_OPTIONS_VALID_FIELDS.to_vec(),
    };
    let mut args = vec![format!("first: {limit}")];
    if let Some(cursor) = cursor {
        args.push(format!("after: \"{}\"", escape_gql_string(cursor)));
    }
    let args_str = format!("({})", args.join(", "));
    let mut sel = String::from("{ ");
    for f in chosen {
        sel.push_str(f);
        if f == "calls" || f == "puts" {
            let item_selection = OPTIONS_COMPOSITE_FIELDS
                .iter()
                .find(|(name, _)| *name == f)
                .map(|(_, s)| *s)
                .unwrap_or("{ }");
            sel.push_str(&args_str);
            sel.push(' ');
            sel.push_str(&build_connection_selection(item_selection));
        }
        sel.push(' ');
    }
    sel.push('}');
    sel
}

/// Build the `(date: <ts>)` argument clause for `options`, omitting the
/// parens entirely when no expiration was given — `options()` with empty
/// parens is invalid GraphQL syntax, not "no arguments".
fn build_options_date_arg(expiration: Option<i64>) -> String {
    match expiration {
        Some(ts) => format!("(date: {ts})"),
        None => String::new(),
    }
}

pub async fn get_options(
    schema: &FinanceSchema,
    symbol: String,
    expiration: Option<i64>,
    fields: Option<String>,
    limit: Option<u32>,
    cursor: Option<String>,
) -> Result<CallToolResult, McpError> {
    let date_arg = build_options_date_arg(expiration);
    let field_list = parse_fields(fields);
    let selection = build_options_selection(
        field_list.as_deref(),
        limit.unwrap_or(DEFAULT_MCP_PAGE_SIZE),
        cursor.as_deref(),
    );

    let query = format!(
        "query GetOpts($symbol: String!) {{ ticker(symbol: $symbol) {{ options{} {} }} }}",
        date_arg, selection
    );
    let mut variables = async_graphql::Variables::default();
    variables.insert(async_graphql::Name::new("symbol"), symbol.into());
    let json = execute_query(schema, &query, variables).await?;
    let mut data = unwrap_ticker_field(json, "options");
    data = wrap_nested_connection(data, "calls");
    data = wrap_nested_connection(data, "puts");
    Ok(CallToolResult::success(vec![rmcp::model::Content::text(
        serde_json::to_string(&data).map_err(ser_err)?,
    )]))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn build_options_date_arg_formats_the_expiration_timestamp() {
        assert_eq!(
            build_options_date_arg(Some(1700000000)),
            "(date: 1700000000)"
        );
    }

    #[test]
    fn build_options_date_arg_is_empty_when_expiration_is_none() {
        assert_eq!(build_options_date_arg(None), "");
    }

    #[test]
    fn build_options_selection_includes_all_valid_fields_by_default() {
        let sel = build_options_selection(None, 25, None);
        for f in GQL_OPTIONS_VALID_FIELDS {
            assert!(sel.contains(f), "missing field {f} in {sel}");
        }
    }

    #[test]
    fn build_options_selection_filters_to_requested_valid_fields() {
        let fields = vec!["calls".to_string()];
        let sel = build_options_selection(Some(&fields), 25, None);
        assert!(sel.contains("calls"));
        assert!(!sel.contains("puts"));
        assert!(!sel.contains("expirationDates"));
        assert!(!sel.contains("strikes"));
    }

    #[test]
    fn build_options_selection_drops_unknown_fields_with_no_fallback() {
        // Unlike `build_type_spec_selection`, this helper has no
        // "fall back to defaults" behavior for an all-unknown field list.
        let fields = vec!["bogus".to_string()];
        let sel = build_options_selection(Some(&fields), 25, None);
        assert_eq!(sel, "{ }");
    }

    #[test]
    fn build_options_selection_expands_calls_and_puts_as_paginated_connections() {
        let fields = vec!["calls".to_string(), "puts".to_string()];
        let sel = build_options_selection(Some(&fields), 10, None);
        assert!(sel.contains("calls(first: 10)"));
        assert!(sel.contains("puts(first: 10)"));
        assert!(sel.contains("contractSymbol"));
        assert!(sel.contains("edges"));
        assert!(sel.contains("pageInfo"));
    }

    #[test]
    fn build_options_selection_appends_escaped_cursor_when_present() {
        let fields = vec!["calls".to_string()];
        let sel = build_options_selection(Some(&fields), 10, Some("a\"b"));
        assert!(sel.contains("first: 10, after: \"a\\\"b\""));
    }

    #[test]
    fn build_options_selection_falls_back_to_all_fields_when_fields_is_empty() {
        let empty: Vec<String> = Vec::new();
        let sel = build_options_selection(Some(&empty), 25, None);
        for f in GQL_OPTIONS_VALID_FIELDS {
            assert!(sel.contains(f));
        }
    }
}
