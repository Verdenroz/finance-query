//! Symbol-specific data access.
//!
//! Provides [`Ticker`] for fetching quotes, charts, financials, and news
//! for specific symbols.

mod core;
mod macros;

pub use core::{ClientHandle, Ticker, TickerBuilder};

#[cfg(feature = "polygon")]
mod polygon;
#[cfg(feature = "polygon")]
pub use polygon::{FinancialPeriod, PolygonHandle};

#[cfg(feature = "fmp")]
mod fmp;
#[cfg(feature = "fmp")]
pub use fmp::{FmpHandle, IntradayInterval};

#[cfg(feature = "alphavantage")]
mod alphavantage;
#[cfg(feature = "alphavantage")]
pub use alphavantage::AlphaVantageHandle;
