use finance_query::{Ticker, Tickers};
use rmcp::{ErrorData as McpError, model::CallToolResult};

use crate::error::{finance_err, ser_err};
use crate::tools::helpers::parse_range;

pub async fn get_dividends(
    symbol: String,
    range: Option<String>,
) -> Result<CallToolResult, McpError> {
    let r = parse_range(range.as_deref().unwrap_or("max"));
    let ticker = Ticker::new(&symbol).await.map_err(finance_err)?;
    let dividends = ticker.dividends(r).await.map_err(finance_err)?;
    let analytics = ticker.dividend_analytics(r).await.map_err(finance_err)?;
    let combined = serde_json::json!({
        "dividends": dividends,
        "analytics": analytics,
    });
    let json = serde_json::to_string(&combined).map_err(ser_err)?;
    Ok(CallToolResult::success(vec![rmcp::model::Content::text(
        json,
    )]))
}

pub async fn get_batch_dividends(
    symbols: String,
    range: Option<String>,
) -> Result<CallToolResult, McpError> {
    let r = parse_range(range.as_deref().unwrap_or("1y"));
    let syms: Vec<&str> = symbols.split(',').map(str::trim).collect();
    let tickers = Tickers::new(syms).await.map_err(finance_err)?;
    let batch = tickers.dividends(r).await.map_err(finance_err)?;
    let json = serde_json::to_string(&batch).map_err(ser_err)?;
    Ok(CallToolResult::success(vec![rmcp::model::Content::text(
        json,
    )]))
}
