//! Earnings History Module
//!
//! Contains historical earnings data comparing actual vs estimated earnings.

use serde::{Deserialize, Serialize};

/// Historical earnings data
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EarningsHistory {
    /// Default methodology (e.g., "gaap")
    #[serde(default)]
    pub default_methodology: Option<String>,

    /// List of historical earnings
    #[serde(default)]
    pub history: Vec<EarningsHistoryEntry>,
}

/// Single historical earnings entry
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EarningsHistoryEntry {
    /// Maximum age of this data in seconds
    #[serde(default)]
    pub max_age: Option<i64>,

    /// Quarter for this earnings report
    #[serde(default)]
    pub quarter: Option<crate::models::quote::FormattedValue<i64>>,

    /// Period identifier (e.g., "-1q", "-2q")
    #[serde(default)]
    pub period: Option<String>,

    /// Currency code
    #[serde(default)]
    pub currency: Option<String>,

    /// Actual EPS reported
    #[serde(default)]
    pub eps_actual: Option<crate::models::quote::FormattedValue<f64>>,

    /// Estimated EPS
    #[serde(default)]
    pub eps_estimate: Option<crate::models::quote::FormattedValue<f64>>,

    /// Difference between actual and estimate
    #[serde(default)]
    pub eps_difference: Option<crate::models::quote::FormattedValue<f64>>,

    /// Surprise percentage
    #[serde(default)]
    pub surprise_percent: Option<crate::models::quote::FormattedValue<f64>>,
}
