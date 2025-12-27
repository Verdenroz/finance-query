//! Search Quote Model
//!
//! Represents individual stock/symbol search results

use serde::{Deserialize, Serialize};

/// A quote result from symbol search
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "dataframe", derive(crate::ToDataFrame))]
#[serde(rename_all = "camelCase")]
pub struct SearchQuote {
    /// Stock symbol
    pub symbol: String,
    /// Short name
    pub short_name: Option<String>,
    /// Long name
    pub long_name: Option<String>,
    /// Quote type (EQUITY, ETF, OPTION, etc.)
    pub quote_type: Option<String>,
    /// Exchange code
    pub exchange: Option<String>,
    /// Exchange display name
    pub exch_disp: Option<String>,
    /// Type display name
    pub type_disp: Option<String>,
    /// Industry classification
    pub industry: Option<String>,
    /// Sector classification
    pub sector: Option<String>,
    /// Whether this is a Yahoo Finance listed symbol
    #[serde(rename = "isYahooFinance")]
    pub is_yahoo_finance: Option<bool>,
    /// Display security industry
    pub disp_sec_ind: Option<String>,
}
