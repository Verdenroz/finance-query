use super::ChartResult;
/// Chart Response module
///
/// Handles parsing of Yahoo Finance chart API responses.
use serde::{Deserialize, Serialize};

/// Response wrapper for chart endpoint
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChartResponse {
    /// Chart container
    pub chart: ChartContainer,
}

/// Container for chart results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChartContainer {
    /// Chart results
    pub result: Option<Vec<ChartResult>>,
    /// Error if any
    pub error: Option<serde_json::Value>,
}

impl ChartResponse {
    /// Parse from JSON value
    pub fn from_json(value: serde_json::Value) -> Result<Self, serde_json::Error> {
        serde_json::from_value(value)
    }
}
