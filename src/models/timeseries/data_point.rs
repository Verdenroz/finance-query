//! Timeseries Data Point Models
//!
//! Individual data points within a timeseries

use serde::{Deserialize, Serialize};

/// A single data point in a timeseries
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TimeseriesDataPoint {
    /// As of date (YYYY-MM-DD)
    pub as_of_date: Option<String>,
    /// Period type (e.g., "12M", "3M")
    pub period_type: Option<String>,
    /// Currency code
    pub currency_code: Option<String>,
    /// The reported value
    pub reported_value: Option<ReportedValue>,
}

/// A reported value with raw and formatted versions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReportedValue {
    /// Raw numeric value
    pub raw: Option<f64>,
    /// Formatted string value
    pub fmt: Option<String>,
}
