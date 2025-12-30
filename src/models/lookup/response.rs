//! Lookup Response Model
//!
//! Top-level wrapper for symbol lookup results

use super::LookupQuote;
use serde::{Deserialize, Serialize};

/// Raw response wrapper from Yahoo Finance lookup endpoint
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
struct RawLookupResponse {
    finance: Option<RawFinanceResult>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
struct RawFinanceResult {
    result: Option<Vec<RawLookupResult>>,
    #[allow(dead_code)]
    error: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
struct RawLookupResult {
    documents: Option<Vec<LookupQuote>>,
    start: Option<i32>,
    count: Option<i32>,
}

/// Response wrapper for lookup endpoint
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
#[serde(rename_all = "camelCase")]
pub struct LookupResults {
    /// Quote/document results
    #[serde(default)]
    pub quotes: Vec<LookupQuote>,
    /// Starting index
    #[serde(skip_serializing_if = "Option::is_none")]
    pub start: Option<i32>,
    /// Total result count
    #[serde(skip_serializing_if = "Option::is_none")]
    pub count: Option<i32>,
}

impl LookupResults {
    /// Parse LookupResults from raw JSON value
    ///
    /// # Example
    /// ```no_run
    /// # use finance_query::LookupResults;
    /// let json = serde_json::json!({
    ///     "finance": {
    ///         "result": [{
    ///             "documents": [],
    ///             "start": 0,
    ///             "count": 0
    ///         }]
    ///     }
    /// });
    /// let results = LookupResults::from_json(json)?;
    /// # Ok::<(), serde_json::Error>(())
    /// ```
    pub fn from_json(value: serde_json::Value) -> Result<Self, serde_json::Error> {
        let raw: RawLookupResponse = serde_json::from_value(value)?;

        let (quotes, start, count) = raw
            .finance
            .and_then(|f| f.result)
            .and_then(|r| r.into_iter().next())
            .map(|result| {
                (
                    result.documents.unwrap_or_default(),
                    result.start,
                    result.count,
                )
            })
            .unwrap_or_default();

        Ok(LookupResults {
            quotes,
            start,
            count,
        })
    }

    /// Get all quote results
    pub fn quotes(&self) -> &[LookupQuote] {
        &self.quotes
    }

    /// Get total result count
    pub fn result_count(&self) -> i32 {
        self.count.unwrap_or(0)
    }

    /// Check if any results were found
    pub fn is_empty(&self) -> bool {
        self.quotes.is_empty()
    }
}

#[cfg(feature = "dataframe")]
impl LookupResults {
    /// Converts the quotes to a polars DataFrame.
    pub fn to_dataframe(&self) -> ::polars::prelude::PolarsResult<::polars::prelude::DataFrame> {
        LookupQuote::vec_to_dataframe(&self.quotes)
    }
}
