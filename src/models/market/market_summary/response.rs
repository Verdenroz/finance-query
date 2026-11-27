//! Market Summary Response Model
//!
//! Represents market summary quotes from Yahoo Finance

use crate::models::quote::FormattedValue;
use serde::{Deserialize, Serialize};

/// A single market summary quote (index, currency, commodity, etc.)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "dataframe", derive(crate::ToDataFrame))]
#[cfg_attr(feature = "dataframe", dataframe(crate = "crate"))]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct MarketSummaryQuote {
    /// Stock symbol
    pub symbol: String,
    /// Full market name
    pub full_exchange_name: Option<String>,
    /// Exchange code
    pub exchange: Option<String>,
    /// Short name
    pub short_name: Option<String>,
    /// Quote type (INDEX, CURRENCY, FUTURE, etc.)
    pub quote_type: Option<String>,
    /// Market state (REGULAR, PRE, POST, CLOSED)
    pub market_state: Option<String>,
    /// Regular market price
    #[serde(default)]
    pub regular_market_price: Option<FormattedValue<f64>>,
    /// Regular market change
    #[serde(default)]
    pub regular_market_change: Option<FormattedValue<f64>>,
    /// Regular market change percent
    #[serde(default)]
    pub regular_market_change_percent: Option<FormattedValue<f64>>,
    /// Regular market previous close
    #[serde(default)]
    pub regular_market_previous_close: Option<FormattedValue<f64>>,
    /// Regular market time (Unix timestamp)
    #[serde(default)]
    pub regular_market_time: Option<FormattedValue<i64>>,
    /// Spark chart data (if available)
    #[serde(skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "dataframe", dataframe(nested))]
    pub spark: Option<SparkData>,
}

/// Spark chart mini-data for market summary
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SparkData {
    /// Close prices
    #[serde(default)]
    pub close: Option<Vec<Option<f64>>>,
    /// Timestamps
    #[serde(default)]
    pub timestamp: Option<Vec<i64>>,
    /// Symbol
    #[serde(default)]
    pub symbol: Option<String>,
    /// Previous close
    #[serde(default)]
    pub previous_close: Option<f64>,
    /// Chart previous close
    #[serde(default)]
    pub chart_previous_close: Option<f64>,
}

/// Raw response from market summary endpoint
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
#[doc(hidden)]
pub struct RawMarketSummaryResponse {
    pub market_summary_response: Option<MarketSummaryResult>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
#[doc(hidden)]
pub struct MarketSummaryResult {
    pub result: Option<Vec<MarketSummaryQuote>>,
}

impl MarketSummaryQuote {
    /// Parse market summary quotes from the raw JSON response
    #[doc(hidden)]
    pub fn from_response(value: serde_json::Value) -> Result<Vec<Self>, serde_json::Error> {
        let raw: RawMarketSummaryResponse = serde_json::from_value(value)?;
        Ok(raw
            .market_summary_response
            .and_then(|r| r.result)
            .unwrap_or_default())
    }
}

#[cfg(feature = "dataframe")]
impl SparkData {
    /// Converts the points to a long-format polars DataFrame, one row per point.
    ///
    /// Columns are `symbol` (string, repeated on every row), `timestamp`
    /// (`Datetime(us, UTC)`) and `close` (f64). Missing closes stay null, and
    /// the shorter series is padded with nulls. A `None` series is empty.
    pub fn to_dataframe(&self) -> polars::prelude::PolarsResult<polars::prelude::DataFrame> {
        use crate::__dataframe::{EpochUnit, epoch_datetime, padded};
        use polars::prelude::*;

        let timestamps = self.timestamp.as_deref().unwrap_or_default();
        let closes = self.close.as_deref().unwrap_or_default();
        let rows = timestamps.len().max(closes.len());
        DataFrame::new(
            rows,
            vec![
                Series::new("symbol".into(), vec![self.symbol.as_deref(); rows]).into(),
                epoch_datetime(
                    "timestamp".into(),
                    EpochUnit::Seconds,
                    false,
                    padded(timestamps.iter().copied().map(Some), rows).into_iter(),
                ),
                Series::new("close".into(), padded(closes.iter().copied(), rows)).into(),
            ],
        )
    }

    /// Converts a slice of sparks to one long-format DataFrame with the columns
    /// of [`to_dataframe`](Self::to_dataframe).
    pub fn vec_to_dataframe(
        items: &[Self],
    ) -> polars::prelude::PolarsResult<polars::prelude::DataFrame> {
        use crate::__dataframe::{EpochUnit, epoch_datetime};
        use polars::prelude::*;

        let schema = DataFrame::new(
            0,
            vec![
                Series::new("symbol".into(), Vec::<&str>::new()).into(),
                epoch_datetime(
                    "timestamp".into(),
                    EpochUnit::Seconds,
                    false,
                    std::iter::empty(),
                ),
                Series::new("close".into(), Vec::<f64>::new()).into(),
            ],
        );
        crate::__dataframe::stack_rows(schema, items, Self::to_dataframe)
    }
}
