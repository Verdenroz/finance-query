/// News endpoint
///
/// Fetches recent news articles for specified symbols.
use crate::client::YahooClient;
use crate::constants::endpoints;
use crate::error::Result;
use tracing::info;

/// Fetch news articles for symbols
///
/// # Arguments
///
/// * `client` - The Yahoo Finance client
/// * `symbols` - Stock symbols to get news for
/// * `count` - Maximum number of articles to return
///
/// # Example
///
/// ```no_run
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// # let client = finance_query::YahooClient::new(Default::default()).await?;
/// use finance_query::endpoints::news;
/// let results = news::fetch(&client, &["AAPL", "MSFT"], 10).await?;
/// # Ok(())
/// # }
/// ```
pub async fn fetch(
    client: &YahooClient,
    symbols: &[&str],
    count: u32,
) -> Result<serde_json::Value> {
    super::common::validate_symbols(symbols)?;

    info!("Fetching news for: {:?}", symbols);

    let symbols_str = symbols.join(",");
    let params = [
        ("symbols", symbols_str.as_str()),
        ("count", &count.to_string()),
    ];

    let response = client.request_with_params(endpoints::NEWS, &params).await?;

    Ok(response.json().await?)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ClientConfig;

    #[tokio::test]
    #[ignore] // Requires network access
    async fn test_fetch_news() {
        let client = YahooClient::new(ClientConfig::default()).await.unwrap();
        let result = fetch(&client, &["AAPL"], 5).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_empty_symbols() {
        let client = YahooClient::new(ClientConfig::default()).await.unwrap();
        let result = fetch(&client, &[], 5).await;
        assert!(result.is_err());
    }
}
