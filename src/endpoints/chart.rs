use super::urls::api;
/// Chart data endpoint
///
/// Fetches historical price and volume data for a symbol.
use crate::client::YahooClient;
use crate::constants::{Interval, TimeRange};
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
/// ```ignore
/// use finance_query::{Interval, TimeRange};
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// # let client = finance_query::YahooClient::new(Default::default()).await?;
/// use finance_query::api::chart;
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

    let url = api::chart(symbol);
    let params = [
        ("interval", interval.as_str()),
        ("range", range.as_str()),
        ("events", "div|split|capitalGain"),
    ];
    let response = client.request_with_params(&url, &params).await?;

    Ok(response.json().await?)
}

/// Fetch chart data for a symbol using absolute date boundaries.
///
/// Uses Yahoo's `period1`/`period2` parameters instead of a relative range.
///
/// # Arguments
///
/// * `client` - The Yahoo Finance client
/// * `symbol` - Stock symbol (e.g., "AAPL")
/// * `interval` - Time interval between data points
/// * `start` - Start date as Unix timestamp (seconds)
/// * `end` - End date as Unix timestamp (seconds)
pub async fn fetch_with_dates(
    client: &YahooClient,
    symbol: &str,
    interval: Interval,
    start: i64,
    end: i64,
) -> Result<serde_json::Value> {
    super::common::validate_symbol(symbol)?;

    info!(
        "Fetching chart for {} ({}, period1={}, period2={})",
        symbol,
        interval.as_str(),
        start,
        end
    );

    let url = api::chart(symbol);
    let start_str = start.to_string();
    let end_str = end.to_string();
    let params = [
        ("interval", interval.as_str()),
        ("period1", start_str.as_str()),
        ("period2", end_str.as_str()),
        ("events", "div|split|capitalGain"),
    ];
    let response = client.request_with_params(&url, &params).await?;

    Ok(response.json().await?)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::client::ClientConfig;

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
    #[ignore = "requires network access - validation tested in common::tests"]
    async fn test_empty_symbol() {
        let client = YahooClient::new(ClientConfig::default()).await.unwrap();
        let result = fetch(&client, "", Interval::OneDay, TimeRange::OneMonth).await;
        assert!(result.is_err());
    }
}
