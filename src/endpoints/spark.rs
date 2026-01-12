//! Spark endpoint for batch sparkline data.
//!
//! Fetches lightweight chart data for multiple symbols in a single request.
//! Optimized for sparkline rendering with only close prices by default.

use super::urls::api;
use crate::client::YahooClient;
use crate::constants::{Interval, TimeRange};
use crate::error::Result;
use tracing::info;

/// Fetch spark data for multiple symbols in a single request.
///
/// This endpoint is optimized for sparkline charts, returning only close prices
/// by default. It's more efficient than multiple chart requests when you need
/// simple price trends for many symbols.
///
/// # Arguments
///
/// * `client` - The Yahoo Finance client
/// * `symbols` - Slice of stock symbols (e.g., `["AAPL", "MSFT", "GOOGL"]`)
/// * `interval` - Time interval between data points
/// * `range` - Time range to fetch data for
///
/// # Example
///
/// ```ignore
/// use finance_query::{Interval, TimeRange};
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// # let client = finance_query::YahooClient::new(Default::default()).await?;
/// use finance_query::api::spark;
/// let spark_data = spark::fetch(&client, &["AAPL", "MSFT"], Interval::FiveMinutes, TimeRange::OneDay).await?;
/// # Ok(())
/// # }
/// ```
pub async fn fetch(
    client: &YahooClient,
    symbols: &[&str],
    interval: Interval,
    range: TimeRange,
) -> Result<serde_json::Value> {
    if symbols.is_empty() {
        return Err(crate::error::YahooError::InvalidParameter {
            param: "symbols".to_string(),
            reason: "At least one symbol is required".to_string(),
        });
    }

    // Validate all symbols
    for symbol in symbols {
        super::common::validate_symbol(symbol)?;
    }

    let symbols_joined = symbols.join(",");
    info!(
        "Fetching spark for {} symbols ({}, {})",
        symbols.len(),
        interval.as_str(),
        range.as_str()
    );

    let url = api::SPARK;
    let params = [
        ("symbols", symbols_joined.as_str()),
        ("interval", interval.as_str()),
        ("range", range.as_str()),
        ("indicators", "close"),
        ("includeTimestamps", "true"),
        ("includePrePost", "false"),
    ];
    let response = client.request_with_params(url, &params).await?;

    Ok(response.json().await?)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::client::ClientConfig;

    #[tokio::test]
    #[ignore = "requires network access"]
    async fn test_fetch_spark() {
        let client = YahooClient::new(ClientConfig::default()).await.unwrap();
        let result = fetch(
            &client,
            &["AAPL", "MSFT"],
            Interval::FiveMinutes,
            TimeRange::OneDay,
        )
        .await;
        assert!(result.is_ok());
        let json = result.unwrap();
        assert!(json.get("spark").is_some());
    }

    #[tokio::test]
    async fn test_empty_symbols() {
        let client = YahooClient::new(ClientConfig::default()).await.unwrap();
        let result = fetch(&client, &[], Interval::FiveMinutes, TimeRange::OneDay).await;
        assert!(result.is_err());
    }
}
