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
    /// Converts the points to a long-format polars DataFrame, one row per point.
    ///
    /// Columns are `symbol` (string), `timestamp` (i64), `close` (f64), `interval`
    /// and `range` (strings, repeated on every row). The shorter of `timestamps`
    /// and `closes` is padded with nulls. Chart metadata is in
    /// [`meta_dataframe`](Self::meta_dataframe).
    pub fn to_dataframe(&self) -> polars::prelude::PolarsResult<polars::prelude::DataFrame> {
        use crate::__dataframe::padded;
        use polars::prelude::*;

        let rows = self.timestamps.len().max(self.closes.len());
        DataFrame::new(
            rows,
            vec![
                Series::new("symbol".into(), vec![self.symbol.as_str(); rows]).into(),
                Series::new(
                    "timestamp".into(),
                    padded(self.timestamps.iter().copied().map(Some), rows),
                )
                .into(),
                Series::new(
                    "close".into(),
                    padded(self.closes.iter().copied().map(Some), rows),
                )
                .into(),
                Series::new("interval".into(), vec![self.interval.as_deref(); rows]).into(),
                Series::new("range".into(), vec![self.range.as_deref(); rows]).into(),
            ],
        )
    }

    /// Converts a slice of sparks to one long-format DataFrame with the columns
    /// of [`to_dataframe`](Self::to_dataframe).
    pub fn vec_to_dataframe(
        items: &[Self],
    ) -> polars::prelude::PolarsResult<polars::prelude::DataFrame> {
        use polars::prelude::*;

        let schema = DataFrame::new(
            0,
            vec![
                Series::new("symbol".into(), Vec::<&str>::new()).into(),
                Series::new("timestamp".into(), Vec::<i64>::new()).into(),
                Series::new("close".into(), Vec::<f64>::new()).into(),
                Series::new("interval".into(), Vec::<&str>::new()).into(),
                Series::new("range".into(), Vec::<&str>::new()).into(),
            ],
        );
        crate::__dataframe::stack_rows(schema, items, Self::to_dataframe)
    }

    /// Converts the chart metadata to a one-row polars DataFrame.
    pub fn meta_dataframe(&self) -> polars::prelude::PolarsResult<polars::prelude::DataFrame> {
        self.meta.to_dataframe()
    }
}
