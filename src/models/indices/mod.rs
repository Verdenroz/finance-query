//! Stock market index data models.
//!
//! Canonical public types for index quotes and constituents,
//! shared across Polygon and FMP providers.
//!
//! Most types are scaffolding for upcoming provider implementations.
#![allow(dead_code)]

use serde::{Deserialize, Serialize};

/// A stock market index quote (e.g., S&P 500, NASDAQ, Dow Jones).
///
/// Obtain via [`Ticker::index`](crate::Ticker::index) (future).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct IndexQuote {
    /// Index ticker symbol (e.g., `"^GSPC"`, `"^IXIC"`)
    pub symbol: String,
    /// Human-readable index name (e.g., `"S&P 500"`)
    pub name: Option<String>,
    /// Current index value
    pub price: Option<f64>,
    /// Price change
    pub change: Option<f64>,
    /// Price change percentage
    pub change_percent: Option<f64>,
    /// Unix timestamp of the last update
    pub timestamp: Option<i64>,
}

/// A constituent (member) of a major stock market index.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct IndexConstituent {
    /// Ticker symbol of the constituent company
    pub symbol: String,
    /// Company name
    pub name: Option<String>,
    /// Sector classification
    pub sector: Option<String>,
    /// Industry classification
    pub industry: Option<String>,
    /// Market capitalization
    pub market_cap: Option<f64>,
    /// Weight in the index
    pub weight: Option<f64>,
}
