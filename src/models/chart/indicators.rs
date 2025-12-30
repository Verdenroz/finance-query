/// Chart Indicators module
///
/// Contains OHLCV data structures and adjusted close indicators.
/// These types are internal implementation details and not exposed in the public API.
use serde::{Deserialize, Serialize};

/// Chart indicators containing OHLCV data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct ChartIndicators {
    /// Quote data (OHLCV)
    pub quote: Vec<QuoteIndicator>,
    /// Adjusted close data
    #[serde(rename = "adjclose")]
    pub adj_close: Option<Vec<AdjCloseIndicator>>,
}

/// OHLCV quote indicator
#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct QuoteIndicator {
    /// Open prices
    pub open: Option<Vec<Option<f64>>>,
    /// High prices
    pub high: Option<Vec<Option<f64>>>,
    /// Low prices
    pub low: Option<Vec<Option<f64>>>,
    /// Close prices
    pub close: Option<Vec<Option<f64>>>,
    /// Volume
    pub volume: Option<Vec<Option<i64>>>,
}

/// Adjusted close indicator
#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct AdjCloseIndicator {
    /// Adjusted close prices
    #[serde(rename = "adjclose")]
    pub adj_close: Option<Vec<Option<f64>>>,
}
