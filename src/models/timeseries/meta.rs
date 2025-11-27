//! Timeseries Metadata Models
//!
//! Metadata describing timeseries data

use serde::{Deserialize, Serialize};

/// Metadata for a timeseries
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TimeseriesMeta {
    /// Stock symbol
    pub symbol: Vec<String>,
    /// Data type (e.g., "annualTotalRevenue")
    #[serde(rename = "type")]
    pub data_type: Vec<String>,
}
