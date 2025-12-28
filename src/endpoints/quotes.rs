use super::urls::api;
/// Batch quotes endpoint
///
/// Fetches basic quote data for multiple symbols in a single request.
/// This uses the /v7/finance/quote endpoint which is more efficient for batch requests
/// than calling quoteSummary for each symbol individually.
use crate::client::YahooClient;
use crate::error::Result;
use tracing::info;

/// Fetch batch quotes for multiple symbols
///
/// This endpoint returns basic quote data (price, volume, market cap, etc.) for multiple
/// symbols in a single API call. It's more efficient than quoteSummary for batch requests.
///
/// # Arguments
///
/// * `client` - The Yahoo Finance client
/// * `symbols` - Array of stock symbols to fetch quotes for
///
/// # Example
///
/// ```no_run
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// # let client = finance_query::YahooClient::new(Default::default()).await?;
/// use finance_query::endpoints::quotes;
/// let quotes = quotes::fetch(&client, &["AAPL", "GOOGL", "MSFT"]).await?;
/// # Ok(())
/// # }
/// ```
#[allow(dead_code)]
pub(crate) async fn fetch(client: &YahooClient, symbols: &[&str]) -> Result<serde_json::Value> {
    super::common::validate_symbols(symbols)?;

    info!("Fetching batch quotes for {} symbols", symbols.len());

    let params = [("symbols", symbols.join(","))];
    let response = client.request_with_params(api::QUOTES, &params).await?;

    Ok(response.json().await?)
}

/// Fetch batch quotes with custom fields and options
///
/// This advanced version allows you to specify which fields to fetch and whether to include
/// logo URLs with specific dimensions. Supports selective field fetching for efficiency.
///
/// # Arguments
///
/// * `client` - The Yahoo Finance client
/// * `symbols` - Array of stock symbols to fetch quotes for
/// * `fields` - Optional list of specific fields to fetch (e.g., ["logoUrl", "regularMarketPrice"])
/// * `formatted` - Whether to return formatted values (e.g., "102,05 %" vs 1.0205495)
/// * `include_logo` - Whether to include logo URLs with 50x50 dimensions
///
/// # Available Fields
///
/// Common fields include: `logoUrl`, `companyLogoUrl`, `longName`, `shortName`,
/// `regularMarketPrice`, `regularMarketChange`, `regularMarketChangePercent`,
/// `regularMarketVolume`, `marketCap`, `fiftyTwoWeekHigh`, `fiftyTwoWeekLow`,
/// `preMarketPrice`, `postMarketPrice`, `quartrId`, and many more.
///
/// # Example
///
/// ```no_run
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// # let client = finance_query::YahooClient::new(Default::default()).await?;
/// use finance_query::endpoints::quotes;
///
/// // Fetch specific fields with logos
/// let fields = vec!["logoUrl", "longName", "regularMarketPrice", "marketCap"];
/// let quotes = quotes::fetch_with_fields(
///     &client,
///     &["AAPL", "TSLA", "NVDA"],
///     Some(&fields),
///     true,  // formatted values
///     true   // include logos
/// ).await?;
/// # Ok(())
/// # }
/// ```
#[allow(dead_code)]
pub(crate) async fn fetch_with_fields(
    client: &YahooClient,
    symbols: &[&str],
    fields: Option<&[&str]>,
    formatted: bool,
    include_logo: bool,
) -> Result<serde_json::Value> {
    super::common::validate_symbols(symbols)?;

    info!(
        "Fetching batch quotes for {} symbols with custom fields (formatted={}, include_logo={})",
        symbols.len(),
        formatted,
        include_logo
    );

    // Build parameters
    let mut params = vec![
        ("symbols", symbols.join(",")),
        ("formatted", formatted.to_string()),
    ];

    // Add fields if specified
    if let Some(field_list) = fields {
        params.push(("fields", field_list.join(",")));
    }

    // Add logo parameters if requested
    if include_logo {
        params.push(("imgHeights", "50".to_string()));
        params.push(("imgWidths", "50".to_string()));
        params.push(("imgLabels", "logoUrl".to_string()));
    }

    // Add overnight price support
    params.push(("overnightPrice", "true".to_string()));

    // Get client config for lang and region
    let config = client.config();
    params.push(("lang", config.lang.clone()));
    params.push(("region", config.region.clone()));

    let response = client.request_with_params(api::QUOTES, &params).await?;

    Ok(response.json().await?)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::client::ClientConfig;

    #[tokio::test]
    #[ignore] // Requires network access
    async fn test_fetch_batch_quotes() {
        let client = YahooClient::new(ClientConfig::default()).await.unwrap();
        let result = fetch(&client, &["AAPL", "GOOGL"]).await;
        assert!(result.is_ok());
        let json = result.unwrap();
        assert!(json.get("quoteResponse").is_some());
    }

    #[tokio::test]
    async fn test_empty_symbols() {
        let client = YahooClient::new(ClientConfig::default()).await.unwrap();
        let result = fetch(&client, &[]).await;
        assert!(result.is_err());
    }
}
