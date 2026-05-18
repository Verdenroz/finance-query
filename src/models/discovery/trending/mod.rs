//! Trending models.
//!
//! Contains data structures for Yahoo Finance's trending tickers endpoint.

mod response;

pub use response::TrendingQuote;

#[cfg(feature = "python")]
pub use response::PyTrendingQuote;
