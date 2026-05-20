use crate::adapters::yahoo::client::YahooClient;
use crate::error::Result;
use crate::models::market::hours::MarketHours;

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
/// Delegates to [`YahooClient::get_hours`] for the typed result.
pub async fn fetch(client: &YahooClient, region: Option<&str>) -> Result<MarketHours> {
    client.get_hours(region).await
}
