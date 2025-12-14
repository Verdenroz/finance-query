//! # finance-query
//!
//! A Rust library for fetching financial data from Yahoo Finance.
//! Inspired by yfinance, with smart lazy loading for efficient data fetching.
//!
//! ## Features
//!
//! - **yfinance-like API**: Familiar interface for Python users migrating to Rust
//! - **Smart lazy loading**: Data fetched only when needed, then cached
//! - **Efficient grouping**: All quote modules fetched in ONE request on first access
//! - **100% type safe**: All data structures fully typed with comprehensive models
//! - **In-memory caching**: Fetched data persists for the lifetime of the Ticker
//! - **Both sync and async**: Choose synchronous or asynchronous API based on your needs
//! - **Configurable client**: Customize timeout, proxy, language, and region settings
//!
//! ## Quick Start - Synchronous API
//!
//! ```no_run
//! use finance_query::Ticker;
//!
//! fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     // Simple: Create a ticker with default configuration
//!     let ticker = Ticker::new("AAPL")?;
//!
//!     // First access to any quote property fetches ALL quote modules in one request
//!     if let Some(financials) = ticker.financial_data()? {
//!         println!("Financial data: {:?}", financials);
//!     }
//!
//!     // Subsequent accesses use cached data (no additional network calls)
//!     if let Some(profile) = ticker.asset_profile()? {
//!         println!("Company profile: {:?}", profile);
//!     }
//!
//!     // Chart data is fetched separately and cached by interval/range
//!     let chart = ticker.chart(
//!         finance_query::Interval::OneDay,
//!         finance_query::TimeRange::OneMonth
//!     )?;
//!     println!("Candles: {}", chart.candles.len());
//!
//!     // Builder pattern: Fluent configuration
//!     let ticker_jp = Ticker::builder("7203.T")
//!         .lang("ja-JP")
//!         .region("JP")
//!         .timeout(std::time::Duration::from_secs(30))
//!         .build()?;
//!
//!     Ok(())
//! }
//! ```
//!
//! ## Quick Start - Asynchronous API
//!
//! ```no_run
//! use finance_query::AsyncTicker;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     // Simple: Create an async ticker with default configuration
//!     let ticker = AsyncTicker::new("AAPL").await?;
//!
//!     // First access to any quote property fetches ALL quote modules in one request
//!     if let Some(financials) = ticker.financial_data().await? {
//!         println!("Financial data: {:?}", financials);
//!     }
//!
//!     // Subsequent accesses use cached data (no additional network calls)
//!     if let Some(profile) = ticker.asset_profile().await? {
//!         println!("Company profile: {:?}", profile);
//!     }
//!
//!     // Chart data is fetched separately and cached by interval/range
//!     let chart = ticker.chart(
//!         finance_query::Interval::OneDay,
//!         finance_query::TimeRange::OneMonth
//!     ).await?;
//!     println!("Candles: {}", chart.candles.len());
//!
//!     // Builder pattern: Fluent configuration
//!     let ticker_jp = AsyncTicker::builder("7203.T")
//!         .lang("ja-JP")
//!         .region("JP")
//!         .timeout(std::time::Duration::from_secs(30))
//!         .build()
//!         .await?;
//!
//!     Ok(())
//! }
//! ```
//!
//! ## Smart Lazy Loading
//!
//! The library uses intelligent lazy loading:
//! - **Quote data**: All ~30 quote modules fetched together on first property access
//! - **Chart data**: Fetched per (interval, range) combination and cached
//! - **Recommendations**: Fetched once and cached
//!
//! This approach minimizes network requests while keeping memory usage efficient.

#![warn(missing_docs)]
#![warn(rustdoc::missing_crate_level_docs)]

// Internal modules (not exposed to users)
pub(crate) mod auth;
pub(crate) mod client;
/// Constants and default values
pub mod constants;
pub(crate) mod endpoints;
pub(crate) mod indicators;

/// Error types and result definitions
pub mod error;
/// Non-symbol-specific operations (search, movers, etc.)
pub mod finance;
/// Data models for Yahoo Finance responses
pub mod models;
/// High-level Ticker API
mod ticker;

// ============================================================================
// High-level API - Primary interface for most use cases
// ============================================================================
pub use ticker::{AsyncTicker, AsyncTickerBuilder, Ticker, TickerBuilder};

// ============================================================================
// Configuration API - Configure client behavior
// ============================================================================
pub use client::{ClientConfig, ClientConfigBuilder};

// ============================================================================
// Error types and results
// ============================================================================
pub use error::{Result, YahooError};

// ============================================================================
// Constants and parameter enums
// ============================================================================
pub use constants::{Country, Frequency, Interval, StatementType, TimeRange};

// ============================================================================
// Data models - Types returned by API methods
// ============================================================================
pub use models::{
    chart::{Candle, Chart, ChartMeta},
    movers::{MoverQuote, MoversResponse},
    news::{NewsArticle, NewsResponse, NewsThumbnail},
    options::{OptionContract, OptionsResponse},
    quote::{FormattedValue, Quote}, // Only Quote and FormattedValue - internals are hidden
    recommendation::Recommendation,
    search::SearchQuote,
    timeseries::{TimeseriesDataPoint, TimeseriesResponse, fundamental_types},
};

// ============================================================================
// Technical Indicators - Types returned by indicators() method
// ============================================================================
pub use indicators::IndicatorsSummary;
