use finance_query_server::graphql::FinanceSchema;
use rmcp::{ErrorData as McpError, model::CallToolResult};

use crate::error::ser_err;
use crate::tools::gql::{
    DEFAULT_MCP_PAGE_SIZE, GQL_CONGRESSIONAL_TRADE_DEFAULT_FIELDS,
    GQL_CONGRESSIONAL_TRADE_VALID_FIELDS, GQL_FAIL_TO_DELIVER_DEFAULT_FIELDS,
    GQL_FAIL_TO_DELIVER_VALID_FIELDS, build_connection_selection, build_selection_or_default,
    escape_gql_string, execute_query, parse_fields, unwrap_ticker_field, wrap_connection,
};

pub async fn get_congressional_trades(
    schema: &FinanceSchema,
    symbol: String,
    fields: Option<String>,
    limit: Option<u32>,
    cursor: Option<String>,
) -> Result<CallToolResult, McpError> {
    let field_list = parse_fields(fields);
    let inner_selection = build_selection_or_default(
        field_list.as_deref(),
        GQL_CONGRESSIONAL_TRADE_VALID_FIELDS,
        GQL_CONGRESSIONAL_TRADE_DEFAULT_FIELDS,
    );
    let selection = build_connection_selection(&inner_selection);
    let first = limit.unwrap_or(DEFAULT_MCP_PAGE_SIZE);
    let after_arg = cursor
        .as_deref()
        .map(|c| format!(", after: \"{}\"", escape_gql_string(c)))
        .unwrap_or_default();

    let query = format!(
        "query GetCongressionalTrades($symbol: String!) {{ ticker(symbol: $symbol) {{ congressionalTrades(first: {first}{after_arg}) {selection} }} }}"
    );
    let mut variables = async_graphql::Variables::default();
    variables.insert(async_graphql::Name::new("symbol"), symbol.into());
    let json = execute_query(schema, &query, variables).await?;
    let data = wrap_connection(unwrap_ticker_field(json, "congressionalTrades"));
    Ok(CallToolResult::success(vec![rmcp::model::Content::text(
        serde_json::to_string(&data).map_err(ser_err)?,
    )]))
}

pub async fn get_fails_to_deliver(
    schema: &FinanceSchema,
    symbol: String,
    fields: Option<String>,
    limit: Option<u32>,
    cursor: Option<String>,
) -> Result<CallToolResult, McpError> {
    let field_list = parse_fields(fields);
    let inner_selection = build_selection_or_default(
        field_list.as_deref(),
        GQL_FAIL_TO_DELIVER_VALID_FIELDS,
        GQL_FAIL_TO_DELIVER_DEFAULT_FIELDS,
    );
    let selection = build_connection_selection(&inner_selection);
    let first = limit.unwrap_or(DEFAULT_MCP_PAGE_SIZE);
    let after_arg = cursor
        .as_deref()
        .map(|c| format!(", after: \"{}\"", escape_gql_string(c)))
        .unwrap_or_default();

    let query = format!(
        "query GetFailsToDeliver($symbol: String!) {{ ticker(symbol: $symbol) {{ failsToDeliver(first: {first}{after_arg}) {selection} }} }}"
    );
    let mut variables = async_graphql::Variables::default();
    variables.insert(async_graphql::Name::new("symbol"), symbol.into());
    let json = execute_query(schema, &query, variables).await?;
    let data = wrap_connection(unwrap_ticker_field(json, "failsToDeliver"));
    Ok(CallToolResult::success(vec![rmcp::model::Content::text(
        serde_json::to_string(&data).map_err(ser_err)?,
    )]))
}
