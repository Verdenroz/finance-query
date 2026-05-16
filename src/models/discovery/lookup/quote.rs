//! Lookup Quote Model
//!
//! Represents individual symbol results from the lookup endpoint

use serde::{Deserialize, Serialize};

/// A quote/document result from symbol lookup
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "dataframe", derive(crate::ToDataFrame))]
#[non_exhaustive]
#[serde(rename_all = "camelCase")]
pub struct LookupQuote {
    /// Stock symbol
    pub symbol: String,
    /// Short name
    #[serde(skip_serializing_if = "Option::is_none")]
    pub short_name: Option<String>,
    /// Long name
    #[serde(skip_serializing_if = "Option::is_none")]
    pub long_name: Option<String>,
    /// Quote type (EQUITY, ETF, MUTUALFUND, INDEX, FUTURE, CURRENCY, CRYPTOCURRENCY)
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
    /// Sector classification
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sector: Option<String>,
    /// Current/last price
    #[serde(skip_serializing_if = "Option::is_none")]
    pub regular_market_price: Option<f64>,
    /// Price change
    #[serde(skip_serializing_if = "Option::is_none")]
    pub regular_market_change: Option<f64>,
    /// Price change percent
    #[serde(skip_serializing_if = "Option::is_none")]
    pub regular_market_change_percent: Option<f64>,
    /// Previous close price
    #[serde(skip_serializing_if = "Option::is_none")]
    pub regular_market_previous_close: Option<f64>,
    /// Company logo URL (populated when include_logo=true)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub logo_url: Option<String>,
    /// Company logo URL (alternate, populated when include_logo=true)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub company_logo_url: Option<String>,
}
