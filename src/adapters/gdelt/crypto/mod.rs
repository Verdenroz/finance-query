//! GDELT market-wide crypto news (no per-coin vocabulary).

use crate::error::Result;
use crate::models::corporate::news::News;

const QUERY: &str = "bitcoin OR ethereum OR cryptocurrency";

const TIMESPAN: &str = "2w";

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
