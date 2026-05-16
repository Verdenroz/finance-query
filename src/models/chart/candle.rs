use crate::Provider;
/// Candle module
///
/// Contains the OHLCV candle/bar structure.
use serde::{Deserialize, Serialize};

/// A single OHLCV candle/bar
///
/// Note: This struct cannot be manually constructed - obtain via `Ticker::chart()`.
#[non_exhaustive]
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(feature = "dataframe", derive(crate::ToDataFrame))]
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

    /// Which data provider served this data (e.g., "yahoo", "polygon").
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub provider_id: Option<Provider>,
}

impl From<crate::providers::types::CandleData> for Candle {
    fn from(c: crate::providers::types::CandleData) -> Self {
        Self {
            timestamp: c.timestamp,
            open: c.open,
            high: c.high,
            low: c.low,
            close: c.close,
            volume: c.volume as i64,
            adj_close: None,
            provider_id: None,
        }
    }
}
