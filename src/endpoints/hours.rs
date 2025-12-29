use super::urls::api;
use crate::client::YahooClient;
use crate::error::Result;
use crate::models::hours::MarketHours;

/// Fetch market hours/time data
///
/// Returns market status for various markets.
///
/// # Arguments
///
/// * `client` - Yahoo Finance client
/// * `region` - Optional region override (e.g., "US", "JP", "GB"). If None, uses client's configured region.
///
/// # Example
///
/// ```ignore
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// # let client = finance_query::YahooClient::new(Default::default()).await?;
/// // Use client's default region
/// let hours = client.get_hours(None).await?;
///
/// // Override with specific region
/// let jp_hours = client.get_hours(Some("JP")).await?;
/// # Ok(())
/// # }
/// ```
pub async fn fetch(client: &YahooClient, region: Option<&str>) -> Result<MarketHours> {
    let config = client.config();
    let region = region.unwrap_or(&config.region);

    let params = [
        ("formatted", "true"),
        ("key", "finance"),
        ("region", region),
    ];

    let response = client
        .request_with_params(api::MARKET_TIME, &params)
        .await?;
    let json: serde_json::Value = response.json().await?;

    parse_hours_response(&json)
}

/// Parse Yahoo Finance hours response into clean MarketHours
fn parse_hours_response(json: &serde_json::Value) -> Result<MarketHours> {
    MarketHours::from_response(json).map_err(|e| crate::error::YahooError::ResponseStructureError {
        field: "hours".to_string(),
        context: e,
    })
}
