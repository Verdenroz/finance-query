/// Chart aggregate module
///
/// Contains the fully typed Chart structure for historical data.
use super::{Candle, ChartMeta};
use serde::{Deserialize, Serialize};

/// Fully typed chart data
///
/// Aggregates chart metadata and candles into a single convenient structure.
/// This is the recommended type for serialization and API responses.
/// Used for both single symbol and batch historical data requests.
///
/// Note: This struct cannot be manually constructed - use `Ticker::chart()` to obtain chart data.
#[non_exhaustive]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Chart {
    /// Stock symbol
    pub symbol: String,

    /// Chart metadata (exchange, currency, 52-week range, etc.)
    pub meta: ChartMeta,

    /// OHLCV candles/bars
    pub candles: Vec<Candle>,

    /// Time interval used (e.g., "1d", "1h")
    #[serde(skip_serializing_if = "Option::is_none")]
    pub interval: Option<String>,

    /// Time range used (e.g., "1mo", "1y")
    #[serde(skip_serializing_if = "Option::is_none")]
    pub range: Option<String>,
}

#[cfg(feature = "dataframe")]
impl Chart {
    /// Converts the candles to a polars DataFrame.
    ///
    /// Each candle becomes a row with columns for timestamp, open, high, low, close, volume.
    pub fn to_dataframe(&self) -> ::polars::prelude::PolarsResult<::polars::prelude::DataFrame> {
        Candle::vec_to_dataframe(&self.candles)
    }
}
