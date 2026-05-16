//! Futures market data models.
//!
//! Canonical public types for futures contracts and quotes,
//! shared across Polygon and other futures data providers.
//!
//! Most types are scaffolding for upcoming provider implementations.
#![allow(dead_code)]

use serde::{Deserialize, Serialize};

/// A futures contract quote.
///
/// Obtain via [`Ticker::futures`](crate::Ticker::futures) (future).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct FuturesQuote {
    /// Contract ticker symbol (e.g., `"ESM26"` for E-mini S&P June 2026)
    pub symbol: String,
    /// Human-readable contract name
    pub name: Option<String>,
    /// Underlying asset (e.g., `"S&P 500"`, `"Crude Oil"`)
    pub underlying: Option<String>,
    /// Exchange where the contract trades
    pub exchange: Option<String>,
    /// Contract expiration date as YYYY-MM-DD
    pub expiration_date: Option<String>,
    /// Current contract price
    pub price: Option<f64>,
    /// Price change
    pub change: Option<f64>,
    /// Price change percentage
    pub change_percent: Option<f64>,
    /// Open interest (number of outstanding contracts)
    pub open_interest: Option<u64>,
    /// Trading volume
    pub volume: Option<u64>,
    /// Unix timestamp of the last update
    pub timestamp: Option<i64>,
}
