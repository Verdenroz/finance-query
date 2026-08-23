//! Discovery models.
//!
//! Search, lookup, screeners, and trending tickers.

/// Security-identifier mapping (CUSIP/ISIN/SEDOL/FIGI → instruments).
#[cfg(feature = "openfigi")]
pub mod figi;
/// Type-filtered symbol lookup.
pub mod lookup;
/// Provider-routed symbol reference data (search, details, exchanges, screener).
pub mod reference;
/// Predefined and custom screeners.
pub mod screeners;
/// Full-text symbol and news search.
pub mod search;
/// Trending tickers.
pub mod trending;
