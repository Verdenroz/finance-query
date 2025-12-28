//! Research Report Model
//!
//! Represents research reports from search results

use serde::{Deserialize, Serialize};

/// A research report result from search
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "dataframe", derive(crate::ToDataFrame))]
#[non_exhaustive]
#[serde(rename_all = "camelCase")]
pub struct ResearchReport {
    /// Report headline/title
    #[serde(skip_serializing_if = "Option::is_none")]
    pub report_headline: Option<String>,
    /// Report author
    #[serde(skip_serializing_if = "Option::is_none")]
    pub author: Option<String>,
    /// Report publication date (Unix timestamp in milliseconds)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub report_date: Option<i64>,
    /// Report unique identifier
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    /// Provider name (e.g., "Morningstar", "Argus")
    #[serde(skip_serializing_if = "Option::is_none")]
    pub provider: Option<String>,
}
