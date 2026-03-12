//! GraphQL type for news articles.

use async_graphql::SimpleObject;
use serde::Deserialize;

/// A scraped news article.
#[derive(SimpleObject, Deserialize, Debug, Clone)]
pub struct GqlNews {
    pub title: String,
    pub link: String,
    pub source: String,
    pub img: String,
    pub time: String,
}
