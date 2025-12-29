//! # finance-query
//!
//! A Rust library for fetching financial data from Yahoo Finance.
//! Inspired by yfinance, with smart lazy loading for efficient data fetching.
//!
//! ## Quick Start
//!
//! ```no_run
//! use finance_query::Ticker;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     // Simple: Create a ticker with default configuration
//!     let ticker = Ticker::new("AAPL").await?;
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
//!     let ticker_jp = Ticker::builder("7203.T")
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
//! ## Lazy Loading
//!
//! The library uses lazy loading:
//! - **Quote data**: All ~30 quote modules fetched together on first property access
//! - **Chart data**: Fetched per (interval, range) combination and cached
//! - **Recommendations**: Fetched once and cached
//!
//! This approach minimizes network requests while keeping memory usage efficient.

#![warn(missing_docs)]
#![warn(rustdoc::missing_crate_level_docs)]

// === Modules ===
// Public modules
/// Error types and result definitions.
pub mod error;
/// Non-symbol-specific operations (search, lookup, screeners, market data, etc.).
pub mod finance;

// Internal modules
mod auth;
mod client;
mod constants;
mod endpoints;
mod models;
mod scrapers;
mod ticker;
mod tickers;

// ============================================================================
// High-level API - Primary interface for most use cases
// ============================================================================
pub use ticker::{Ticker, TickerBuilder};
pub use tickers::{BatchChartsResponse, BatchQuotesResponse, Tickers, TickersBuilder};

// ============================================================================
// Error types and results
// ============================================================================
pub use error::{Result, YahooError};

// ============================================================================
// Options - Configure API requests
// ============================================================================
pub use finance::{LookupOptions, LookupType, SearchOptions};

// ============================================================================
// Parameter enums - Used with Ticker and finance methods
// ============================================================================
pub use constants::indices::Region as IndicesRegion;
pub use constants::screener_query;
pub use constants::screener_types::ScreenerType;
pub use constants::sector_types::SectorType;
pub use constants::{Country, Frequency, Interval, StatementType, TimeRange, ValueFormat};

// ============================================================================
// Response types - Top-level types returned by API methods
// ============================================================================
pub use models::transcript::{Transcript, TranscriptWithMeta};
pub use models::{
    chart::Chart, currencies::Currency, financials::FinancialStatement, hours::MarketHours,
    indicators::IndicatorsSummary, industries::Industry, lookup::LookupResults,
    market_summary::MarketSummaryQuote, news::News, options::Options, quote::Quote,
    recommendation::Recommendation, screeners::ScreenerResults, search::SearchResults,
    sectors::Sector, trending::TrendingQuote,
};
pub use scrapers::yahoo_earnings::EarningsCall;

// ============================================================================
// Nested types - Commonly accessed fields within response types
// ============================================================================
pub use models::{
    chart::{Candle, CapitalGain, ChartMeta, Dividend, Split},
    hours::MarketTime,
    lookup::LookupQuote,
    market_summary::SparkData,
    options::{OptionChain, OptionContract, OptionsQuote},
    quote::FormattedValue,
    recommendation::SimilarSymbol,
    screeners::ScreenerQuote,
    search::{ResearchReport, SearchNews, SearchQuote},
};

// ============================================================================
// Query builders - Types for constructing custom screener queries
// ============================================================================
pub use models::screeners::{QueryCondition, QueryGroup, QueryOperand, QueryValue, ScreenerQuery};

// ============================================================================
// Real-time streaming
// ============================================================================
// WebSocket-based real-time price streaming with a Flow-like Stream API.
pub mod streaming;

// ============================================================================
// DataFrame support (requires "dataframe" feature)
// ============================================================================
// When enabled, structs with #[derive(ToDataFrame)] get a to_dataframe() method.
// The derive macro auto-generates DataFrame conversion for all scalar fields.
#[cfg(feature = "dataframe")]
pub use finance_query_derive::ToDataFrame;
