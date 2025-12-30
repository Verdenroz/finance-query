//! Trending Response Model
//!
//! Represents trending ticker data from Yahoo Finance

use serde::{Deserialize, Serialize};

/// A trending stock/symbol quote
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "dataframe", derive(crate::ToDataFrame))]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct TrendingQuote {
    /// Stock symbol
    pub symbol: String,
}

/// Raw response from trending endpoint
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct RawTrendingResponse {
    pub finance: Option<TrendingFinance>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
#[allow(dead_code)]
pub(crate) struct TrendingFinance {
    pub result: Option<Vec<TrendingResult>>,
    pub error: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
#[allow(dead_code)]
pub(crate) struct TrendingResult {
    pub count: Option<i32>,
    pub quotes: Option<Vec<TrendingQuote>>,
    pub job_timestamp: Option<i64>,
    pub start_interval: Option<i64>,
}

impl TrendingQuote {
    /// Parse trending quotes from the raw JSON response
    pub(crate) fn from_response(value: serde_json::Value) -> Result<Vec<Self>, serde_json::Error> {
        let raw: RawTrendingResponse = serde_json::from_value(value)?;
        Ok(raw
            .finance
            .and_then(|f| f.result)
            .and_then(|r| r.into_iter().next())
            .and_then(|r| r.quotes)
            .unwrap_or_default())
    }
}
