use super::urls::api;
/// Options endpoint
///
/// Fetches options chain data including calls, puts, strikes, and expirations.
use crate::client::YahooClient;
use crate::error::Result;
use tracing::info;

/// Fetch options chain for a symbol
///
/// # Arguments
///
/// * `client` - The Yahoo Finance client
/// * `symbol` - Stock symbol to get options for
/// * `date` - Optional expiration date (Unix timestamp)
///
/// # Example
///
/// ```no_run
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// # let client = finance_query::YahooClient::new(Default::default()).await?;
/// use finance_query::api::options;
/// let results = options::fetch(&client, "AAPL", None).await?;
/// # Ok(())
/// # }
/// ```
pub async fn fetch(
    client: &YahooClient,
    symbol: &str,
    date: Option<i64>,
) -> Result<serde_json::Value> {
    super::common::validate_symbol(symbol)?;

    info!("Fetching options for: {}", symbol);

    let url = api::options(symbol);

    let response = if let Some(date) = date {
        let params = [("date", date.to_string())];
        client.request_with_params(&url, &params).await?
    } else {
        client.request_with_crumb(&url).await?
    };

    Ok(response.json().await?)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::client::ClientConfig;

    #[tokio::test]
    #[ignore] // Requires network access
    async fn test_fetch_options() {
        let client = YahooClient::new(ClientConfig::default()).await.unwrap();
        let result = fetch(&client, "AAPL", None).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_empty_symbol() {
        let client = YahooClient::new(ClientConfig::default()).await.unwrap();
        let result = fetch(&client, "", None).await;
        assert!(result.is_err());
    }
}
