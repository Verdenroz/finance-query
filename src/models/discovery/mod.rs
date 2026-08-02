//! Discovery models.
//!
//! Search, lookup, screeners, and trending tickers.

/// Type-filtered symbol lookup.
pub mod lookup;
/// Provider-routed symbol reference data (search, details, exchanges, screener).
#[cfg(any(feature = "fmp", feature = "polygon", feature = "alphavantage"))]
pub mod reference;
/// Predefined and custom screeners.
pub mod screeners;
/// Full-text symbol and news search.
pub mod search;
/// Trending tickers.
pub mod trending;
