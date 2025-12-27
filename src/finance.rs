//! Non-symbol-specific Yahoo Finance operations
//!
//! This module provides functions for operations that don't require a specific stock symbol,
//! such as searching for symbols and getting market movers.

use crate::client::{ClientConfig, YahooClient};
use crate::error::Result;
use crate::models::movers::MoversResponse;
use crate::models::search::SearchResponse;
use serde_json::Value;

/// Search for stock symbols and companies
///
/// # Arguments
///
/// * `query` - Search term (company name, symbol, etc.)
/// * `limit` - Maximum number of results to return
///
/// # Examples
///
/// ```no_run
/// use finance_query::finance;
///
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let results = finance::search("Apple", 10).await?;
/// println!("Found {} results", results.result_count());
/// # Ok(())
/// # }
/// ```
pub async fn search(query: &str, limit: u32) -> Result<SearchResponse> {
    let client = YahooClient::new(ClientConfig::default()).await?;
    client.search(query, limit).await
}

/// Get top gaining stocks
///
/// # Arguments
///
/// * `count` - Number of gainers to return
///
/// # Examples
///
/// ```no_run
/// use finance_query::finance;
///
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let gainers = finance::gainers(10).await?;
/// println!("Top gainers: {:#?}", gainers);
/// # Ok(())
/// # }
/// ```
pub async fn gainers(count: u32) -> Result<MoversResponse> {
    let client = YahooClient::new(ClientConfig::default()).await?;
    client.get_movers("DAY_GAINERS", count).await
}

/// Get top losing stocks
///
/// # Arguments
///
/// * `count` - Number of losers to return
///
/// # Examples
///
/// ```no_run
/// use finance_query::finance;
///
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let losers = finance::losers(10).await?;
/// println!("Top losers: {:#?}", losers);
/// # Ok(())
/// # }
/// ```
pub async fn losers(count: u32) -> Result<MoversResponse> {
    let client = YahooClient::new(ClientConfig::default()).await?;
    client.get_movers("DAY_LOSERS", count).await
}

/// Get most active stocks by volume
///
/// # Arguments
///
/// * `count` - Number of active stocks to return
///
/// # Examples
///
/// ```no_run
/// use finance_query::finance;
///
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let actives = finance::actives(25).await?;
/// println!("Most active: {:#?}", actives);
/// # Ok(())
/// # }
/// ```
pub async fn actives(count: u32) -> Result<MoversResponse> {
    let client = YahooClient::new(ClientConfig::default()).await?;
    client.get_movers("MOST_ACTIVES", count).await
}

/// Get general market news
///
/// # Examples
///
/// ```no_run
/// use finance_query::finance;
///
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let news = finance::news().await?;
/// for article in news {
///     println!("{}: {}", article.source, article.title);
/// }
/// # Ok(())
/// # }
/// ```
pub async fn news() -> Result<Vec<crate::models::news::News>> {
    crate::scrapers::stockanalysis::scrape_general_news().await
}

/// Get earnings transcript
///
/// # Arguments
///
/// * `event_id` - Event ID for the earnings call
/// * `company_id` - Company ID
///
/// # Examples
///
/// ```no_run
/// use finance_query::finance;
///
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let transcript = finance::earnings_transcript("event123", "company456").await?;
/// # Ok(())
/// # }
/// ```
pub async fn earnings_transcript(event_id: &str, company_id: &str) -> Result<Value> {
    let client = YahooClient::new(ClientConfig::default()).await?;
    client.get_earnings_transcript(event_id, company_id).await
}

/// Get market hours/status
///
/// Returns the current status for various markets.
///
/// # Arguments
///
/// * `region` - Optional region override (e.g., "US", "JP", "GB"). If None, uses default (US).
///
/// # Examples
///
/// ```no_run
/// use finance_query::finance;
///
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// // Get US market hours (default)
/// let hours = finance::hours(None).await?;
///
/// // Get Japan market hours
/// let jp_hours = finance::hours(Some("JP")).await?;
/// # Ok(())
/// # }
/// ```
pub async fn hours(region: Option<&str>) -> Result<crate::models::hours::HoursResponse> {
    let client = YahooClient::new(ClientConfig::default()).await?;
    client.get_hours(region).await
}

/// Get world market indices quotes
///
/// Returns quotes for major world indices, optionally filtered by region.
///
/// # Arguments
///
/// * `region` - Optional region filter. If None, returns all world indices.
///
/// # Examples
///
/// ```no_run
/// use finance_query::{finance, IndicesRegion};
///
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// // Get all world indices
/// let all = finance::indices(None).await?;
/// println!("Fetched {} indices", all.success_count());
///
/// // Get only Americas indices
/// let americas = finance::indices(Some(IndicesRegion::Americas)).await?;
/// # Ok(())
/// # }
/// ```
pub async fn indices(
    region: Option<crate::constants::indices::Region>,
) -> Result<crate::tickers::BatchQuotesResponse> {
    use crate::Tickers;
    use crate::constants::indices::all_symbols;

    let symbols: Vec<&str> = match region {
        Some(r) => r.symbols().to_vec(),
        None => all_symbols(),
    };

    let tickers = Tickers::new(symbols).await?;
    tickers.quotes(false).await
}
