//! Screener models.
//!
//! Types for Yahoo Finance predefined screener data (most actives, day gainers,
//! day losers, most shorted, growth stocks, undervalued stocks, and more).
//!
//! ## Usage
//!
//! The API returns a clean, flattened `ScreenersResponse`:
//! ```json
//! {
//!   "quotes": [...],
//!   "type": "most_actives",
//!   "description": "Stocks ordered in descending order by intraday trade volume",
//!   "lastUpdated": 1234567890
//! }
//! ```
//!
//! Each quote in the array is a `ScreenerQuote` with ~50 fields of market data.

mod quote;
mod response;

pub use quote::ScreenerQuote;
pub use response::ScreenersResponse;
