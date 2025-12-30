use super::urls::api;
/// Trending tickers endpoint
///
/// Fetch trending tickers for a region from Yahoo Finance.
use crate::client::YahooClient;
use crate::constants::Region;
use crate::error::Result;
use tracing::info;

/// Fetch trending tickers for a region from Yahoo Finance
///
/// # Arguments
///
/// * `client` - The Yahoo Finance client
/// * `region` - Optional region for localization. If None, uses client's configured lang/region.
///
/// # Example
///
/// ```ignore
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// # let client = finance_query::YahooClient::new(Default::default()).await?;
/// use finance_query::{api::trending, Region};
/// // Use client's default config
/// let result = trending::fetch(&client, None).await?;
/// // Or specify a region
/// let result = trending::fetch(&client, Some(Region::Japan)).await?;
/// # Ok(())
/// # }
/// ```
pub async fn fetch(client: &YahooClient, region: Option<Region>) -> Result<serde_json::Value> {
    let (lang, region) = match region {
        Some(c) => (c.lang().to_string(), c.region().to_string()),
        None => (client.config().lang.clone(), client.config().region.clone()),
    };

    info!(
        "Fetching trending tickers (lang={}, region={})",
        lang, region
    );

    let url = api::trending(&region);
    let params = [("lang", lang.as_str()), ("region", region.as_str())];

    let response = client.request_with_params(&url, &params).await?;

    Ok(response.json().await?)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::client::ClientConfig;

    #[tokio::test]
    #[ignore] // Requires network access
    async fn test_fetch_trending() {
        let client = YahooClient::new(ClientConfig::default()).await.unwrap();
        let result = fetch(&client, None).await;
        assert!(result.is_ok());
        let json = result.unwrap();
        assert!(json.get("finance").is_some());
    }
}
