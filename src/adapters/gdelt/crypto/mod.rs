//! `CRYPTO` capability for GDELT — market-wide crypto news only. GDELT has
//! no per-coin vocabulary, so this isn't scoped to a specific asset the way
//! `Ticker::news` is scoped to a symbol.

use crate::error::Result;
use crate::models::corporate::news::News;

const QUERY: &str = "bitcoin OR ethereum OR cryptocurrency";

/// Lookback window, matching the symbol-news adapter's tradeoff between
/// freshness and still returning results on a quiet day.
const TIMESPAN: &str = "2w";

/// Fetch market-wide crypto news, newest first.
pub async fn fetch_crypto_news_response(limit: u32) -> Result<Vec<News>> {
    let response = super::client()?
        .article_search(QUERY, TIMESPAN, limit.min(250))
        .await?;
    let now = chrono::Utc::now();
    Ok(response
        .articles
        .into_iter()
        .map(|a| super::corporate::to_news_at(a, now))
        .collect())
}
