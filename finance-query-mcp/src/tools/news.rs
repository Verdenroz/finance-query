use finance_query_server::graphql::FinanceSchema;
use rmcp::{ErrorData as McpError, model::CallToolResult};

use crate::error::ser_err;
use crate::tools::gql::{
    GQL_NEWS_DEFAULT_FIELDS, GQL_NEWS_VALID_FIELDS, build_selection_or_default, execute_query,
    parse_fields, unwrap_field, unwrap_ticker_field,
};

pub async fn get_news(
    schema: &FinanceSchema,
    symbol: Option<String>,
    lang: Option<String>,
    fields: Option<String>,
) -> Result<CallToolResult, McpError> {
    let field_list = parse_fields(fields);

    let selection = build_selection_or_default(
        field_list.as_deref(),
        GQL_NEWS_VALID_FIELDS,
        GQL_NEWS_DEFAULT_FIELDS,
    );

    let lang_arg = match crate::lang::normalize(lang.as_deref()) {
        Some(l) => format!(", lang: \"{}\"", l),
        None => String::new(),
    };

    // Per-symbol and general news hit different root fields, so unwrap
    // inside each branch rather than probing both shapes afterward.
    let result = if let Some(sym) = symbol {
        let query = format!(
            "query GetNews($symbol: String!) {{ ticker(symbol: $symbol) {{ news(count: 10{lang_arg}) {selection} }} }}"
        );
        let mut variables = async_graphql::Variables::default();
        variables.insert(async_graphql::Name::new("symbol"), sym.into());
        let json = execute_query(schema, &query, variables).await?;
        unwrap_ticker_field(json, "news")
    } else {
        let query = format!("query {{ news(count: 10{lang_arg}) {selection} }}");
        let json = execute_query(schema, &query, async_graphql::Variables::default()).await?;
        unwrap_field(json, "news")
    };

    let text = serde_json::to_string(&result).map_err(ser_err)?;
    Ok(CallToolResult::success(vec![rmcp::model::Content::text(
        text,
    )]))
}
