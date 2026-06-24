use finance_query::Tickers;
use rmcp::{ErrorData as McpError, model::CallToolResult};

use crate::error::{finance_err, ser_err};
use crate::tools::helpers::parse_range;

/// Aggregate upcoming financial events (earnings, dividends, options
/// expirations, and — when `FRED_API_KEY` is set — economic releases) across
/// the given symbols into a single time-sorted list.
pub async fn get_calendar(
    symbols: String,
    range: Option<String>,
) -> Result<CallToolResult, McpError> {
    let range = parse_range(range.as_deref().unwrap_or("1mo"));
    let syms: Vec<&str> = symbols.split(',').map(str::trim).collect();
    let tickers = Tickers::new(syms).await.map_err(finance_err)?;
    let events = tickers.calendar(range).await.map_err(finance_err)?;
    let json = serde_json::to_string(&events).map_err(ser_err)?;
    Ok(CallToolResult::success(vec![rmcp::model::Content::text(
        json,
    )]))
}
