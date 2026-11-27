use crate::Provider;
/// Candle module
///
/// Contains the OHLCV candle/bar structure.
use serde::{Deserialize, Serialize};

/// A single OHLCV candle/bar
///
/// Note: This struct cannot be manually constructed - obtain via `Ticker::chart()`.
#[non_exhaustive]
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(
    feature = "dataframe",
    derive(crate::ToDataFrame),
    dataframe(crate = "crate")
)]
pub struct Candle {
    /// Timestamp (Unix)
    #[cfg_attr(feature = "dataframe", dataframe(datetime = "s"))]
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

    /// Which data provider served this data (e.g., "yahoo", "polygon").
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub provider_id: Option<Provider>,
}

impl Candle {
    /// Builds a candle from every field. Internal support for sibling crates, not public API.
    #[doc(hidden)]
    #[allow(clippy::too_many_arguments)]
    pub fn __new(
        timestamp: i64,
        open: f64,
        high: f64,
        low: f64,
        close: f64,
        volume: i64,
        adj_close: Option<f64>,
        provider_id: Option<Provider>,
    ) -> Self {
        Self {
            timestamp,
            open,
            high,
            low,
            close,
            volume,
            adj_close,
            provider_id,
        }
    }
}
