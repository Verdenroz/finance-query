use super::urls::api;
/// Currencies endpoint
///
/// Fetch available currencies from Yahoo Finance.
use crate::client::YahooClient;
use crate::error::Result;
use tracing::info;

/// Fetch currencies from Yahoo Finance
///
/// # Arguments
///
/// * `client` - The Yahoo Finance client
///
/// # Example
///
/// ```no_run
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// # let client = finance_query::YahooClient::new(Default::default()).await?;
/// use finance_query::endpoints::currencies;
/// let result = currencies::fetch(&client).await?;
/// # Ok(())
/// # }
/// ```
pub async fn fetch(client: &YahooClient) -> Result<serde_json::Value> {
    let config = client.config();
    info!(
        "Fetching currencies (lang={}, region={})",
        config.lang, config.region
    );

    let params = [
        ("lang", config.lang.as_str()),
        ("region", config.region.as_str()),
    ];

    let response = client.request_with_params(api::CURRENCIES, &params).await?;

    Ok(response.json().await?)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::client::ClientConfig;

    #[tokio::test]
    #[ignore] // Requires network access
    async fn test_fetch_currencies() {
        let client = YahooClient::new(ClientConfig::default()).await.unwrap();
        let result = fetch(&client).await;
        assert!(result.is_ok());
        let json = result.unwrap();
        assert!(json.get("currencies").is_some());
    }
}
