//! Spark models for batch sparkline data.
//!
//! Spark provides lightweight chart data optimized for sparkline rendering.
//! It fetches multiple symbols in a single request, returning only close prices.

pub(crate) mod response;

use super::chart::ChartMeta;
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
pub struct Spark {
    /// Stock symbol
    pub symbol: String,
    /// Metadata about the chart (currency, exchange, current price, etc.)
    pub meta: ChartMeta,
    /// Timestamps for each data point (Unix timestamps)
    pub timestamps: Vec<i64>,
    /// Close prices for each data point
    pub closes: Vec<f64>,
    /// Time interval (e.g., "5m", "1d")
    pub interval: Option<String>,
    /// Time range (e.g., "1d", "1mo")
    pub range: Option<String>,
}

impl Spark {
    /// Create from internal response data
    pub(crate) fn from_response(
        result: &response::SparkSymbolResult,
        interval: Option<String>,
        range: Option<String>,
    ) -> Option<Self> {
        let data = result.response.first()?;

        let timestamps = data.timestamp.clone().unwrap_or_default();

        // Extract close prices, filtering out None values
        let closes: Vec<f64> = data
            .indicators
            .quote
            .first()
            .and_then(|q| q.close.as_ref())
            .map(|prices| prices.iter().filter_map(|&p| p).collect())
            .unwrap_or_default();

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
