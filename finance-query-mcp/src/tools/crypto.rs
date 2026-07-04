use finance_query_server::graphql::FinanceSchema;
use rmcp::{ErrorData as McpError, model::CallToolResult};

use crate::error::ser_err;
use crate::tools::gql::{
    GQL_COIN_VALID_FIELDS, build_selection_or_default, execute_query, parse_fields, unwrap_field,
};

pub async fn get_crypto_coins(
    schema: &FinanceSchema,
    count: Option<u32>,
    vs_currency: Option<String>,
    fields: Option<String>,
) -> Result<CallToolResult, McpError> {
    let n = count.unwrap_or(50);
    let currency = vs_currency.as_deref().unwrap_or("usd");
    let field_list = parse_fields(fields);
    let selection = build_selection_or_default(
        field_list.as_deref(),
        GQL_COIN_VALID_FIELDS,
        GQL_COIN_VALID_FIELDS,
    );
    let query =
        format!("query {{ cryptoCoins(vsCurrency: \"{currency}\", count: {n}) {selection} }}");
    let json = execute_query(schema, &query, async_graphql::Variables::default()).await?;
    let data = unwrap_field(json, "cryptoCoins");
    Ok(CallToolResult::success(vec![rmcp::model::Content::text(
        serde_json::to_string(&data).map_err(ser_err)?,
    )]))
}
