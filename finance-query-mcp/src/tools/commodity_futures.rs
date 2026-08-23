use finance_query_server::graphql::FinanceSchema;
use rmcp::{ErrorData as McpError, model::CallToolResult};

use crate::error::ser_err;
use crate::tools::gql::{
    GQL_COMMODITY_QUOTE_DEFAULT_FIELDS, GQL_COMMODITY_QUOTE_VALID_FIELDS,
    GQL_FUTURES_QUOTE_DEFAULT_FIELDS, GQL_FUTURES_QUOTE_VALID_FIELDS,
    GQL_INDEX_CONSTITUENT_DEFAULT_FIELDS, GQL_INDEX_CONSTITUENT_VALID_FIELDS,
    build_selection_or_default, execute_query, parse_fields, unwrap_field,
};

/// A commodity's current quote (e.g. gold, silver, crude oil),
/// provider-routed (Yahoo, keyless).
pub async fn get_commodity(
    schema: &FinanceSchema,
    symbol: String,
    fields: Option<String>,
) -> Result<CallToolResult, McpError> {
    let field_list = parse_fields(fields);
    let selection = build_selection_or_default(
        field_list.as_deref(),
        GQL_COMMODITY_QUOTE_VALID_FIELDS,
        GQL_COMMODITY_QUOTE_DEFAULT_FIELDS,
    );
    let query = format!(
        "query GetCommodity($symbol: String!) {{ commodity(symbol: $symbol) {selection} }}"
    );
    let mut variables = async_graphql::Variables::default();
    variables.insert(async_graphql::Name::new("symbol"), symbol.into());
    let json = execute_query(schema, &query, variables).await?;
    let data = unwrap_field(json, "commodity");
    Ok(CallToolResult::success(vec![rmcp::model::Content::text(
        serde_json::to_string(&data).map_err(ser_err)?,
    )]))
}

/// A futures contract's current quote, provider-routed (Yahoo, keyless).
pub async fn get_futures(
    schema: &FinanceSchema,
    symbol: String,
    fields: Option<String>,
) -> Result<CallToolResult, McpError> {
    let field_list = parse_fields(fields);
    let selection = build_selection_or_default(
        field_list.as_deref(),
        GQL_FUTURES_QUOTE_VALID_FIELDS,
        GQL_FUTURES_QUOTE_DEFAULT_FIELDS,
    );
    let query =
        format!("query GetFutures($symbol: String!) {{ futures(symbol: $symbol) {selection} }}");
    let mut variables = async_graphql::Variables::default();
    variables.insert(async_graphql::Name::new("symbol"), symbol.into());
    let json = execute_query(schema, &query, variables).await?;
    let data = unwrap_field(json, "futures");
    Ok(CallToolResult::success(vec![rmcp::model::Content::text(
        serde_json::to_string(&data).map_err(ser_err)?,
    )]))
}

/// An index's current constituent list, provider-routed (Wikipedia, S&P 500 only).
pub async fn get_index_constituents(
    schema: &FinanceSchema,
    symbol: String,
    fields: Option<String>,
) -> Result<CallToolResult, McpError> {
    let field_list = parse_fields(fields);
    let selection = build_selection_or_default(
        field_list.as_deref(),
        GQL_INDEX_CONSTITUENT_VALID_FIELDS,
        GQL_INDEX_CONSTITUENT_DEFAULT_FIELDS,
    );
    let query = format!(
        "query GetIndexConstituents($symbol: String!) {{ indexConstituents(symbol: $symbol) {selection} }}"
    );
    let mut variables = async_graphql::Variables::default();
    variables.insert(async_graphql::Name::new("symbol"), symbol.into());
    let json = execute_query(schema, &query, variables).await?;
    let data = unwrap_field(json, "indexConstituents");
    Ok(CallToolResult::success(vec![rmcp::model::Content::text(
        serde_json::to_string(&data).map_err(ser_err)?,
    )]))
}
