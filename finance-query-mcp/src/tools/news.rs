use finance_query_server::graphql::FinanceSchema;
use rmcp::{ErrorData as McpError, model::CallToolResult};

use crate::error::ser_err;
use crate::tools::gql::{
    DEFAULT_MCP_PAGE_SIZE, GQL_NEWS_DEFAULT_FIELDS, GQL_NEWS_VALID_FIELDS,
    build_connection_selection, build_selection_or_default, escape_gql_string, execute_query,
    parse_fields, unwrap_field, unwrap_ticker_field, wrap_connection,
};

pub async fn get_news(
    schema: &FinanceSchema,
    symbol: Option<String>,
    lang: Option<String>,
    fields: Option<String>,
    limit: Option<u32>,
    cursor: Option<String>,
) -> Result<CallToolResult, McpError> {
    let field_list = parse_fields(fields);

    let inner_selection = build_selection_or_default(
        field_list.as_deref(),
        GQL_NEWS_VALID_FIELDS,
        GQL_NEWS_DEFAULT_FIELDS,
    );
    let selection = build_connection_selection(&inner_selection);

    let lang_arg = match crate::lang::normalize(lang.as_deref()) {
        Some(l) => format!(", lang: \"{}\"", l),
        None => String::new(),
    };
    // `count` bounds the overall upstream fetch (kept at its historical value
    // of 10, unchanged); `first`/`after` paginate a page out of that fetched
    // pool — the two are independent GraphQL args on the same field.
    let mut conn_args = vec![
        "count: 10".to_string(),
        format!("first: {}", limit.unwrap_or(DEFAULT_MCP_PAGE_SIZE)),
    ];
    if let Some(c) = cursor.as_deref() {
        conn_args.push(format!("after: \"{}\"", escape_gql_string(c)));
    }
    let conn_args = conn_args.join(", ");

    // Per-symbol and general news hit different root fields, so unwrap
    // inside each branch rather than probing both shapes afterward.
    let result = if let Some(sym) = symbol {
        let query = format!(
            "query GetNews($symbol: String!) {{ ticker(symbol: $symbol) {{ news({conn_args}{lang_arg}) {selection} }} }}"
        );
        let mut variables = async_graphql::Variables::default();
        variables.insert(async_graphql::Name::new("symbol"), sym.into());
        let json = execute_query(schema, &query, variables).await?;
        unwrap_ticker_field(json, "news")
    } else {
        let query = format!("query {{ news({conn_args}{lang_arg}) {selection} }}");
        let json = execute_query(schema, &query, async_graphql::Variables::default()).await?;
        unwrap_field(json, "news")
    };
    let result = wrap_connection(result);

    let text = serde_json::to_string(&result).map_err(ser_err)?;
    Ok(CallToolResult::success(vec![rmcp::model::Content::text(
        text,
    )]))
}
