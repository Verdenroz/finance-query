/// Batch quotes endpoint
///
/// Fetches basic quote data for multiple symbols in a single request.
/// This uses the /v7/finance/quote endpoint which is more efficient for batch requests
/// than calling quoteSummary for each symbol individually.
use crate::client::YahooClient;
use crate::constants::endpoints;
use crate::error::Result;
use tracing::info;

/// Fetch batch quotes for multiple symbols
///
/// This endpoint returns basic quote data (price, volume, market cap, etc.) for multiple
/// symbols in a single API call. It's more efficient than quoteSummary for batch requests.
///
/// # Arguments
///
/// * `client` - The Yahoo Finance client
/// * `symbols` - Array of stock symbols to fetch quotes for
///
/// # Example
///
/// ```no_run
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// # let client = finance_query::YahooClient::new(Default::default()).await?;
/// use finance_query::endpoints::quotes;
/// let quotes = quotes::fetch(&client, &["AAPL", "GOOGL", "MSFT"]).await?;
/// # Ok(())
/// # }
/// ```
pub async fn fetch(client: &YahooClient, symbols: &[&str]) -> Result<serde_json::Value> {
    super::common::validate_symbols(symbols)?;

    info!("Fetching batch quotes for {} symbols", symbols.len());

    let params = [("symbols", symbols.join(","))];
    let response = client
        .request_with_params(endpoints::QUOTES, &params)
        .await?;

    Ok(response.json().await?)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ClientConfig;

    #[tokio::test]
    #[ignore] // Requires network access
    async fn test_fetch_batch_quotes() {
        let client = YahooClient::new(ClientConfig::default()).await.unwrap();
        let result = fetch(&client, &["AAPL", "GOOGL"]).await;
        assert!(result.is_ok());
        let json = result.unwrap();
        assert!(json.get("quoteResponse").is_some());
    }

    #[tokio::test]
    async fn test_empty_symbols() {
        let client = YahooClient::new(ClientConfig::default()).await.unwrap();
        let result = fetch(&client, &[]).await;
        assert!(result.is_err());
    }
}
