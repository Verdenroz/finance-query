use finance_query_server::graphql::FinanceSchema;
use rmcp::{ErrorData as McpError, model::CallToolResult};

use crate::error::ser_err;
use crate::tools::gql::{
    DEFAULT_MCP_PAGE_SIZE, GQL_NEWS_DEFAULT_FIELDS, GQL_NEWS_VALID_FIELDS, NEWS_COMPOSITE_FIELDS,
    build_connection_selection, build_type_spec_selection, escape_gql_string, execute_query,
    parse_fields, unwrap_field, wrap_connection,
};

/// Build the `forexNews(...)` connection args: the upstream-fetch `limit`,
/// the page-size `first`, and an optional `after` cursor.
fn build_news_conn_args(count: Option<u32>, limit: Option<u32>, cursor: Option<&str>) -> String {
    let mut args = vec![
        format!("limit: {}", count.unwrap_or(20)),
        format!("first: {}", limit.unwrap_or(DEFAULT_MCP_PAGE_SIZE)),
    ];
    if let Some(c) = cursor {
        args.push(format!("after: \"{}\"", escape_gql_string(c)));
    }
    args.join(", ")
}

pub async fn get_forex_news(
    schema: &FinanceSchema,
    count: Option<u32>,
    fields: Option<String>,
    limit: Option<u32>,
    cursor: Option<String>,
) -> Result<CallToolResult, McpError> {
    let field_list = parse_fields(fields);
    let inner_selection = build_type_spec_selection(
        field_list.as_deref(),
        GQL_NEWS_VALID_FIELDS,
        GQL_NEWS_DEFAULT_FIELDS,
        NEWS_COMPOSITE_FIELDS,
    );
    let selection = build_connection_selection(&inner_selection);
    let conn_args = build_news_conn_args(count, limit, cursor.as_deref());
    let query = format!("query {{ forexNews({conn_args}) {selection} }}");
    let json = execute_query(schema, &query, async_graphql::Variables::default()).await?;
    let data = wrap_connection(unwrap_field(json, "forexNews"));
    Ok(CallToolResult::success(vec![rmcp::model::Content::text(
        serde_json::to_string(&data).map_err(ser_err)?,
    )]))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn build_news_conn_args_uses_default_count_and_limit_without_cursor() {
        let args = build_news_conn_args(None, None, None);
        assert_eq!(args, format!("limit: 20, first: {DEFAULT_MCP_PAGE_SIZE}"));
    }

    #[test]
    fn build_news_conn_args_uses_given_count_and_limit() {
        let args = build_news_conn_args(Some(5), Some(3), None);
        assert_eq!(args, "limit: 5, first: 3");
    }

    #[test]
    fn build_news_conn_args_includes_cursor_when_present() {
        let args = build_news_conn_args(None, None, Some("abc123"));
        assert_eq!(
            args,
            format!("limit: 20, first: {DEFAULT_MCP_PAGE_SIZE}, after: \"abc123\"")
        );
    }
}
