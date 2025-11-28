use crate::client::YahooClient;
use crate::constants::url_builders;
use crate::error::Result;
use crate::models::movers::MoversResponse;

/// Fetch market movers (gainers, losers, actives)
///
/// # Arguments
///
/// * `client` - Yahoo Finance client
/// * `screener_id` - The screener ID (DAY_GAINERS, DAY_LOSERS, MOST_ACTIVES)
/// * `count` - Number of results to return
///
/// # Example
///
/// ```no_run
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// # let client = finance_query::YahooClient::new(Default::default()).await?;
/// use finance_query::constants::screener_ids;
/// let movers = client.get_movers(screener_ids::MOST_ACTIVES, 25).await?;
/// for result in &movers.finance.result {
///     println!("Screener: {}", result.canonical_name);
///     for quote in &result.quotes {
///         println!("  {} - {}", quote.symbol, quote.short_name);
///     }
/// }
/// # Ok(())
/// # }
/// ```
pub async fn fetch(
    client: &YahooClient,
    screener_id: &str,
    count: u32,
) -> Result<MoversResponse> {
    let url = url_builders::movers(screener_id, count);

    let response = client.request_with_crumb(&url).await?;

    let movers: MoversResponse = response.json().await?;

    Ok(movers)
}
