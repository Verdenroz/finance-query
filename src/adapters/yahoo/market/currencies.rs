/// Currencies endpoint
///
/// Fetch available currencies from Yahoo Finance.
use crate::adapters::yahoo::client::YahooClient;
use crate::error::Result;
use crate::models::market::currencies::Currency;

/// Fetch currencies from Yahoo Finance.
///
/// Delegates to [`YahooClient::get_currencies`] for the typed result.
pub async fn fetch(client: &YahooClient) -> Result<Vec<Currency>> {
    client.get_currencies().await
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::adapters::yahoo::client::ClientConfig;

    #[tokio::test]
    #[ignore] // Requires network access
    async fn test_fetch_currencies() {
        let client = YahooClient::new(ClientConfig::default()).await.unwrap();
        let result = fetch(&client).await;
        assert!(result.is_ok());
    }
}
