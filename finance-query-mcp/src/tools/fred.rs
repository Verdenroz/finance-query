use finance_query_server::graphql::FinanceSchema;
use rmcp::{ErrorData as McpError, model::CallToolResult};

use crate::error::{invalid_params, ser_err};
use crate::tools::gql::{
    DEFAULT_MCP_PAGE_SIZE, GQL_MACRO_SERIES_VALID_FIELDS, GQL_TREASURY_YIELD_VALID_FIELDS,
    MACRO_SERIES_COMPOSITE_FIELDS, build_connection_selection, build_paginated_composite_selection,
    build_selection_or_default, escape_gql_string, execute_query, parse_fields, unwrap_field,
    wrap_connection, wrap_nested_connection,
};

/// Look up the `observations` nested selection from `MACRO_SERIES_COMPOSITE_FIELDS`,
/// falling back to a minimal `date`/`value` selection if the entry is ever missing.
fn observations_selection() -> &'static str {
    MACRO_SERIES_COMPOSITE_FIELDS
        .iter()
        .find(|(name, _)| *name == "observations")
        .map(|(_, sel)| *sel)
        .unwrap_or("{ date value }")
}

pub async fn get_fred_series(
    schema: &FinanceSchema,
    id: String,
    fields: Option<String>,
    limit: Option<u32>,
    cursor: Option<String>,
    as_of: Option<String>,
) -> Result<CallToolResult, McpError> {
    if std::env::var("FRED_API_KEY").is_err() {
        return Err(invalid_params(
            "FRED not configured — set the FRED_API_KEY environment variable to enable FRED tools",
        ));
    }
    let field_list = parse_fields(fields);
    let observations_item_selection = observations_selection();
    let fields_csv = field_list.as_ref().map(|fs| fs.join(","));
    let selection = build_paginated_composite_selection(
        fields_csv.as_deref(),
        GQL_MACRO_SERIES_VALID_FIELDS,
        GQL_MACRO_SERIES_VALID_FIELDS,
        MACRO_SERIES_COMPOSITE_FIELDS,
        "observations",
        observations_item_selection,
        Some(limit.unwrap_or(DEFAULT_MCP_PAGE_SIZE)),
        cursor.as_deref(),
    );
    let as_of_arg = as_of
        .as_deref()
        .map(|d| format!(", asOf: \"{}\"", escape_gql_string(d)))
        .unwrap_or_default();
    let query = format!(
        "query GetFredSeries($id: String!) {{ fredSeries(id: $id{as_of_arg}) {selection} }}"
    );
    let mut variables = async_graphql::Variables::default();
    variables.insert(async_graphql::Name::new("id"), id.into());
    let json = execute_query(schema, &query, variables).await?;
    let data = wrap_nested_connection(unwrap_field(json, "fredSeries"), "observations");
    Ok(CallToolResult::success(vec![rmcp::model::Content::text(
        serde_json::to_string(&data).map_err(ser_err)?,
    )]))
}

/// Build the `treasuryYields(...)` field argument list: an optional `year`
/// filter, always followed by `first` and an optional `after` cursor.
fn build_treasury_args(year: Option<u32>, limit: Option<u32>, cursor: Option<&str>) -> String {
    let mut args = Vec::new();
    if let Some(y) = year {
        args.push(format!("year: {y}"));
    }
    args.push(format!("first: {}", limit.unwrap_or(DEFAULT_MCP_PAGE_SIZE)));
    if let Some(c) = cursor {
        args.push(format!("after: \"{}\"", escape_gql_string(c)));
    }
    args.join(", ")
}

pub async fn get_treasury_yields(
    schema: &FinanceSchema,
    year: Option<u32>,
    fields: Option<String>,
    limit: Option<u32>,
    cursor: Option<String>,
) -> Result<CallToolResult, McpError> {
    let field_list = parse_fields(fields);
    let inner_selection = build_selection_or_default(
        field_list.as_deref(),
        GQL_TREASURY_YIELD_VALID_FIELDS,
        GQL_TREASURY_YIELD_VALID_FIELDS,
    );
    let selection = build_connection_selection(&inner_selection);
    let args = build_treasury_args(year, limit, cursor.as_deref());
    let query = format!("query {{ treasuryYields({args}) {selection} }}");
    let json = execute_query(schema, &query, async_graphql::Variables::default()).await?;
    let data = wrap_connection(unwrap_field(json, "treasuryYields"));
    Ok(CallToolResult::success(vec![rmcp::model::Content::text(
        serde_json::to_string(&data).map_err(ser_err)?,
    )]))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn observations_selection_returns_a_nonempty_brace_wrapped_selection() {
        let sel = observations_selection();
        assert!(sel.starts_with('{'));
        assert!(sel.ends_with('}'));
        assert!(sel.contains("date"));
        assert!(sel.contains("value"));
    }

    #[test]
    fn build_treasury_args_uses_default_limit_without_year_or_cursor() {
        let args = build_treasury_args(None, None, None);
        assert_eq!(args, format!("first: {DEFAULT_MCP_PAGE_SIZE}"));
    }

    #[test]
    fn build_treasury_args_includes_year_before_first() {
        let args = build_treasury_args(Some(2024), Some(10), None);
        assert_eq!(args, "year: 2024, first: 10");
    }

    #[test]
    fn build_treasury_args_includes_cursor_after_first() {
        let args = build_treasury_args(None, Some(10), Some("cur1"));
        assert_eq!(args, "first: 10, after: \"cur1\"");
    }

    #[test]
    fn build_treasury_args_includes_year_first_and_cursor_together() {
        let args = build_treasury_args(Some(2023), Some(5), Some("cur2"));
        assert_eq!(args, "year: 2023, first: 5, after: \"cur2\"");
    }

    #[test]
    fn build_treasury_args_escapes_special_characters_in_cursor() {
        let args = build_treasury_args(None, None, Some("has\"quote"));
        assert!(args.contains("after: \"has\\\"quote\""));
    }
}
