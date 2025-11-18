use crate::client::YahooClient;
use crate::error::Result;
use tracing::info;

/// Fetch quote summary data for a symbol from Yahoo Finance
///
/// This returns the raw JSON response from Yahoo Finance's quoteSummary endpoint.
/// The response includes modules: assetProfile, price, summaryDetail, defaultKeyStatistics,
/// calendarEvents, and quoteUnadjustedPerformanceOverview.
///
/// # Arguments
///
/// * `client` - The Yahoo Finance client
/// * `symbol` - The stock symbol (e.g., "AAPL", "NVDA")
///
/// # Example
///
/// ```no_run
/// use finance_query::{YahooClient, ClientConfig, endpoints::fetch_quote_summary};
///
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let client = YahooClient::new(ClientConfig::default()).await?;
/// let quote = fetch_quote_summary(&client, "AAPL").await?;
/// println!("{}", serde_json::to_string_pretty(&quote)?);
/// # Ok(())
/// # }
/// ```
pub async fn fetch_quote_summary(client: &YahooClient, symbol: &str) -> Result<serde_json::Value> {
    info!("Fetching quote summary for symbol: {}", symbol);

    // Construct the URL
    let url = crate::constants::endpoints::quote_summary(symbol);

    // Define the modules we want to fetch
    let modules = "assetProfile,price,summaryDetail,defaultKeyStatistics,calendarEvents,quoteUnadjustedPerformanceOverview";

    // Build full URL with modules parameter
    let full_url = format!("{}?modules={}", url, modules);

    // Make the request
    let response = client.request_with_crumb(&full_url).await?;

    // Parse JSON response
    let json = response.json::<serde_json::Value>().await?;

    Ok(json)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{ClientConfig, YahooClient};

    #[tokio::test]
    #[ignore] // Ignore by default as it makes real network requests
    async fn test_fetch_quote_summary() {
        let client = YahooClient::new(ClientConfig::default())
            .await
            .expect("Failed to create client");

        let result = fetch_quote_summary(&client, "AAPL").await;
        assert!(result.is_ok());

        let json = result.unwrap();
        // Check that we got a valid response structure
        assert!(json.get("quoteSummary").is_some());
    }
}
