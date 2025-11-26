/// Fundamentals timeseries endpoint
///
/// Fetches financial statement data over time (revenue, income, etc.).
use crate::client::YahooClient;
use crate::constants::endpoints;
use crate::error::Result;
use tracing::info;

/// Fetch fundamentals timeseries data (financial statements)
///
/// # Arguments
///
/// * `client` - The Yahoo Finance client
/// * `symbol` - Stock symbol
/// * `period1` - Start Unix timestamp
/// * `period2` - End Unix timestamp
/// * `types` - List of fundamental types (e.g., "annualTotalRevenue", "quarterlyNetIncome")
///
/// # Example
///
/// ```no_run
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// # let client = finance_query::YahooClient::new(Default::default()).await?;
/// use finance_query::endpoints::timeseries;
/// let types = vec!["annualTotalRevenue", "annualNetIncome"];
/// let financials = timeseries::fetch(&client, "AAPL", 0, 9999999999, &types).await?;
/// # Ok(())
/// # }
/// ```
pub async fn fetch(
    client: &YahooClient,
    symbol: &str,
    period1: i64,
    period2: i64,
    types: &[&str],
) -> Result<serde_json::Value> {
    super::common::validate_symbol(symbol)?;

    if types.is_empty() {
        return Err(crate::error::YahooError::InvalidParameter(
            "No types provided for timeseries".to_string(),
        ));
    }

    info!("Fetching fundamentals timeseries for: {}", symbol);

    let url = endpoints::financials(symbol);
    let params = [
        ("merge", "false"),
        ("padTimeSeries", "true"),
        ("period1", &period1.to_string()),
        ("period2", &period2.to_string()),
        ("type", &types.join(",")),
        ("lang", "en-US"),
        ("region", "US"),
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
    async fn test_fetch_timeseries() {
        let client = YahooClient::new(ClientConfig::default()).await.unwrap();
        let result = fetch(
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
    async fn test_empty_types() {
        let client = YahooClient::new(ClientConfig::default()).await.unwrap();
        let result = fetch(&client, "AAPL", 0, 9999999999, &[]).await;
        assert!(result.is_err());
    }
}
