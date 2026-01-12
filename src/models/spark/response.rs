//! Internal response types for spark endpoint.
//!
//! These types are internal implementation details and not exposed in the public API.

use super::super::chart::ChartMeta;
use serde::{Deserialize, Serialize};

/// Response wrapper for spark endpoint
#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct SparkResponse {
    /// Spark container
    pub spark: SparkContainer,
}

/// Container for spark results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct SparkContainer {
    /// Spark results per symbol
    pub result: Option<Vec<SparkSymbolResult>>,
    /// Error if any
    pub error: Option<serde_json::Value>,
}

/// Spark result for a single symbol
#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct SparkSymbolResult {
    /// Symbol
    pub symbol: String,
    /// Response data (array of chart-like responses)
    pub response: Vec<SparkData>,
}

/// Spark data (similar to chart result but with minimal indicators)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct SparkData {
    /// Metadata about the chart
    pub meta: ChartMeta,
    /// Timestamps for each data point
    pub timestamp: Option<Vec<i64>>,
    /// Price indicators (only close by default)
    pub indicators: SparkIndicators,
}

/// Spark indicators containing only close prices
#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct SparkIndicators {
    /// Quote data (close prices only)
    pub quote: Vec<SparkQuote>,
}

/// Spark quote with only close prices
#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct SparkQuote {
    /// Close prices
    pub close: Option<Vec<Option<f64>>>,
}

impl SparkResponse {
    /// Parse from JSON value
    pub(crate) fn from_json(value: serde_json::Value) -> Result<Self, serde_json::Error> {
        serde_json::from_value(value)
    }
}
