/// Batch quotes endpoint
///
/// Fetches basic quote data for multiple symbols in a single request.
/// This uses the /v7/finance/quote endpoint which is more efficient for batch requests
/// than calling quoteSummary for each symbol individually.
use crate::adapters::yahoo::client::YahooClient;
use crate::adapters::yahoo::endpoints::api;
use crate::error::Result;
use crate::models::quote::{FormattedValue, Price, QuoteSummaryResponse};
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
/// ```ignore
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// # let client = finance_query::YahooClient::new(Default::default()).await?;
/// use finance_query::endpoints::quotes;
/// let quotes = quotes::fetch(&client, &["AAPL", "GOOGL", "MSFT"]).await?;
/// # Ok(())
/// # }
/// ```
#[allow(dead_code)]
pub(crate) async fn fetch(client: &YahooClient, symbols: &[&str]) -> Result<serde_json::Value> {
    crate::adapters::yahoo::common::validate_symbols(symbols)?;

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
/// ```ignore
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
    crate::adapters::yahoo::common::validate_symbols(symbols)?;

    info!(
        "Fetching batch quotes for {} symbols with custom fields (formatted={}, include_logo={})",
        symbols.len(),
        formatted,
        include_logo
    );

    // Get client config for lang and region (read once)
    let config = client.config();

    // Build parameters — static string literals avoid per-call allocations
    let mut params: Vec<(&str, std::borrow::Cow<str>)> = vec![
        ("symbols", symbols.join(",").into()),
        (
            "formatted",
            if formatted {
                "true".into()
            } else {
                "false".into()
            },
        ),
    ];

    // Add fields if specified
    if let Some(field_list) = fields {
        params.push(("fields", field_list.join(",").into()));
    }

    // Add logo parameters if requested
    if include_logo {
        params.push(("imgHeights", "50".into()));
        params.push(("imgWidths", "50".into()));
        params.push(("imgLabels", "logoUrl".into()));
    }

    // Add overnight price support
    params.push(("overnightPrice", "true".into()));

    params.push(("lang", (&*config.lang).into()));
    params.push(("region", (&*config.region).into()));

    let response = client.request_with_params(api::QUOTES, &params).await?;

    Ok(response.json().await?)
}

/// Fetch batch quotes and convert to canonical `(symbol, QuoteSummaryResponse)` pairs.
///
/// The batch endpoint returns basic fields only (price module), not full quoteSummary data.
/// This function constructs partial `QuoteSummaryResponse` objects from the batch response.
pub(crate) async fn fetch_quotes_batch(
    client: &YahooClient,
    symbols: &[&str],
) -> Result<Vec<(String, QuoteSummaryResponse)>> {
    let json = fetch(client, symbols).await?;
    let result = json
        .get("quoteResponse")
        .and_then(|qr| qr.get("result"))
        .and_then(|r| r.as_array());

    let mut quotes = Vec::new();
    if let Some(results) = result {
        for item in results {
            let symbol = item["symbol"].as_str().unwrap_or("").to_string();
            let price = Price {
                short_name: item["shortName"].as_str().map(String::from),
                long_name: item["longName"].as_str().map(String::from),
                exchange_name: item["fullExchangeName"].as_str().map(String::from),
                exchange: item["exchange"].as_str().map(String::from),
                quote_type: item["quoteType"].as_str().map(String::from),
                currency: item["currency"].as_str().map(String::from),
                market_state: item["marketState"].as_str().map(String::from),
                regular_market_price: item["regularMarketPrice"].as_f64().map(FormattedValue::new),
                regular_market_change: item["regularMarketChange"]
                    .as_f64()
                    .map(FormattedValue::new),
                regular_market_change_percent: item["regularMarketChangePercent"]
                    .as_f64()
                    .map(FormattedValue::new),
                regular_market_volume: item["regularMarketVolume"]
                    .as_i64()
                    .map(FormattedValue::new),
                regular_market_previous_close: item["regularMarketPreviousClose"]
                    .as_f64()
                    .map(FormattedValue::new),
                regular_market_open: item["regularMarketOpen"].as_f64().map(FormattedValue::new),
                regular_market_day_high: item["regularMarketDayHigh"]
                    .as_f64()
                    .map(FormattedValue::new),
                regular_market_day_low: item["regularMarketDayLow"]
                    .as_f64()
                    .map(FormattedValue::new),
                market_cap: item["marketCap"].as_i64().map(FormattedValue::new),
                ..Default::default()
            };
            let response = QuoteSummaryResponse {
                symbol: symbol.clone(),
                price: Some(price),
                ..Default::default()
            };
            quotes.push((symbol, response));
        }
    }
    Ok(quotes)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::adapters::yahoo::client::ClientConfig;

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
    #[ignore = "requires network access - validation tested in common::tests"]
    async fn test_empty_symbols() {
        let client = YahooClient::new(ClientConfig::default()).await.unwrap();
        let result = fetch(&client, &[]).await;
        assert!(result.is_err());
    }
}
