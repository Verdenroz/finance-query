use finance_query_server::graphql::FinanceSchema;
use rmcp::{ErrorData as McpError, model::CallToolResult};

use crate::error::ser_err;
use crate::tools::gql::{
    DEFAULT_MCP_PAGE_SIZE, GQL_FEEDS_DEFAULT_FIELDS, GQL_FEEDS_VALID_FIELDS,
    build_connection_selection, build_selection_or_default, escape_gql_string, execute_query,
    gql_string_list_literal, parse_fields, unwrap_field, wrap_connection,
};

pub async fn get_feeds(
    schema: &FinanceSchema,
    sources: Option<String>,
    fields: Option<String>,
    limit: Option<u32>,
    cursor: Option<String>,
) -> Result<CallToolResult, McpError> {
    let field_list = parse_fields(fields);
    let inner_selection = build_selection_or_default(
        field_list.as_deref(),
        GQL_FEEDS_VALID_FIELDS,
        GQL_FEEDS_DEFAULT_FIELDS,
    );
    let selection = build_connection_selection(&inner_selection);

    // Preserve this tool's historical default sources (distinct from the
    // GraphQL field's own default) by always passing an explicit list.
    let list: Vec<&str> = match sources.as_deref() {
        Some(raw) => raw
            .split(',')
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .collect(),
        None => vec![],
    };
    let list: Vec<&str> = if list.is_empty() {
        vec!["marketwatch", "bloomberg", "wsj", "fortune"]
    } else {
        list
    };
    let mut args = vec![
        format!("sources: [{}]", gql_string_list_literal(&list)),
        format!("first: {}", limit.unwrap_or(DEFAULT_MCP_PAGE_SIZE)),
    ];
    if let Some(c) = cursor.as_deref() {
        args.push(format!("after: \"{}\"", escape_gql_string(c)));
    }
    let args_str = format!("({})", args.join(", "));

    let query = format!("query {{ feeds{args_str} {selection} }}");
    let json = execute_query(schema, &query, async_graphql::Variables::default()).await?;
    let result = wrap_connection(unwrap_field(json, "feeds"));

    let text = serde_json::to_string(&result).map_err(ser_err)?;
    Ok(CallToolResult::success(vec![rmcp::model::Content::text(
        text,
    )]))
}
