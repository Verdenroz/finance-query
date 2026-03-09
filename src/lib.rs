//! # finance-query
//!
//! A Rust library for querying financial data.
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
//!         .region_code("JP")
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
/// SEC EDGAR API client for filing history, XBRL data, and full-text search.
pub mod edgar;
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
pub(crate) mod rate_limiter;
mod scrapers;
mod ticker;
mod tickers;
mod utils;

// Feature-gated external data source modules
#[cfg(feature = "fred")]
pub mod fred;

#[cfg(feature = "crypto")]
pub mod crypto {
    //! CoinGecko cryptocurrency data (requires `crypto` feature).
    pub use crate::coingecko::{CoinQuote, coin, coins};
}

#[cfg(feature = "crypto")]
mod coingecko;

#[cfg(feature = "rss")]
pub mod feeds;

#[cfg(feature = "risk")]
pub mod risk;

// ============================================================================
// High-level API - Primary interface for most use cases
// ============================================================================
pub use ticker::{ClientHandle, Ticker, TickerBuilder};
pub use tickers::{
    BatchCapitalGainsResponse, BatchChartsResponse, BatchDividendsResponse,
    BatchFinancialsResponse, BatchNewsResponse, BatchOptionsResponse, BatchQuotesResponse,
    BatchRecommendationsResponse, BatchSparksResponse, Tickers, TickersBuilder,
};

#[cfg(feature = "indicators")]
pub use tickers::BatchIndicatorsResponse;

// ============================================================================
// Error types and results
// ============================================================================
pub use error::{FinanceError, Result};

// ============================================================================
// Options - Configure API requests
// ============================================================================
pub use finance::{LookupOptions, LookupType, SearchOptions};

// ============================================================================
// Parameter enums - Used with Ticker and finance methods
// ============================================================================
pub use constants::indices::Region as IndicesRegion;
pub use constants::screeners::Screener;
pub use constants::sectors::Sector;
pub use constants::{Frequency, Interval, Region, StatementType, TimeRange, ValueFormat};

// ============================================================================
// Response types - Top-level types returned by API methods
// ============================================================================
pub use models::{
    chart::Chart,
    currencies::Currency,
    edgar::{CompanyFacts, EdgarSearchResults, EdgarSubmissions},
    exchanges::Exchange,
    financials::FinancialStatement,
    hours::MarketHours,
    industries::IndustryData,
    lookup::LookupResults,
    market_summary::MarketSummaryQuote,
    news::News,
    options::Options,
    quote::Quote,
    recommendation::Recommendation,
    screeners::ScreenerResults,
    search::SearchResults,
    sectors::SectorData,
    sentiment::{FearAndGreed, FearGreedLabel},
    spark::Spark,
    transcript::Transcript,
    transcript::TranscriptWithMeta,
    trending::TrendingQuote,
};

// ============================================================================
// Nested types - Commonly accessed fields within response types
// ============================================================================
pub use models::{
    chart::{Candle, CapitalGain, ChartMeta, Dividend, DividendAnalytics, Split},
    edgar::filing_index::{EdgarFilingIndex, EdgarFilingIndexItem},
    edgar::{
        CikEntry, EdgarFiling, EdgarFilingFile, EdgarFilingRecent, EdgarFilings, EdgarSearchHit,
        EdgarSearchHitsContainer, EdgarSearchSource, EdgarSearchTotal, FactConcept, FactUnit,
        FactsByTaxonomy,
    },
    hours::MarketTime,
    lookup::LookupQuote,
    market_summary::SparkData,
    options::{Contracts, OptionChain, OptionContract, OptionsQuote},
    quote::FormattedValue,
    recommendation::SimilarSymbol,
    screeners::ScreenerQuote,
    search::{
        ResearchReport, ResearchReports, SearchNews, SearchNewsList, SearchQuote, SearchQuotes,
    },
};

// ============================================================================
// Query builders - Types for constructing custom screener queries
// ============================================================================
pub use constants::exchange_codes::ExchangeCode;
pub use constants::industries::Industry;
pub use models::screeners::{
    ConditionValue, EquityField, EquityScreenerQuery, FundField, FundScreenerQuery,
    LogicalOperator, Operator, QueryCondition, QueryGroup, QueryOperand, QuoteType, ScreenerField,
    ScreenerFieldExt, ScreenerFundCategory, ScreenerPeerGroup, ScreenerQuery, SortType,
};

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

// ============================================================================
// Technical Indicators (requires "indicators" feature)
// ============================================================================
// Technical analysis indicators for price data (SMA, EMA, RSI, MACD, Bollinger Bands).
// When enabled, Chart gets extension methods: chart.sma(), chart.ema(), chart.rsi(), etc.
#[cfg(feature = "indicators")]
pub mod indicators;

#[cfg(feature = "indicators")]
pub use indicators::{
    // Summary types
    AroonData,
    // Individual indicator types
    BollingerBands,
    BollingerBandsData,
    BullBearPowerData,
    // Candlestick pattern types
    CandlePattern,
    DonchianChannelsData,
    ElderRayData,
    IchimokuData,
    Indicator,
    IndicatorError,
    IndicatorResult,
    IndicatorsSummary,
    KeltnerChannelsData,
    MacdData,
    MacdResult,
    PatternSentiment,
    StochasticData,
    SuperTrendData,
    atr,
    patterns,
};

// ============================================================================
// Backtesting Engine (requires "backtesting" feature)
// ============================================================================
// Strategy backtesting with pre-built and custom strategies, position tracking,
// stop-loss/take-profit, comprehensive performance metrics, parameter optimization,
// walk-forward validation, Monte Carlo simulation, and multi-symbol portfolio.
#[cfg(feature = "backtesting")]
pub mod backtesting;

// ============================================================================
// Compile-time thread-safety assertions
// ============================================================================
// Ticker and Tickers must be Send + Sync so they can be shared across
// async tasks and held across .await points (e.g., in Arc, tokio::spawn).
const _: () = {
    const fn assert_send_sync<T: Send + Sync>() {}
    let _ = assert_send_sync::<Ticker>;
    let _ = assert_send_sync::<Tickers>;
};
