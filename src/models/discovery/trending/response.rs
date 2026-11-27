//! Trending Response Model
//!
//! Represents trending ticker data from Yahoo Finance

use serde::{Deserialize, Serialize};

/// A trending stock/symbol quote
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "dataframe", derive(crate::ToDataFrame))]
#[cfg_attr(feature = "dataframe", dataframe(crate = "crate"))]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct TrendingQuote {
    /// Stock symbol
    pub symbol: String,
}

/// Raw response from trending endpoint
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
#[doc(hidden)]
pub struct RawTrendingResponse {
    pub finance: Option<TrendingFinance>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
#[doc(hidden)]
pub struct TrendingFinance {
    pub result: Option<Vec<TrendingResult>>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
#[doc(hidden)]
pub struct TrendingResult {
    pub quotes: Option<Vec<TrendingQuote>>,
}

impl TrendingQuote {
    /// Parse trending quotes from the raw JSON response
    #[doc(hidden)]
    pub fn from_response(value: serde_json::Value) -> Result<Vec<Self>, serde_json::Error> {
        let raw: RawTrendingResponse = serde_json::from_value(value)?;
        Ok(raw
            .finance
            .and_then(|f| f.result)
            .and_then(|r| r.into_iter().next())
            .and_then(|r| r.quotes)
            .unwrap_or_default())
    }
}
