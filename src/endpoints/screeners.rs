use crate::client::YahooClient;
use crate::constants::screener_types::ScreenerType;
use crate::constants::url_builders;
use crate::error::Result;
use crate::models::screeners::ScreenersResponse;

/// Fetch data from a predefined Yahoo Finance screener
///
/// Returns a flattened, user-friendly response structure with quotes
/// matching the screener criteria.
///
/// # Arguments
///
/// * `client` - Yahoo Finance client
/// * `screener_type` - The predefined screener type to use
/// * `count` - Number of results to return (max 250)
///
/// # Example
///
/// ```no_run
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// # let client = finance_query::YahooClient::new(Default::default()).await?;
/// use finance_query::ScreenerType;
/// let result = client.get_screener(ScreenerType::MostActives, 25).await?;
/// println!("Screener type: {}", result.screener_type);
/// for quote in &result.quotes {
///     println!("  {} - {}", quote.symbol, quote.short_name);
/// }
/// # Ok(())
/// # }
/// ```
pub async fn fetch(
    client: &YahooClient,
    screener_type: ScreenerType,
    count: u32,
) -> Result<ScreenersResponse> {
    let url = url_builders::screener(screener_type, count);
    let response = client.request_with_crumb(&url).await?;
    let json: serde_json::Value = response.json().await?;

    // Parse and flatten Yahoo Finance response internally
    parse_screeners_response(&json)
}

/// Parse Yahoo Finance screener response into clean ScreenersResponse
///
/// Handles all Yahoo-specific nested structure and data transformation internally.
fn parse_screeners_response(json: &serde_json::Value) -> Result<ScreenersResponse> {
    ScreenersResponse::from_response(json).map_err(|e| {
        crate::error::YahooError::ResponseStructureError {
            field: "screeners".to_string(),
            context: e,
        }
    })
}
