use super::urls::api;
/// Market Summary endpoint
///
/// Fetch market summary from Yahoo Finance.
use crate::client::YahooClient;
use crate::constants::Country;
use crate::error::Result;
use tracing::info;

/// Fetch market summary from Yahoo Finance
///
/// # Arguments
///
/// * `client` - The Yahoo Finance client
/// * `country` - Optional country for localization. If None, uses client's configured lang/region.
///
/// # Example
///
/// ```no_run
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// # let client = finance_query::YahooClient::new(Default::default()).await?;
/// use finance_query::{endpoints::market_summary, Country};
/// // Use client's default config
/// let result = market_summary::fetch(&client, None).await?;
/// // Or specify a country
/// let result = market_summary::fetch(&client, Some(Country::Japan)).await?;
/// # Ok(())
/// # }
/// ```
pub async fn fetch(client: &YahooClient, country: Option<Country>) -> Result<serde_json::Value> {
    let (lang, region) = match country {
        Some(c) => (c.lang().to_string(), c.region().to_string()),
        None => (client.config().lang.clone(), client.config().region.clone()),
    };

    info!("Fetching market summary (lang={}, region={})", lang, region);

    let params = [("lang", lang.as_str()), ("region", region.as_str())];

    let response = client
        .request_with_params(api::MARKET_SUMMARY, &params)
        .await?;

    Ok(response.json().await?)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::client::ClientConfig;

    #[tokio::test]
    #[ignore] // Requires network access
    async fn test_fetch_market_summary() {
        let client = YahooClient::new(ClientConfig::default()).await.unwrap();
        let result = fetch(&client, None).await;
        assert!(result.is_ok());
        let json = result.unwrap();
        assert!(json.get("marketSummaryResponse").is_some());
    }
}
