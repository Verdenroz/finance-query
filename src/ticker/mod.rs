//! Symbol-specific data access.
//!
//! Provides [`Ticker`] (sync) and [`AsyncTicker`] (async) for fetching quotes,
//! charts, financials, and news for specific symbols.

mod core;
mod macros;

pub use core::{AsyncTicker, AsyncTickerBuilder, Ticker, TickerBuilder};
