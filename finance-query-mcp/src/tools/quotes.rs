use finance_query::{Ticker, Tickers, TimeRange};
use rmcp::{ErrorData as McpError, model::CallToolResult};

use crate::error::{finance_err, ser_err};
use crate::tools::helpers::parse_range;

pub async fn get_quote(symbol: String) -> Result<CallToolResult, McpError> {
    let ticker = Ticker::new(&symbol).await.map_err(finance_err)?;
    let quote = ticker.quote().await.map_err(finance_err)?;
    let json = serde_json::to_string(&quote).map_err(ser_err)?;
    Ok(CallToolResult::success(vec![rmcp::model::Content::text(
        json,
    )]))
}

pub async fn get_quotes(symbols: String) -> Result<CallToolResult, McpError> {
    let syms: Vec<&str> = symbols.split(',').map(str::trim).collect();
    let tickers = Tickers::new(syms).await.map_err(finance_err)?;
    let batch = tickers.quotes().await.map_err(finance_err)?;
    let json = serde_json::to_string(&batch).map_err(ser_err)?;
    Ok(CallToolResult::success(vec![rmcp::model::Content::text(
        json,
    )]))
}

pub async fn get_recommendations(
    symbol: String,
    limit: Option<u32>,
) -> Result<CallToolResult, McpError> {
    let ticker = Ticker::new(&symbol).await.map_err(finance_err)?;
    let recs = ticker
        .recommendations(limit.unwrap_or(5))
        .await
        .map_err(finance_err)?;
    let json = serde_json::to_string(&recs).map_err(ser_err)?;
    Ok(CallToolResult::success(vec![rmcp::model::Content::text(
        json,
    )]))
}

pub async fn get_splits(symbol: String, range: Option<String>) -> Result<CallToolResult, McpError> {
    let r: TimeRange = range.as_deref().map(parse_range).unwrap_or(TimeRange::Max);
    let ticker = Ticker::new(&symbol).await.map_err(finance_err)?;
    let splits = ticker.splits(r).await.map_err(finance_err)?;
    let json = serde_json::to_string(&splits).map_err(ser_err)?;
    Ok(CallToolResult::success(vec![rmcp::model::Content::text(
        json,
    )]))
}
