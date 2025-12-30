//! Symbol-specific data access.
//!
//! Provides [`Ticker`] for fetching quotes, charts, financials, and news
//! for specific symbols.

mod core;
mod macros;

pub use core::{Ticker, TickerBuilder};
