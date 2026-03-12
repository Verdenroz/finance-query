use finance_query::Ticker;
use rmcp::{ErrorData as McpError, model::CallToolResult};

use crate::error::{finance_err, ser_err};

pub async fn get_options(
    symbol: String,
    expiration: Option<i64>,
) -> Result<CallToolResult, McpError> {
    let ticker = Ticker::new(&symbol).await.map_err(finance_err)?;
    let options = ticker.options(expiration).await.map_err(finance_err)?;
    let json = serde_json::to_string(&options).map_err(ser_err)?;
    Ok(CallToolResult::success(vec![rmcp::model::Content::text(
        json,
    )]))
}
