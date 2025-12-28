//! Financials timeseries endpoint
//!
//! Fetches financial statement data over time (income statement, balance sheet, cash flow).

use super::urls::api;
use crate::client::YahooClient;
use crate::client::{API_PARAM_MERGE, API_PARAM_PAD_TIMESERIES};
use crate::constants::{Frequency, StatementType};
use crate::error::Result;
use crate::models::financials::FinancialStatement;
use tracing::info;

/// Fetch financial statement data for a symbol
///
/// # Arguments
///
/// * `client` - The Yahoo Finance client
/// * `symbol` - Stock symbol
/// * `statement_type` - Type of statement (income, balance, cashflow)
/// * `frequency` - Annual or Quarterly
///
/// # Example
///
/// ```ignore
/// use finance_query::{Ticker, StatementType, Frequency};
///
/// let ticker = Ticker::new("AAPL").await?;
/// let statement = ticker.financials(StatementType::Income, Frequency::Annual).await?;
/// println!("Revenue: {:?}", statement.statement.get("TotalRevenue"));
/// ```
pub async fn fetch(
    client: &YahooClient,
    symbol: &str,
    statement_type: StatementType,
    frequency: Frequency,
) -> Result<FinancialStatement> {
    super::common::validate_symbol(symbol)?;

    info!(
        "Fetching {} {} financials for: {}",
        frequency.as_str(),
        statement_type.as_str(),
        symbol
    );

    let fields = statement_type.get_fields();
    let types: Vec<String> = fields.iter().map(|&f| frequency.prefix(f)).collect();
    let types_str = types.join(",");

    let url = api::financials(symbol);

    // Use client config for lang and region
    let config = client.config();

    // Go back ~10 years for historical data
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() as i64;
    let period1 = now - (10 * 365 * 24 * 60 * 60); // 10 years ago

    let params = [
        ("merge", API_PARAM_MERGE),
        ("padTimeSeries", API_PARAM_PAD_TIMESERIES),
        ("period1", &period1.to_string()),
        ("period2", &now.to_string()),
        ("type", types_str.as_str()),
        ("lang", config.lang.as_str()),
        ("region", config.region.as_str()),
    ];
    let response = client.request_with_params(&url, &params).await?;
    let json: serde_json::Value = response.json().await?;

    FinancialStatement::from_response(&json, symbol, statement_type, frequency)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::client::ClientConfig;

    #[tokio::test]
    #[ignore] // Requires network access
    async fn test_fetch_financials() {
        let client = YahooClient::new(ClientConfig::default()).await.unwrap();
        let result = fetch(&client, "AAPL", StatementType::Income, Frequency::Annual).await;
        assert!(result.is_ok());
        let statement = result.unwrap();
        assert_eq!(statement.symbol, "AAPL");
        assert!(statement.statement.contains_key("TotalRevenue"));
    }
}
