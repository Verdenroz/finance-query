/// Trending tickers endpoint
///
/// Fetch trending tickers for a region from Yahoo Finance.
use crate::adapters::yahoo::client::YahooClient;
use crate::constants::Region;
use crate::error::Result;
use crate::models::discovery::trending::TrendingQuote;

/// Fetch trending tickers for a region from Yahoo Finance.
///
/// Delegates to [`YahooClient::get_trending`] for the typed result.
pub async fn fetch(client: &YahooClient, region: Option<Region>) -> Result<Vec<TrendingQuote>> {
    client.get_trending(region).await
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::adapters::yahoo::client::ClientConfig;

    #[tokio::test]
    #[ignore] // Requires network access
    async fn test_fetch_trending() {
        let client = YahooClient::new(ClientConfig::default()).await.unwrap();
        let result = fetch(&client, None).await;
        assert!(result.is_ok());
    }
}
