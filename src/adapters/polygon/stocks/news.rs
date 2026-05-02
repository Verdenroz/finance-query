//! Stock news endpoints with sentiment analysis.

use serde::{Deserialize, Serialize};

use crate::error::Result;

use super::super::build_client;
use super::super::models::PaginatedResponse;

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
pub struct Insight {
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
    pub insights: Option<Vec<Insight>>,
}

/// Fetch news articles, optionally filtered by ticker.
///
/// * `params` - Query params: `ticker`, `published_utc`, `order`, `limit`, `sort`
pub async fn stock_news(params: &[(&str, &str)]) -> Result<PaginatedResponse<NewsArticle>> {
    let client = build_client()?;
    client.get("/v2/reference/news", params).await
}
