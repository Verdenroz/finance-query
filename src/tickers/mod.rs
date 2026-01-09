//! Multi-symbol data access.
//!
//! Provides [`Tickers`] for fetching quotes, charts, and other data
//! for multiple symbols with optimized batch operations.

mod core;

pub use core::{
    BatchChartsResponse, BatchQuotesResponse, BatchSparksResponse, Tickers, TickersBuilder,
};
