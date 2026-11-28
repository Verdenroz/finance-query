//! Commodities market data models.
//!
//! Canonical public types for commodity quotes (gold, silver, oil, etc.),
//! shared across FMP and Alpha Vantage providers.

use finance_query_derive::FieldsInit;
use serde::{Deserialize, Serialize};

/// A commodity price quote (e.g., gold, silver, crude oil).
///
/// Obtain via `Providers::commodity``(symbol).quote()`.
#[derive(Debug, Clone, Serialize, Deserialize, FieldsInit)]
#[cfg_attr(feature = "dataframe", derive(crate::ToDataFrame))]
#[cfg_attr(feature = "dataframe", dataframe(crate = "crate"))]
#[non_exhaustive]
pub struct CommodityQuote {
    /// Commodity symbol (e.g., `"GCUSD"` for gold, `"CLUSD"` for crude oil)
    pub symbol: String,
    /// Human-readable commodity name (e.g., `"Gold"`, `"Crude Oil WTI"`)
    pub name: Option<String>,
    /// Unit of measurement (e.g., `"troy ounce"`, `"barrel"`)
    pub unit: Option<String>,
    /// Current price
    pub price: Option<f64>,
    /// Price change
    pub change: Option<f64>,
    /// Price change percentage
    pub change_percent: Option<f64>,
    /// Unix timestamp of the last update
    #[cfg_attr(feature = "dataframe", dataframe(datetime = "s"))]
    pub timestamp: Option<i64>,
}

#[cfg(all(test, feature = "dataframe"))]
mod dataframe_tests;
