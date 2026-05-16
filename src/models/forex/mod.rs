//! Forex (foreign exchange) market data models.
//!
//! Canonical public types for currency pair quotes and exchange rates,
//! shared across Polygon, FMP, and Alpha Vantage providers.
//!
//! Most types are scaffolding for upcoming provider implementations.
#![allow(dead_code)]

use serde::{Deserialize, Serialize};

/// A forex currency pair quote (e.g., EUR/USD).
///
/// Obtain via [`Ticker::forex`](crate::Ticker::forex) (future).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct ForexQuote {
    /// Currency pair symbol (e.g., `"EURUSD"`, `"BTCUSD"`)
    pub symbol: String,
    /// Base currency code (e.g., `"EUR"`)
    pub base_currency: Option<String>,
    /// Quote currency code (e.g., `"USD"`)
    pub quote_currency: Option<String>,
    /// Current exchange rate (bid)
    pub bid: Option<f64>,
    /// Ask price
    pub ask: Option<f64>,
    /// Midpoint or last traded price
    pub price: Option<f64>,
    /// Price change
    pub change: Option<f64>,
    /// Price change percentage
    pub change_percent: Option<f64>,
    /// Unix timestamp of the last update
    pub timestamp: Option<i64>,
}
