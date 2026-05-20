/// Market Summary endpoint
///
/// Fetch market summary from Yahoo Finance.
use crate::adapters::yahoo::client::YahooClient;
use crate::constants::Region;
use crate::error::Result;
use crate::models::market::market_summary::MarketSummaryQuote;

/// Fetch market summary from Yahoo Finance.
///
/// Delegates to [`YahooClient::get_market_summary`] for the typed result.
pub async fn fetch(
    client: &YahooClient,
    region: Option<Region>,
) -> Result<Vec<MarketSummaryQuote>> {
    client.get_market_summary(region).await
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::adapters::yahoo::client::ClientConfig;

    #[tokio::test]
    #[ignore] // Requires network access
    async fn test_fetch_market_summary() {
        let client = YahooClient::new(ClientConfig::default()).await.unwrap();
        let result = fetch(&client, None).await;
        assert!(result.is_ok());
        let quotes = result.unwrap();
        assert!(!quotes.is_empty(), "Should have at least one quote");
    }
}
