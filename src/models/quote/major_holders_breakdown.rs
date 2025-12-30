//! Major Holders Breakdown Module
//!
//! Contains data about insider and institutional ownership percentages.

use serde::{Deserialize, Serialize};

/// Breakdown of ownership by different types of holders
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MajorHoldersBreakdown {
    /// Percentage of shares held by insiders
    #[serde(default)]
    pub insiders_percent_held: Option<crate::models::quote::FormattedValue<f64>>,

    /// Number of institutions holding shares
    #[serde(default)]
    pub institutions_count: Option<crate::models::quote::FormattedValue<i64>>,

    /// Percentage of float held by institutions
    #[serde(default)]
    pub institutions_float_percent_held: Option<crate::models::quote::FormattedValue<f64>>,

    /// Percentage of total shares held by institutions
    #[serde(default)]
    pub institutions_percent_held: Option<crate::models::quote::FormattedValue<f64>>,

    /// Maximum age of this data in seconds
    #[serde(default)]
    pub max_age: Option<i64>,
}
