use finance_query::Ticker;
use rmcp::{ErrorData as McpError, model::CallToolResult};

use crate::error::{finance_err, ser_err};
use crate::tools::helpers::{parse_interval, parse_range};

pub async fn get_risk(
    symbol: String,
    interval: Option<String>,
    range: Option<String>,
    benchmark: Option<String>,
) -> Result<CallToolResult, McpError> {
    let interval = parse_interval(interval.as_deref().unwrap_or("1d"));
    let range = parse_range(range.as_deref().unwrap_or("1y"));
    let ticker = Ticker::new(&symbol).await.map_err(finance_err)?;
    let risk = ticker.risk(interval, range, benchmark.as_deref()).await.map_err(finance_err)?;
    let json = serde_json::to_string(&risk).map_err(ser_err)?;
    Ok(CallToolResult::success(vec![rmcp::model::Content::text(json)]))
}
