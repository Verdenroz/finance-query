//! Timeseries Response Models
//!
//! Top-level wrapper and container for fundamentals timeseries data

use super::{TimeseriesDataPoint, TimeseriesMeta};
use serde::{Deserialize, Serialize};

/// Response wrapper for fundamentals timeseries endpoint
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeseriesResponse {
    /// Timeseries container
    pub timeseries: TimeseriesContainer,
}

/// Container for timeseries results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeseriesContainer {
    /// Timeseries results
    pub result: Vec<TimeseriesResult>,
    /// Error if any
    pub error: Option<serde_json::Value>,
}

/// A single timeseries result (e.g., annualTotalRevenue)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeseriesResult {
    /// Metadata about this timeseries
    pub meta: TimeseriesMeta,
    /// The actual timeseries data points
    #[serde(flatten)]
    pub data: std::collections::HashMap<String, Vec<TimeseriesDataPoint>>,
}

impl TimeseriesResponse {
    /// Parse from JSON value
    ///
    /// # Example
    /// ```no_run
    /// let json = serde_json::json!({
    ///     "timeseries": {
    ///         "result": [],
    ///         "error": null
    ///     }
    /// });
    /// let response = TimeseriesResponse::from_json(json)?;
    /// ```
    pub fn from_json(value: serde_json::Value) -> Result<Self, serde_json::Error> {
        serde_json::from_value(value)
    }

    /// Get all timeseries results
    pub fn results(&self) -> &[TimeseriesResult] {
        &self.timeseries.result
    }

    /// Check if there was an error
    pub fn has_error(&self) -> bool {
        self.timeseries.error.is_some()
    }
}
