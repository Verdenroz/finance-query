//! Search Response Model
//!
//! Top-level wrapper for symbol search results

use super::{SearchNews, SearchQuote};
use serde::{Deserialize, Serialize};

/// Response wrapper for search endpoint
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SearchResponse {
    /// Total result count
    pub count: Option<i32>,
    /// Quote/symbol results
    pub quotes: Vec<SearchQuote>,
    /// News article results
    pub news: Option<Vec<SearchNews>>,
    /// Total search execution time (milliseconds)
    pub total_time: Option<i64>,
}

impl SearchResponse {
    /// Parse SearchResponse from JSON value
    ///
    /// # Example
    /// ```no_run
    /// let json = serde_json::json!({
    ///     "count": 10,
    ///     "quotes": [],
    ///     "news": []
    /// });
    /// let response = SearchResponse::from_json(json)?;
    /// ```
    pub fn from_json(value: serde_json::Value) -> Result<Self, serde_json::Error> {
        serde_json::from_value(value)
    }

    /// Get all quote results
    pub fn quotes(&self) -> &[SearchQuote] {
        &self.quotes
    }

    /// Get news results if available
    pub fn news_results(&self) -> Vec<&SearchNews> {
        self.news
            .as_ref()
            .map(|news| news.iter().collect())
            .unwrap_or_default()
    }

    /// Get total result count
    pub fn result_count(&self) -> i32 {
        self.count.unwrap_or(0)
    }
}
