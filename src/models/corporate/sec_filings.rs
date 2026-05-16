//! SEC Filings Module
//!
//! Contains recent SEC filing information (10-K, 10-Q, 8-K, etc.).

use serde::{Deserialize, Serialize};

/// SEC filings data
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SecFilings {
    /// Maximum age of this data in seconds
    #[serde(default)]
    pub max_age: Option<i64>,

    /// List of SEC filings
    #[serde(default)]
    pub filings: Vec<SecFiling>,
}

/// Individual SEC filing
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SecFiling {
    /// Maximum age of this data in seconds
    #[serde(default)]
    pub max_age: Option<i64>,

    /// Filing date (string format)
    #[serde(default)]
    pub date: Option<String>,

    /// Filing date (epoch timestamp)
    #[serde(default)]
    pub epoch_date: Option<i64>,

    /// Type of filing (e.g., "10-K", "10-Q", "8-K")
    #[serde(default)]
    #[serde(rename = "type")]
    pub filing_type: Option<String>,

    /// Title of the filing
    #[serde(default)]
    pub title: Option<String>,

    /// URL to the Edgar filing
    #[serde(default)]
    pub edgar_url: Option<String>,

    /// List of exhibits associated with this filing
    #[serde(default)]
    pub exhibits: Vec<SecExhibit>,
}

/// SEC filing exhibit
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SecExhibit {
    /// Type/number of exhibit (e.g., "EX-21.1", "10-K")
    #[serde(default)]
    #[serde(rename = "type")]
    pub exhibit_type: Option<String>,

    /// URL to the exhibit document
    #[serde(default)]
    pub url: Option<String>,
}
