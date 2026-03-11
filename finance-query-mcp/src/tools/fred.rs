use finance_query::fred;
use rmcp::{ErrorData as McpError, model::CallToolResult};

use crate::error::{finance_err, invalid_params, ser_err};

pub async fn get_fred_series(id: String) -> Result<CallToolResult, McpError> {
    if std::env::var("FRED_API_KEY").is_err() {
        return Err(invalid_params(
            "FRED not configured — set the FRED_API_KEY environment variable to enable FRED tools",
        ));
    }
    let series = fred::series(&id).await.map_err(finance_err)?;
    let json = serde_json::to_string(&series).map_err(ser_err)?;
    Ok(CallToolResult::success(vec![rmcp::model::Content::text(json)]))
}

pub async fn get_treasury_yields(year: Option<u32>) -> Result<CallToolResult, McpError> {
    let y = year.unwrap_or(2025);
    let yields = fred::treasury_yields(y).await.map_err(finance_err)?;
    let json = serde_json::to_string(&yields).map_err(ser_err)?;
    Ok(CallToolResult::success(vec![rmcp::model::Content::text(json)]))
}
