//! Search Response Model
//!
//! Top-level wrapper for symbol search results

use super::{ResearchReport, SearchNews, SearchQuote};
use serde::{Deserialize, Serialize};

/// Response wrapper for search endpoint
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
#[serde(rename_all = "camelCase")]
pub struct SearchResults {
    /// Total result count
    #[serde(skip_serializing_if = "Option::is_none")]
    pub count: Option<i32>,
    /// Quote/symbol results
    #[serde(default)]
    pub quotes: Vec<SearchQuote>,
    /// News article results
    #[serde(skip_serializing_if = "Option::is_none")]
    pub news: Option<Vec<SearchNews>>,
    /// Research reports (requires enableResearchReports=true in search request)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub research_reports: Option<Vec<ResearchReport>>,
    /// Total search execution time (milliseconds)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub total_time: Option<i64>,
}

impl SearchResults {
    /// Parse SearchResults from JSON value
    ///
    /// # Example
    /// ```no_run
    /// # use finance_query::models::search::SearchResults;
    /// let json = serde_json::json!({
    ///     "count": 10,
    ///     "quotes": [],
    ///     "news": []
    /// });
    /// let response = SearchResults::from_json(json)?;
    /// # Ok::<(), serde_json::Error>(())
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

    /// Get research reports if available
    pub fn research_reports(&self) -> Vec<&ResearchReport> {
        self.research_reports
            .as_ref()
            .map(|reports| reports.iter().collect())
            .unwrap_or_default()
    }

    /// Get total result count
    pub fn result_count(&self) -> i32 {
        self.count.unwrap_or(0)
    }
}
