//! Multi-symbol data access.
//!
//! Provides [`Tickers`] for fetching quotes, charts, and other data
//! for multiple symbols with optimized batch operations.

mod core;
mod macros;

pub use core::{
    BatchCapitalGainsResponse, BatchChartsResponse, BatchDividendsResponse,
    BatchFinancialsResponse, BatchNewsResponse, BatchOptionsResponse, BatchQuotesResponse,
    BatchRecommendationsResponse, BatchSparksResponse, Tickers, TickersBuilder,
};

#[cfg(feature = "indicators")]
pub use core::BatchIndicatorsResponse;
