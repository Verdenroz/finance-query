//! Multi-symbol data access.
//!
//! Provides [`Tickers`] for fetching quotes, charts, and other data
//! for multiple symbols with optimized batch operations.

mod core;
#[cfg(feature = "dataframe")]
mod dataframe;
mod macros;

pub use core::{
    BatchCapitalGainsResponse, BatchChartsResponse, BatchDividendsResponse,
    BatchFinancialsResponse, BatchNewsResponse, BatchOptionsResponse, BatchQuotesAs,
    BatchQuotesResponse, BatchRecommendationsResponse, BatchSparksResponse, BatchSplitsResponse,
    Tickers, TickersBuilder,
};

#[cfg(feature = "indicators")]
pub use core::BatchIndicatorsResponse;
