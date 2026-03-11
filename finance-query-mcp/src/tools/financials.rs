use finance_query::{Ticker, Tickers};
use rmcp::{ErrorData as McpError, model::CallToolResult};

use crate::error::{finance_err, invalid_params, ser_err};
use crate::tools::helpers::{parse_frequency, parse_statement_type};

pub async fn get_financials(
    symbol: String,
    statement: String,
    frequency: Option<String>,
) -> Result<CallToolResult, McpError> {
    let st = parse_statement_type(&statement)
        .ok_or_else(|| invalid_params(format!("Invalid statement type: '{statement}'. Use: income, balance, cashflow")))?;
    let freq = parse_frequency(frequency.as_deref().unwrap_or("annual"));
    let ticker = Ticker::new(&symbol).await.map_err(finance_err)?;
    let financials = ticker.financials(st, freq).await.map_err(finance_err)?;
    let json = serde_json::to_string(&financials).map_err(ser_err)?;
    Ok(CallToolResult::success(vec![rmcp::model::Content::text(json)]))
}

pub async fn get_batch_financials(
    symbols: String,
    statement: String,
    frequency: Option<String>,
) -> Result<CallToolResult, McpError> {
    let st = parse_statement_type(&statement)
        .ok_or_else(|| invalid_params(format!("Invalid statement type: '{statement}'. Use: income, balance, cashflow")))?;
    let freq = parse_frequency(frequency.as_deref().unwrap_or("annual"));
    let syms: Vec<&str> = symbols.split(',').map(str::trim).collect();
    let tickers = Tickers::new(syms).await.map_err(finance_err)?;
    let batch = tickers.financials(st, freq).await.map_err(finance_err)?;
    let json = serde_json::to_string(&batch).map_err(ser_err)?;
    Ok(CallToolResult::success(vec![rmcp::model::Content::text(json)]))
}
