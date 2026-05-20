//! Stock news endpoints with sentiment analysis.

use serde::{Deserialize, Serialize};

use crate::Provider;
use crate::error::Result;
use crate::models::corporate::news::News;

use super::super::build_client;
use super::super::models::PaginatedResponseDTO;

/// Publisher information.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct Publisher {
    /// Publisher name.
    pub name: Option<String>,
    /// Homepage URL.
    pub homepage_url: Option<String>,
    /// Logo URL.
    pub logo_url: Option<String>,
    /// Favicon URL.
    pub favicon_url: Option<String>,
}

/// Sentiment insight.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct InsightDTO {
    /// Ticker symbol.
    pub ticker: Option<String>,
    /// Sentiment label (e.g., `"positive"`, `"negative"`, `"neutral"`).
    pub sentiment: Option<String>,
    /// Sentiment reasoning.
    pub sentiment_reasoning: Option<String>,
}

/// News article.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct NewsArticle {
    /// Article ID.
    pub id: Option<String>,
    /// Publisher.
    pub publisher: Option<Publisher>,
    /// Article title.
    pub title: Option<String>,
    /// Author.
    pub author: Option<String>,
    /// Published UTC timestamp.
    pub published_utc: Option<String>,
    /// Article URL.
    pub article_url: Option<String>,
    /// Image URL.
    pub image_url: Option<String>,
    /// Description.
    pub description: Option<String>,
    /// Keywords.
    pub keywords: Option<Vec<String>>,
    /// Related tickers.
    pub tickers: Option<Vec<String>>,
    /// AMP URL.
    pub amp_url: Option<String>,
    /// Sentiment insights.
    pub insights: Option<Vec<InsightDTO>>,
}

/// Fetch news articles, optionally filtered by ticker.
///
/// * `params` - Query params: `ticker`, `published_utc`, `order`, `limit`, `sort`
pub async fn stock_news(params: &[(&str, &str)]) -> Result<PaginatedResponseDTO<NewsArticle>> {
    let client = build_client()?;
    client.get("/v2/reference/news", params).await
}

/// Fetch news (canonical) for a stock ticker.
pub async fn fetch_news_response(symbol: &str) -> Result<Vec<News>> {
    let limit = "50".to_string();
    let paginated = stock_news(&[("ticker", symbol), ("limit", &limit)]).await?;
    Ok(paginated
        .results
        .into_iter()
        .flatten()
        .map(|a| News {
            title: a.title.unwrap_or_default(),
            link: a.article_url.unwrap_or_default(),
            source: a.publisher.and_then(|p| p.name).unwrap_or_default(),
            img: String::new(),
            time: a.published_utc.unwrap_or_default(),
            provider_id: Some(Provider::Polygon),
        })
        .collect())
}
