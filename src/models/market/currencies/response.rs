//! Currency Response Model
//!
//! Represents currency information from Yahoo Finance

use serde::{Deserialize, Serialize};

/// A single currency with its properties
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "dataframe", derive(crate::ToDataFrame))]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct Currency {
    /// Short name (e.g., "USD/EUR")
    pub short_name: Option<String>,
    /// Long name (e.g., "USD/EUR")
    pub long_name: Option<String>,
    /// Symbol (e.g., "USDEUR=X")
    pub symbol: Option<String>,
    /// Local long name
    pub local_long_name: Option<String>,
}

/// Raw response from currencies endpoint
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct RawCurrenciesResponse {
    pub currencies: Option<CurrenciesResult>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct CurrenciesResult {
    pub result: Option<Vec<Currency>>,
}

impl Currency {
    /// Parse currencies from the raw JSON response
    pub(crate) fn from_response(value: serde_json::Value) -> Result<Vec<Self>, serde_json::Error> {
        let raw: RawCurrenciesResponse = serde_json::from_value(value)?;
        Ok(raw.currencies.and_then(|c| c.result).unwrap_or_default())
    }
}
