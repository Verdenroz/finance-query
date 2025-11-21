//! # finance-query
//!
//! A Rust library for fetching financial data from Yahoo Finance.
//! Inspired by yfinance, but leveraging Rust's speed for eager data loading.
//!
//! ## Features
//!
//! - **yfinance-like API**: Familiar interface for Python users migrating to Rust
//! - **Eager loading**: ONE HTTP request fetches ALL data (~1ms to deserialize)
//! - **Synchronous property access**: After creation, no `.await` needed!
//! - **Strongly typed**: All data structures are fully typed with serde support
//! - **Zero configuration**: Just create a ticker and access data
//!
//! ## Quick Start
//!
//! ```no_run
//! use finance_query::Ticker;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     // Create a ticker - fetches ALL modules in ONE request
//!     let ticker = Ticker::new("AAPL").await?;  // <-- Only .await needed!
//!
//!     // All subsequent access is synchronous (no .await!)
//!     // Access modules - all already loaded!
//!     if let Some(financials) = ticker.financial_data() {
//!         println!("Financial data: {:?}", financials);
//!     }
//!
//!     if let Some(profile) = ticker.asset_profile() {
//!         println!("Company profile: {:?}", profile);
//!     }
//!
//!     Ok(())
//! }
//! ```
//!
//! ## For Non-Async Code
//!
//! If you need to use this library in synchronous code:
//!
//! ```no_run
//! use finance_query::Ticker;
//!
//! fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     // Create a runtime and block on ticker creation
//!     let ticker = tokio::runtime::Runtime::new()?
//!         .block_on(Ticker::new("AAPL"))?;
//!
//!     // Everything else is synchronous!
//!     if let Some(financials) = ticker.financial_data() {
//!         println!("Financial data: {:?}", financials);
//!     }
//!
//!     Ok(())
//! }
//! ```

#![warn(missing_docs)]
#![warn(rustdoc::missing_crate_level_docs)]

// Internal modules (not exposed to users)
pub(crate) mod auth;
pub(crate) mod client;
pub(crate) mod constants;
pub(crate) mod endpoints;

/// Error types and result definitions
pub mod error;
/// Data models for Yahoo Finance responses
pub mod models;
/// High-level Ticker API
mod ticker;

// Public exports
pub use client::{ClientConfig, YahooClient};
pub use constants::{Interval, TimeRange, defaults};
pub use error::{Error, Result, YahooError};
pub use models::{
    chart::{Candle, ChartMeta, ChartResponse, ChartResult},
    quote_summary::{Module, QuoteSummaryData},
    quote_type::{QuoteTypeResponse, QuoteTypeResult},
    recommendation::{RecommendationResponse, RecommendedSymbol},
    search::{SearchQuote, SearchResponse},
    timeseries::{TimeseriesDataPoint, TimeseriesResponse, fundamental_types},
};
pub use ticker::Ticker;
