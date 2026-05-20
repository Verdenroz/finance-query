/// Recommendations endpoint
///
/// Fetches similar/recommended quotes for a given symbol.
use crate::adapters::yahoo::client::YahooClient;
use crate::error::Result;
use crate::models::corporate::recommendation::SimilarSymbol;

/// Fetch recommendations and convert to canonical `SimilarSymbol` models.
///
/// Delegates to [`YahooClient::get_recommendations`] for the typed result.
pub async fn fetch(client: &YahooClient, symbol: &str, limit: u32) -> Result<Vec<SimilarSymbol>> {
    client.get_recommendations(symbol, limit).await
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::adapters::yahoo::client::ClientConfig;

    #[tokio::test]
    #[ignore] // Requires network access
    async fn test_fetch_recommendations() {
        let client = YahooClient::new(ClientConfig::default()).await.unwrap();
        let result = fetch(&client, "AAPL", 5).await;
        assert!(result.is_ok());
        let sims = result.unwrap();
        assert!(!sims.is_empty(), "Should have at least one similar symbol");
    }

    #[tokio::test]
    #[ignore = "requires network access - validation tested in common::tests"]
    async fn test_empty_symbol() {
        let client = YahooClient::new(ClientConfig::default()).await.unwrap();
        let result = fetch(&client, "", 5).await;
        assert!(result.is_err());
    }
}
