use finance_query_server::graphql::FinanceSchema;
use rmcp::{ErrorData as McpError, model::CallToolResult};

use crate::error::ser_err;
use crate::tools::gql::{
    DEFAULT_MCP_PAGE_SIZE, GQL_COIN_VALID_FIELDS, build_connection_selection,
    build_selection_or_default, escape_gql_string, execute_query, parse_fields, unwrap_field,
    wrap_connection,
};

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
    let mut conn_args = vec![format!("first: {}", limit.unwrap_or(DEFAULT_MCP_PAGE_SIZE))];
    if let Some(c) = cursor.as_deref() {
        conn_args.push(format!("after: \"{}\"", escape_gql_string(c)));
    }
    let conn_args = conn_args.join(", ");
    let query = format!(
        "query {{ cryptoCoins(vsCurrency: \"{currency}\", count: {n}, {conn_args}) {selection} }}"
    );
    let json = execute_query(schema, &query, async_graphql::Variables::default()).await?;
    let data = wrap_connection(unwrap_field(json, "cryptoCoins"));
    Ok(CallToolResult::success(vec![rmcp::model::Content::text(
        serde_json::to_string(&data).map_err(ser_err)?,
    )]))
}
