use super::urls::api;
/// Recommendations endpoint
///
/// Fetches similar/recommended quotes for a given symbol.
use crate::client::YahooClient;
use crate::error::Result;
use tracing::info;

/// Get similar/recommended quotes for a symbol
///
/// # Arguments
///
/// * `client` - The Yahoo Finance client
/// * `symbol` - Stock symbol to get recommendations for
/// * `limit` - Maximum number of recommendations to return
///
/// # Example
///
/// ```ignore
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// # let client = finance_query::YahooClient::new(Default::default()).await?;
/// use finance_query::api::recommendations;
/// let similar = recommendations::fetch(&client, "AAPL", 5).await?;
/// # Ok(())
/// # }
/// ```
pub async fn fetch(client: &YahooClient, symbol: &str, limit: u32) -> Result<serde_json::Value> {
    super::common::validate_symbol(symbol)?;

    info!("Fetching similar quotes for: {}", symbol);

    let url = api::recommendations(symbol);
    let params = [("count", limit.to_string())];
    let response = client.request_with_params(&url, &params).await?;

    Ok(response.json().await?)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::client::ClientConfig;

    #[tokio::test]
    #[ignore] // Requires network access
    async fn test_fetch_recommendations() {
        let client = YahooClient::new(ClientConfig::default()).await.unwrap();
        let result = fetch(&client, "AAPL", 5).await;
        assert!(result.is_ok());
        let json = result.unwrap();
        assert!(json.get("finance").is_some());
    }

    #[tokio::test]
    #[ignore = "requires network access - validation tested in common::tests"]
    async fn test_empty_symbol() {
        let client = YahooClient::new(ClientConfig::default()).await.unwrap();
        let result = fetch(&client, "", 5).await;
        assert!(result.is_err());
    }
}
