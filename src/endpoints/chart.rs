/// Chart data endpoint
///
/// Fetches historical price and volume data for a symbol.
use crate::client::YahooClient;
use crate::constants::{Interval, TimeRange, endpoints};
use crate::error::Result;
use tracing::info;

/// Fetch chart data for a symbol
///
/// # Arguments
///
/// * `client` - The Yahoo Finance client
/// * `symbol` - Stock symbol (e.g., "AAPL")
/// * `interval` - Time interval between data points
/// * `range` - Time range to fetch data for
///
/// # Example
///
/// ```no_run
/// use finance_query::{Interval, TimeRange};
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// # let client = finance_query::YahooClient::new(Default::default()).await?;
/// use finance_query::endpoints::chart;
/// let chart_data = chart::fetch(&client, "AAPL", Interval::OneDay, TimeRange::OneMonth).await?;
/// # Ok(())
/// # }
/// ```
pub async fn fetch(
    client: &YahooClient,
    symbol: &str,
    interval: Interval,
    range: TimeRange,
) -> Result<serde_json::Value> {
    super::common::validate_symbol(symbol)?;

    info!(
        "Fetching chart for {} ({}, {})",
        symbol,
        interval.as_str(),
        range.as_str()
    );

    let url = endpoints::chart(symbol);
    let params = [("interval", interval.as_str()), ("range", range.as_str())];
    let response = client.request_with_params(&url, &params).await?;

    Ok(response.json().await?)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ClientConfig;

    #[tokio::test]
    #[ignore] // Requires network access
    async fn test_fetch_chart() {
        let client = YahooClient::new(ClientConfig::default()).await.unwrap();
        let result = fetch(&client, "AAPL", Interval::OneDay, TimeRange::OneMonth).await;
        assert!(result.is_ok());
        let json = result.unwrap();
        assert!(json.get("chart").is_some());
    }

    #[tokio::test]
    async fn test_empty_symbol() {
        let client = YahooClient::new(ClientConfig::default()).await.unwrap();
        let result = fetch(&client, "", Interval::OneDay, TimeRange::OneMonth).await;
        assert!(result.is_err());
    }
}
