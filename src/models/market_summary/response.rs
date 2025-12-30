//! Market Summary Response Model
//!
//! Represents market summary quotes from Yahoo Finance

use crate::models::quote::FormattedValue;
use serde::{Deserialize, Serialize};

/// A single market summary quote (index, currency, commodity, etc.)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "dataframe", derive(crate::ToDataFrame))]
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
pub(crate) struct RawMarketSummaryResponse {
    pub market_summary_response: Option<MarketSummaryResult>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
#[allow(dead_code)]
pub(crate) struct MarketSummaryResult {
    pub result: Option<Vec<MarketSummaryQuote>>,
    pub error: Option<serde_json::Value>,
}

impl MarketSummaryQuote {
    /// Parse market summary quotes from the raw JSON response
    pub(crate) fn from_response(value: serde_json::Value) -> Result<Vec<Self>, serde_json::Error> {
        let raw: RawMarketSummaryResponse = serde_json::from_value(value)?;
        Ok(raw
            .market_summary_response
            .and_then(|r| r.result)
            .unwrap_or_default())
    }
}
