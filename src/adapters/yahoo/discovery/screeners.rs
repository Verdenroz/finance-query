use crate::adapters::yahoo::client::YahooClient;
use crate::constants::screeners::Screener;
use crate::error::Result;
use crate::models::discovery::screeners::{ScreenerField, ScreenerQuery, ScreenerResults};

/// Fetch data from a predefined Yahoo Finance screener.
///
/// Delegates to [`YahooClient::get_screener`] for the typed result.
pub async fn fetch(
    client: &YahooClient,
    screener_type: Screener,
    count: u32,
) -> Result<ScreenerResults> {
    client.get_screener(screener_type, count).await
}

/// Fetch data using a custom screener query.
///
/// Delegates to [`YahooClient::custom_screener`] for the typed result.
pub async fn fetch_custom<F: ScreenerField>(
    client: &YahooClient,
    query: ScreenerQuery<F>,
) -> Result<ScreenerResults> {
    client.custom_screener(query).await
}
