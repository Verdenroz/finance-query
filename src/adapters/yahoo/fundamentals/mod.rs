//! Financials timeseries endpoint
//!
//! Fetches financial statement data over time (income statement, balance sheet, cash flow).

use crate::adapters::yahoo::client::YahooClient;
use crate::constants::{Frequency, StatementType};
use crate::error::Result;
use crate::models::fundamentals::FinancialStatement;

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
/// Delegates to [`YahooClient::get_financials`] for the typed result.
pub async fn fetch(
    client: &YahooClient,
    symbol: &str,
    statement_type: StatementType,
    frequency: Frequency,
) -> Result<FinancialStatement> {
    client
        .get_financials(symbol, statement_type, frequency)
        .await
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::adapters::yahoo::client::ClientConfig;

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
