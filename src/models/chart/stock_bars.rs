//! Lossless stock-bar downloads, separate from chart presentation.

use serde::{Deserialize, Serialize};

/// Price adjustment policy. There is no implicit provider default.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub enum PriceAdjustment {
    /// Preserve historical, unadjusted prices.
    Unadjusted,
    /// Ask the provider to adjust prices for splits.
    SplitAdjusted,
}

/// A provider price bar with exact timestamp units and fractional volume.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[cfg_attr(feature = "dataframe", derive(crate::ToDataFrame))]
#[cfg_attr(feature = "dataframe", dataframe(crate = "crate"))]
#[non_exhaustive]
pub struct StockBar {
    /// UTC Unix milliseconds, without rounding or timezone conversion.
    #[cfg_attr(feature = "dataframe", dataframe(datetime = "ms", rename = "timestamp"))]
    pub timestamp_ms: i64,
    /// Opening price.
    pub open: f64,
    /// Highest price.
    pub high: f64,
    /// Lowest price.
    pub low: f64,
    /// Closing price.
    pub close: f64,
    /// Volume, preserving fractional provider values.
    pub volume: f64,
    /// Number of transactions, absent when not supplied.
    pub transactions: Option<u64>,
    /// Volume-weighted average price, when supplied.
    pub vwap: Option<f64>,
}
