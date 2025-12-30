//! Search Quote Model
//!
//! Represents individual stock/symbol search results

use serde::{Deserialize, Serialize};
use std::ops::Deref;

/// A collection of search quotes with DataFrame support.
///
/// This wrapper allows `search_results.quotes.to_dataframe()` syntax while still
/// acting like a `Vec<SearchQuote>` for iteration, indexing, etc.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(transparent)]
pub struct SearchQuotes(pub Vec<SearchQuote>);

impl Deref for SearchQuotes {
    type Target = Vec<SearchQuote>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl IntoIterator for SearchQuotes {
    type Item = SearchQuote;
    type IntoIter = std::vec::IntoIter<SearchQuote>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl<'a> IntoIterator for &'a SearchQuotes {
    type Item = &'a SearchQuote;
    type IntoIter = std::slice::Iter<'a, SearchQuote>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.iter()
    }
}

#[cfg(feature = "dataframe")]
impl SearchQuotes {
    /// Converts the quotes to a polars DataFrame.
    pub fn to_dataframe(&self) -> ::polars::prelude::PolarsResult<::polars::prelude::DataFrame> {
        SearchQuote::vec_to_dataframe(&self.0)
    }
}

/// A quote result from symbol search
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "dataframe", derive(crate::ToDataFrame))]
#[non_exhaustive]
#[serde(rename_all = "camelCase")]
pub struct SearchQuote {
    /// Stock symbol
    pub symbol: String,
    /// Short name
    #[serde(skip_serializing_if = "Option::is_none")]
    pub short_name: Option<String>,
    /// Long name
    #[serde(skip_serializing_if = "Option::is_none")]
    pub long_name: Option<String>,
    /// Quote type (EQUITY, ETF, OPTION, etc.)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub quote_type: Option<String>,
    /// Exchange code
    #[serde(skip_serializing_if = "Option::is_none")]
    pub exchange: Option<String>,
    /// Exchange display name
    #[serde(skip_serializing_if = "Option::is_none")]
    pub exch_disp: Option<String>,
    /// Type display name
    #[serde(skip_serializing_if = "Option::is_none")]
    pub type_disp: Option<String>,
    /// Industry classification
    #[serde(skip_serializing_if = "Option::is_none")]
    pub industry: Option<String>,
    /// Industry display name
    #[serde(skip_serializing_if = "Option::is_none")]
    pub industry_disp: Option<String>,
    /// Sector classification
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sector: Option<String>,
    /// Sector display name
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sector_disp: Option<String>,
    /// Whether this is a Yahoo Finance listed symbol
    #[serde(rename = "isYahooFinance")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_yahoo_finance: Option<bool>,
    /// Display security industry flag
    #[serde(skip_serializing_if = "Option::is_none")]
    pub disp_sec_ind_flag: Option<bool>,
    /// Company logo URL (requires enableLogoUrl=true in search request)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub logo_url: Option<String>,
    /// Search relevance score
    #[serde(skip_serializing_if = "Option::is_none")]
    pub score: Option<f64>,
    /// Index identifier
    #[serde(skip_serializing_if = "Option::is_none")]
    pub index: Option<String>,
    /// Previous company name (for recent name changes)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prev_name: Option<String>,
    /// Date of name change
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name_change_date: Option<String>,
}
