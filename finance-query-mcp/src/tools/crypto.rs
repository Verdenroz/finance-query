use finance_query::crypto;
use rmcp::{ErrorData as McpError, model::CallToolResult};

use crate::error::{finance_err, ser_err};

pub async fn get_crypto_coins(
    count: Option<u32>,
    vs_currency: Option<String>,
) -> Result<CallToolResult, McpError> {
    let n = count.unwrap_or(50);
    let currency = vs_currency.as_deref().unwrap_or("usd");
    let coins = crypto::coins(currency, n as usize)
        .await
        .map_err(finance_err)?;
    let json = serde_json::to_string(&coins).map_err(ser_err)?;
    Ok(CallToolResult::success(vec![rmcp::model::Content::text(
        json,
    )]))
}
