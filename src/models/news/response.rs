use super::article::NewsArticle;
use serde::{Deserialize, Serialize};

/// Response wrapper for news endpoint
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NewsResponse {
    /// News articles
    pub items: Option<Vec<NewsArticle>>,

    /// Total count
    pub count: Option<i32>,

    /// Start index
    pub start: Option<i32>,
}

impl NewsResponse {
    /// Parse from JSON value
    pub fn from_json(value: serde_json::Value) -> Result<Self, serde_json::Error> {
        serde_json::from_value(value)
    }

    /// Get the articles, returning empty vec if None
    pub fn articles(&self) -> Vec<NewsArticle> {
        self.items.clone().unwrap_or_default()
    }
}
