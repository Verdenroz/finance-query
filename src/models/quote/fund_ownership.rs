//! Fund Ownership Module
//!
//! Contains data about mutual fund ownership.

use serde::{Deserialize, Serialize};

/// Fund ownership data
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FundOwnership {
    /// Maximum age of this data in seconds
    #[serde(default)]
    pub max_age: Option<i64>,

    /// List of fund owners
    #[serde(default)]
    pub ownership_list: Vec<FundOwner>,
}

/// Individual fund owner
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FundOwner {
    /// Maximum age of this data in seconds
    #[serde(default)]
    pub max_age: Option<i64>,

    /// Name of the fund organization
    #[serde(default)]
    pub organization: Option<String>,

    /// Percentage of shares held
    #[serde(default)]
    pub pct_held: Option<crate::models::quote::FormattedValue<f64>>,

    /// Number of shares held
    #[serde(default)]
    pub position: Option<crate::models::quote::FormattedValue<i64>>,

    /// Total value of holdings
    #[serde(default)]
    pub value: Option<crate::models::quote::FormattedValue<i64>>,

    /// Percentage change in position
    #[serde(default)]
    pub pct_change: Option<crate::models::quote::FormattedValue<f64>>,

    /// Date of report
    #[serde(default)]
    pub report_date: Option<crate::models::quote::FormattedValue<i64>>,
}
