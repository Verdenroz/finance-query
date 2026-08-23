//! `FOREX` capability for GDELT — market-wide forex news only. GDELT has no
//! per-pair vocabulary, so this can't be scoped to e.g. EUR/USD specifically,
//! only broad currency-market news.

use crate::error::Result;
use crate::models::corporate::news::News;

const QUERY: &str = "forex OR \"foreign exchange\" OR currency";

/// Lookback window, matching the symbol-news adapter's tradeoff between
/// freshness and still returning results on a quiet day.
const TIMESPAN: &str = "2w";

/// Fetch market-wide forex news, newest first.
pub async fn fetch_forex_news_response(limit: u32) -> Result<Vec<News>> {
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
