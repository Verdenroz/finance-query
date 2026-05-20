/// Options endpoint
///
/// Fetches options chain data including calls, puts, strikes, and expirations.
use crate::adapters::yahoo::client::YahooClient;
use crate::error::Result;

/// Fetch options chain and return a canonical `Options` model.
///
/// Delegates to [`YahooClient::get_options`] for the typed result.
pub async fn fetch(
    client: &YahooClient,
    symbol: &str,
    date: Option<i64>,
) -> Result<crate::models::options::Options> {
    client.get_options(symbol, date).await
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::adapters::yahoo::client::ClientConfig;

    #[tokio::test]
    #[ignore] // Requires network access
    async fn test_fetch_options() {
        let client = YahooClient::new(ClientConfig::default()).await.unwrap();
        let result = fetch(&client, "AAPL", None).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    #[ignore = "requires network access - validation tested in common::tests"]
    async fn test_empty_symbol() {
        let client = YahooClient::new(ClientConfig::default()).await.unwrap();
        let result = fetch(&client, "", None).await;
        assert!(result.is_err());
    }
}
