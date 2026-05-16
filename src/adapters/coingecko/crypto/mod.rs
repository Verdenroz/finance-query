use crate::error::Result;
pub use super::models::CoinQuote;

/// Fetch the top `count` cryptocurrencies by market cap.
///
/// # Arguments
///
/// * `vs_currency` - Quote currency (e.g., `"usd"`, `"eur"`, `"btc"`)
/// * `count` - Number of coins to return (max 250)
pub async fn coins(vs_currency: &str, count: usize) -> Result<Vec<CoinQuote>> {
    super::client()?.coins(vs_currency, count).await
}

/// Fetch a single coin by its CoinGecko ID (e.g., `"bitcoin"`, `"ethereum"`).
///
/// # Arguments
///
/// * `id` - CoinGecko coin ID
/// * `vs_currency` - Quote currency (e.g., `"usd"`)
pub async fn coin(id: &str, vs_currency: &str) -> Result<CoinQuote> {
    super::client()?.coin(id, vs_currency).await
}
