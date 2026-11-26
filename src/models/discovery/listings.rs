//! Complete listing identity.

use serde::{Deserialize, Serialize};

/// Provider-reported stock identity and listing lifecycle.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[cfg_attr(feature = "dataframe", derive(crate::ToDataFrame))]
#[non_exhaustive]
pub struct StockListing {
    /// Stock symbol.
    pub symbol: String,
    /// Company or security name.
    pub name: Option<String>,
    /// Primary exchange code.
    pub exchange: Option<String>,
    /// Provider stock-type code.
    pub stock_type: Option<String>,
    /// Market locale.
    pub locale: Option<String>,
    /// Provider-reported listing status.
    pub active: Option<bool>,
    /// Company identifier, preserving leading zeros.
    pub cik: Option<String>,
    /// Composite security identifier.
    pub composite_figi: Option<String>,
    /// Share-class identifier.
    pub share_class_figi: Option<String>,
    /// Provider-reported listing date.
    pub list_date: Option<String>,
    /// Provider-reported delisting timestamp.
    pub delisted_utc: Option<String>,
}

/// A ticker a security started trading under on a date.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "dataframe", derive(crate::ToDataFrame))]
#[non_exhaustive]
pub struct TickerChange {
    /// First day under this ticker (`YYYY-MM-DD`).
    pub date: String,
    /// The ticker from that day.
    pub ticker: String,
}

/// An open-ended provider stock-type code.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[cfg_attr(feature = "dataframe", derive(crate::ToDataFrame))]
#[non_exhaustive]
pub struct StockType {
    /// Provider code; unknown future codes are retained.
    pub code: String,
    /// Human-readable description.
    pub description: Option<String>,
    /// Asset class.
    pub asset_class: Option<String>,
    /// Market locale.
    pub locale: Option<String>,
}
