//! GraphQL type for news articles.

use async_graphql::SimpleObject;
use serde::Deserialize;

/// Lexicon-based sentiment score for a news article's title.
#[derive(SimpleObject, Deserialize, Debug, Clone)]
pub struct GqlSentiment {
    pub label: String,
    pub score: f64,
    pub confidence: f64,
}

/// A scraped news article.
#[derive(SimpleObject, Deserialize, Debug, Clone)]
pub struct GqlNews {
    pub title: String,
    pub link: String,
    pub source: String,
    pub img: String,
    pub time: String,
    /// Sentiment score for this article's title (VADER lexicon-based); `null`
    /// if the `sentiment` feature isn't compiled in.
    pub sentiment: Option<GqlSentiment>,
}
