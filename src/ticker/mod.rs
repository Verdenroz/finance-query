//! Symbol-specific data access.
#![allow(missing_docs)]
//!
//! Provides [`Ticker`] for fetching quotes, charts, financials, and news
//! for specific symbols from all configured data providers.

mod core;
mod macros;
pub use core::{Ticker, TickerBuilder};
