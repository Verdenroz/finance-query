//! Financials timeseries endpoint
//!
//! Fetches financial statement data over time (income statement, balance sheet, cash flow).

use crate::client::YahooClient;
use crate::constants::{Frequency, StatementType, api_params, endpoints};
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
/// ```no_run
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// # let client = finance_query::YahooClient::new(Default::default()).await?;
/// use finance_query::endpoints::financials;
/// use finance_query::constants::{StatementType, Frequency};
///
/// let statement = financials::fetch(
///     &client,
///     "AAPL",
///     StatementType::Income,
///     Frequency::Annual
/// ).await?;
/// println!("Revenue: {:?}", statement.statement.get("TotalRevenue"));
/// # Ok(())
/// # }
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

    let url = endpoints::financials(symbol);

    // Use client config for lang and region
    let config = client.config();

    // Go back ~10 years for historical data
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() as i64;
    let period1 = now - (10 * 365 * 24 * 60 * 60); // 10 years ago

    let params = [
        ("merge", api_params::MERGE),
        ("padTimeSeries", api_params::PAD_TIMESERIES),
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

/// Fetch raw fundamentals timeseries data (for advanced use cases)
///
/// This returns the raw JSON from Yahoo Finance's fundamentals-timeseries endpoint.
/// For most use cases, prefer `fetch()` which returns a parsed `FinancialStatement`.
///
/// # Arguments
///
/// * `client` - The Yahoo Finance client
/// * `symbol` - Stock symbol
/// * `period1` - Start Unix timestamp
/// * `period2` - End Unix timestamp
/// * `types` - List of fundamentals field keys (e.g., "annualTotalRevenue", "quarterlyNetIncome")
///
/// # Example
///
/// ```no_run
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// # let client = finance_query::YahooClient::new(Default::default()).await?;
/// use finance_query::endpoints::financials;
///
/// let types = vec!["annualTotalRevenue", "annualNetIncome"];
/// let raw = financials::fetch_raw(&client, "AAPL", 0, 9999999999, &types).await?;
/// # Ok(())
/// # }
/// ```
pub async fn fetch_raw(
    client: &YahooClient,
    symbol: &str,
    period1: i64,
    period2: i64,
    types: &[&str],
) -> Result<serde_json::Value> {
    super::common::validate_symbol(symbol)?;

    if types.is_empty() {
        return Err(crate::error::YahooError::InvalidParameter {
            param: "types".to_string(),
            reason: "No types provided for fundamentals".to_string(),
        });
    }

    info!("Fetching raw fundamentals-timeseries for: {}", symbol);

    let url = endpoints::financials(symbol);

    let yahoo_types = types.join(",");

    // Use client config for lang and region
    let config = client.config();
    let params = [
        ("merge", api_params::MERGE),
        ("padTimeSeries", api_params::PAD_TIMESERIES),
        ("period1", &period1.to_string()),
        ("period2", &period2.to_string()),
        ("type", yahoo_types.as_str()),
        ("lang", config.lang.as_str()),
        ("region", config.region.as_str()),
    ];
    let response = client.request_with_params(&url, &params).await?;

    Ok(response.json().await?)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ClientConfig;

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

    #[tokio::test]
    #[ignore] // Requires network access
    async fn test_fetch_raw_fundamentals() {
        let client = YahooClient::new(ClientConfig::default()).await.unwrap();
        let result = fetch_raw(
            &client,
            "AAPL",
            0,
            9999999999,
            &["annualTotalRevenue", "annualNetIncome"],
        )
        .await;
        assert!(result.is_ok());
        let json = result.unwrap();
        assert!(json.get("timeseries").is_some());
    }

    #[tokio::test]
    #[ignore] // Requires network access (YahooClient::new establishes session)
    async fn test_empty_types() {
        let client = YahooClient::new(ClientConfig::default()).await.unwrap();
        let result = fetch_raw(&client, "AAPL", 0, 9999999999, &[]).await;
        assert!(result.is_err());
    }
}
