/// Candle module
///
/// Contains the OHLCV candle/bar structure.
use serde::{Deserialize, Serialize};

#[cfg(feature = "python")]
use finance_query_derive::PyModel;

/// A single OHLCV candle/bar
///
/// Note: This struct cannot be manually constructed - obtain via `Ticker::chart()`.
#[non_exhaustive]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(feature = "dataframe", derive(crate::ToDataFrame))]
#[cfg_attr(feature = "python", derive(PyModel))]
#[cfg_attr(feature = "python", py_model(dataframe = "columns"))]
pub struct Candle {
    /// Timestamp (Unix)
    pub timestamp: i64,
    /// Open price
    pub open: f64,
    /// High price
    pub high: f64,
    /// Low price
    pub low: f64,
    /// Close price
    pub close: f64,
    /// Volume
    pub volume: i64,
    /// Adjusted close (if available)
    pub adj_close: Option<f64>,
}
