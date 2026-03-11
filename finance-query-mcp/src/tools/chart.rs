use finance_query::{Ticker, Tickers};
use rmcp::{ErrorData as McpError, model::CallToolResult};

use crate::error::{finance_err, ser_err};
use crate::tools::helpers::{parse_interval, parse_range};

pub async fn get_chart(
    symbol: String,
    interval: Option<String>,
    range: Option<String>,
) -> Result<CallToolResult, McpError> {
    let interval = parse_interval(interval.as_deref().unwrap_or("1d"));
    let range = parse_range(range.as_deref().unwrap_or("1mo"));
    let ticker = Ticker::new(&symbol).await.map_err(finance_err)?;
    let chart = ticker.chart(interval, range).await.map_err(finance_err)?;
    let json = serde_json::to_string(&chart).map_err(ser_err)?;
    Ok(CallToolResult::success(vec![rmcp::model::Content::text(json)]))
}

pub async fn get_charts(
    symbols: String,
    interval: Option<String>,
    range: Option<String>,
) -> Result<CallToolResult, McpError> {
    let interval = parse_interval(interval.as_deref().unwrap_or("1d"));
    let range = parse_range(range.as_deref().unwrap_or("1mo"));
    let syms: Vec<&str> = symbols.split(',').map(str::trim).collect();
    let tickers = Tickers::new(syms).await.map_err(finance_err)?;
    let batch = tickers.charts(interval, range).await.map_err(finance_err)?;
    let json = serde_json::to_string(&batch).map_err(ser_err)?;
    Ok(CallToolResult::success(vec![rmcp::model::Content::text(json)]))
}

pub async fn get_spark(
    symbols: String,
    interval: Option<String>,
    range: Option<String>,
) -> Result<CallToolResult, McpError> {
    let interval = parse_interval(interval.as_deref().unwrap_or("1d"));
    let range = parse_range(range.as_deref().unwrap_or("1mo"));
    let syms: Vec<&str> = symbols.split(',').map(str::trim).collect();
    let tickers = Tickers::new(syms).await.map_err(finance_err)?;
    let batch = tickers.spark(interval, range).await.map_err(finance_err)?;
    let json = serde_json::to_string(&batch).map_err(ser_err)?;
    Ok(CallToolResult::success(vec![rmcp::model::Content::text(json)]))
}
