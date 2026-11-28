//! Spark models for batch sparkline data.
//!
//! Spark provides lightweight chart data optimized for sparkline rendering.
//! It fetches multiple symbols in a single request, returning only close prices.

#[cfg(all(test, feature = "dataframe"))]
mod dataframe_tests;
pub mod response;

use super::ChartMeta;
use serde::{Deserialize, Serialize};

/// Sparkline data for a single symbol.
///
/// Contains lightweight chart data optimized for sparkline rendering,
/// with only timestamps and close prices.
///
/// Note: This struct cannot be manually constructed - obtain via `Tickers::spark()`.
#[non_exhaustive]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(
    feature = "dataframe",
    derive(crate::ToDataFrame),
    dataframe(crate = "crate", columnar)
)]
pub struct Spark {
    /// Stock symbol
    pub symbol: String,
    /// Metadata about the chart (currency, exchange, current price, etc.)
    #[cfg_attr(feature = "dataframe", dataframe(skip))]
    pub meta: ChartMeta,
    /// Timestamps for each data point (Unix timestamps)
    #[cfg_attr(feature = "dataframe", dataframe(rename = "timestamp", datetime = "s"))]
    pub timestamps: Vec<i64>,
    /// Close prices for each data point
    #[cfg_attr(feature = "dataframe", dataframe(rename = "close"))]
    pub closes: Vec<f64>,
    /// Time interval (e.g., "5m", "1d")
    pub interval: Option<String>,
    /// Time range (e.g., "1d", "1mo")
    pub range: Option<String>,
}

impl Spark {
    /// Create from internal response data
    #[doc(hidden)]
    pub fn from_response(
        result: &response::SparkSymbolResult,
        interval: Option<String>,
        range: Option<String>,
    ) -> Option<Self> {
        let data = result.response.first()?;

        let prices = data
            .indicators
            .quote
            .first()
            .and_then(|q| q.close.as_deref())
            .unwrap_or_default();
        let (timestamps, closes) = match data.timestamp.as_deref() {
            Some(timestamps) => timestamps
                .iter()
                .zip(prices)
                .filter_map(|(&timestamp, &close)| Some((timestamp, close?)))
                .unzip(),
            None => (Vec::new(), prices.iter().flatten().copied().collect()),
        };

        Some(Self {
            symbol: result.symbol.clone(),
            meta: data.meta.clone(),
            timestamps,
            closes,
            interval,
            range,
        })
    }

    /// Number of data points
    pub fn len(&self) -> usize {
        self.closes.len()
    }

    /// Check if empty
    pub fn is_empty(&self) -> bool {
        self.closes.is_empty()
    }

    /// Get the price change from first to last close
    pub fn price_change(&self) -> Option<f64> {
        if self.closes.len() < 2 {
            return None;
        }
        let first = self.closes.first()?;
        let last = self.closes.last()?;
        Some(last - first)
    }

    /// Get the percentage change from first to last close
    pub fn percent_change(&self) -> Option<f64> {
        if self.closes.len() < 2 {
            return None;
        }
        let first = self.closes.first()?;
        let last = self.closes.last()?;
        if *first == 0.0 {
            return None;
        }
        Some(((last - first) / first) * 100.0)
    }

    /// Get the minimum close price
    pub fn min_close(&self) -> Option<f64> {
        self.closes.iter().copied().reduce(f64::min)
    }

    /// Get the maximum close price
    pub fn max_close(&self) -> Option<f64> {
        self.closes.iter().copied().reduce(f64::max)
    }
}

#[cfg(feature = "dataframe")]
impl Spark {
    /// Converts the chart metadata to a one-row polars DataFrame.
    pub fn meta_dataframe(&self) -> polars::prelude::PolarsResult<polars::prelude::DataFrame> {
        self.meta.to_dataframe()
    }
}
