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
/// * `symbols` - Stock symbols to get news for (empty array for general market news)
/// * `count` - Maximum number of articles to return
///
/// # Example
///
/// ```no_run
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// # let client = finance_query::YahooClient::new(Default::default()).await?;
/// use finance_query::endpoints::news;
/// // Symbol-specific news
/// let results = news::fetch(&client, &["AAPL", "MSFT"], 10).await?;
/// // General market news
/// let general = news::fetch(&client, &[], 10).await?;
/// # Ok(())
/// # }
/// ```
pub async fn fetch(
    client: &YahooClient,
    symbols: &[&str],
    count: u32,
) -> Result<serde_json::Value> {
    if symbols.is_empty() {
        info!("Fetching general market news");
    } else {
        info!("Fetching news for: {:?}", symbols);
    }

    let count_str = count.to_string();
    let response = if symbols.is_empty() {
        // General market news (no symbols parameter)
        let params = [("count", count_str.as_str())];
        client.request_with_params(endpoints::NEWS, &params).await?
    } else {
        // Symbol-specific news
        let symbols_str = symbols.join(",");
        let params = [
            ("symbols", symbols_str.as_str()),
            ("count", count_str.as_str()),
        ];
        client.request_with_params(endpoints::NEWS, &params).await?
    };

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
    #[ignore] // Requires network access
    async fn test_general_market_news() {
        let client = YahooClient::new(ClientConfig::default()).await.unwrap();
        let result = fetch(&client, &[], 5).await;
        assert!(
            result.is_ok(),
            "Empty symbols should fetch general market news"
        );
    }
}
