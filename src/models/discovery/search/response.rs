//! Search Response Model
//!
//! Top-level wrapper for symbol search results

use super::{ResearchReports, SearchNewsList, SearchQuotes};
use serde::{Deserialize, Serialize};

/// Response wrapper for search endpoint
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
#[serde(rename_all = "camelCase")]
pub struct SearchResults {
    /// Total result count
    #[serde(skip_serializing_if = "Option::is_none")]
    pub count: Option<i32>,
    /// Quote/symbol results - use `.quotes.to_dataframe()` for DataFrame conversion
    #[serde(default)]
    pub quotes: SearchQuotes,
    /// News article results - use `.news.to_dataframe()` for DataFrame conversion
    #[serde(default)]
    pub news: SearchNewsList,
    /// Research reports - use `.research_reports.to_dataframe()` for DataFrame conversion
    #[serde(default)]
    pub research_reports: ResearchReports,
    /// Total search execution time (milliseconds)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub total_time: Option<i64>,
}

impl SearchResults {
    /// Parse SearchResults from JSON value
    ///
    /// # Example
    /// ```no_run
    /// # use finance_query::SearchResults;
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

    /// Get total result count
    pub fn result_count(&self) -> i32 {
        self.count.unwrap_or(0)
    }
}
