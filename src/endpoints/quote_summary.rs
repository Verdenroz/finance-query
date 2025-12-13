/// Quote summary endpoint
///
/// Fetches comprehensive quote summary data with customizable modules.
use crate::client::YahooClient;
use crate::constants::{api_params, endpoints};
use crate::error::Result;
use tracing::info;

/// Fetch quote summary with specified modules
///
/// # Arguments
///
/// * `client` - The Yahoo Finance client
/// * `symbol` - Stock symbol
/// * `modules` - List of module names to fetch (e.g., "price", "summaryDetail")
///
/// # Available Modules
///
/// Common modules include: assetProfile, price, summaryDetail, defaultKeyStatistics,
/// calendarEvents, quoteUnadjustedPerformanceOverview, financialData, earningsHistory,
/// earningsTrend, industryTrend, indexTrend, sectorTrend, etc.
///
/// # Example
///
/// ```no_run
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// # let client = finance_query::YahooClient::new(Default::default()).await?;
/// use finance_query::endpoints::quote_summary;
/// let modules = vec!["price", "summaryDetail"];
/// let summary = quote_summary::fetch(&client, "AAPL", &modules).await?;
/// # Ok(())
/// # }
/// ```
pub async fn fetch(
    client: &YahooClient,
    symbol: &str,
    modules: &[&str],
) -> Result<serde_json::Value> {
    super::common::validate_symbol(symbol)?;

    if modules.is_empty() {
        return Err(crate::error::YahooError::InvalidParameter {
            param: "modules".to_string(),
            reason: "No modules provided for quote summary".to_string(),
        });
    }

    info!(
        "Fetching quote summary for {} with {} modules",
        symbol,
        modules.len()
    );

    let url = endpoints::quote_summary(symbol);

    // Get client config for lang and region
    let config = client.config();
    let params = [
        ("modules", modules.join(",")),
        ("formatted", api_params::FORMATTED.to_string()),
        ("lang", config.lang.clone()),
        ("region", config.region.clone()),
    ];
    let response = client.request_with_params(&url, &params).await?;

    Ok(response.json().await?)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ClientConfig;

    #[tokio::test]
    #[ignore] // Requires network access
    async fn test_fetch_quote_summary() {
        let client = YahooClient::new(ClientConfig::default()).await.unwrap();
        let result = fetch(&client, "AAPL", &["price", "summaryDetail"]).await;
        assert!(result.is_ok());
        let json = result.unwrap();
        assert!(json.get("quoteSummary").is_some());
    }

    #[tokio::test]
    async fn test_empty_modules() {
        let client = YahooClient::new(ClientConfig::default()).await.unwrap();
        let result = fetch(&client, "AAPL", &[]).await;
        assert!(result.is_err());
    }
}
