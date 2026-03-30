//! External data source adapters.

/// Alpha Vantage financial data API (requires `alphavantage` feature).
#[cfg(feature = "alphavantage")]
pub mod alphavantage;

/// Polygon.io financial data API (requires `polygon` feature).
#[cfg(feature = "polygon")]
pub mod polygon;

/// Financial Modeling Prep (FMP) financial data API (requires `fmp` feature).
#[cfg(feature = "fmp")]
pub mod fmp;
