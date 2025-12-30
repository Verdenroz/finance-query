use super::urls::api;
/// Quote type endpoint
///
/// Fetches quote type data including company ID (quartrId).
use crate::client::YahooClient;
use crate::error::Result;
use tracing::info;

/// Fetch quote type data including company ID (quartrId)
///
/// # Arguments
///
/// * `client` - The Yahoo Finance client
/// * `symbol` - Stock symbol
///
/// # Example
///
/// ```ignore
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// # let client = finance_query::YahooClient::new(Default::default()).await?;
/// use finance_query::api::quote_type;
/// let quote_type = quote_type::fetch(&client, "AAPL").await?;
/// # Ok(())
/// # }
/// ```
#[allow(dead_code)]
pub(crate) async fn fetch(client: &YahooClient, symbol: &str) -> Result<serde_json::Value> {
    super::common::validate_symbol(symbol)?;

    info!("Fetching quote type for: {}", symbol);

    let url = api::quote_type(symbol);
    let response = client.request_with_crumb(&url).await?;

    Ok(response.json().await?)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::client::ClientConfig;

    #[tokio::test]
    #[ignore] // Requires network access
    async fn test_fetch_quote_type() {
        let client = YahooClient::new(ClientConfig::default()).await.unwrap();
        let result = fetch(&client, "AAPL").await;
        assert!(result.is_ok());
        let json = result.unwrap();
        assert!(json.get("quoteType").is_some());
    }

    #[tokio::test]
    #[ignore = "requires network access - validation tested in common::tests"]
    async fn test_empty_symbol() {
        let client = YahooClient::new(ClientConfig::default()).await.unwrap();
        let result = fetch(&client, "").await;
        assert!(result.is_err());
    }
}
