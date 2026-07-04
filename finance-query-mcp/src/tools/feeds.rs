use finance_query_server::graphql::FinanceSchema;
use rmcp::{ErrorData as McpError, model::CallToolResult};

use crate::error::ser_err;
use crate::tools::gql::{
    GQL_FEEDS_DEFAULT_FIELDS, GQL_FEEDS_VALID_FIELDS, build_selection_or_default, execute_query,
    gql_string_list_literal, parse_fields, unwrap_field,
};

pub async fn get_feeds(
    schema: &FinanceSchema,
    sources: Option<String>,
    fields: Option<String>,
) -> Result<CallToolResult, McpError> {
    let field_list = parse_fields(fields);
    let selection = build_selection_or_default(
        field_list.as_deref(),
        GQL_FEEDS_VALID_FIELDS,
        GQL_FEEDS_DEFAULT_FIELDS,
    );

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
    let sources_arg = format!("(sources: [{}])", gql_string_list_literal(&list));

    let query = format!("query {{ feeds{sources_arg} {selection} }}");
    let json = execute_query(schema, &query, async_graphql::Variables::default()).await?;
    let result = unwrap_field(json, "feeds");

    let text = serde_json::to_string(&result).map_err(ser_err)?;
    Ok(CallToolResult::success(vec![rmcp::model::Content::text(
        text,
    )]))
}
