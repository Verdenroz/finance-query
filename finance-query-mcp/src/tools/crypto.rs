use finance_query_server::graphql::FinanceSchema;
use rmcp::{ErrorData as McpError, model::CallToolResult};

use crate::error::ser_err;
use crate::tools::gql::{
    DEFAULT_MCP_PAGE_SIZE, GQL_COIN_VALID_FIELDS, GQL_NEWS_DEFAULT_FIELDS, GQL_NEWS_VALID_FIELDS,
    NEWS_COMPOSITE_FIELDS, build_connection_selection, build_selection_or_default,
    build_type_spec_selection, escape_gql_string, execute_query, parse_fields, unwrap_field,
    wrap_connection,
};

/// Build the `first`/`after` connection args for the `cryptoCoins` query.
fn build_conn_args(limit: Option<u32>, cursor: Option<&str>) -> String {
    let mut args = vec![format!("first: {}", limit.unwrap_or(DEFAULT_MCP_PAGE_SIZE))];
    if let Some(c) = cursor {
        args.push(format!("after: \"{}\"", escape_gql_string(c)));
    }
    args.join(", ")
}

pub async fn get_crypto_coins(
    schema: &FinanceSchema,
    count: Option<u32>,
    vs_currency: Option<String>,
    fields: Option<String>,
    limit: Option<u32>,
    cursor: Option<String>,
) -> Result<CallToolResult, McpError> {
    let n = count.unwrap_or(50);
    let currency = vs_currency.as_deref().unwrap_or("usd");
    let field_list = parse_fields(fields);
    let inner_selection = build_selection_or_default(
        field_list.as_deref(),
        GQL_COIN_VALID_FIELDS,
        GQL_COIN_VALID_FIELDS,
    );
    let selection = build_connection_selection(&inner_selection);
    let conn_args = build_conn_args(limit, cursor.as_deref());
    let query = format!(
        "query {{ cryptoCoins(vsCurrency: \"{currency}\", count: {n}, {conn_args}) {selection} }}"
    );
    let json = execute_query(schema, &query, async_graphql::Variables::default()).await?;
    let data = wrap_connection(unwrap_field(json, "cryptoCoins"));
    Ok(CallToolResult::success(vec![rmcp::model::Content::text(
        serde_json::to_string(&data).map_err(ser_err)?,
    )]))
}

/// Build the `cryptoNews(...)` connection args: the upstream-fetch `limit`,
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

pub async fn get_crypto_news(
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
    let query = format!("query {{ cryptoNews({conn_args}) {selection} }}");
    let json = execute_query(schema, &query, async_graphql::Variables::default()).await?;
    let data = wrap_connection(unwrap_field(json, "cryptoNews"));
    Ok(CallToolResult::success(vec![rmcp::model::Content::text(
        serde_json::to_string(&data).map_err(ser_err)?,
    )]))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn build_conn_args_uses_default_limit_without_cursor() {
        let args = build_conn_args(None, None);
        assert_eq!(args, format!("first: {DEFAULT_MCP_PAGE_SIZE}"));
    }

    #[test]
    fn build_conn_args_uses_given_limit() {
        let args = build_conn_args(Some(5), None);
        assert_eq!(args, "first: 5");
    }

    #[test]
    fn build_conn_args_includes_cursor_when_present() {
        let args = build_conn_args(Some(10), Some("abc123"));
        assert_eq!(args, "first: 10, after: \"abc123\"");
    }

    #[test]
    fn build_conn_args_escapes_special_characters_in_cursor() {
        let args = build_conn_args(None, Some("has\"quote\\slash"));
        assert_eq!(
            args,
            format!("first: {DEFAULT_MCP_PAGE_SIZE}, after: \"has\\\"quote\\\\slash\"")
        );
    }
}
