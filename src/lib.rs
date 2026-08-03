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
//! ## Lazy Loading and Caching
//!
//! The library fetches on demand and caches by default:
//! - **Quote data**: all quote modules fetched together on first property access, then reused
//! - **Chart data**: fetched and cached per (interval, range) combination
//! - **Recommendations**: fetched once and cached
//!
//! A handle caches for as long as it lives. Use `.cache(ttl)` on the builder to
//! bound that, or `.no_cache()` to fetch fresh on every call.

#![warn(missing_docs)]
#![warn(rustdoc::missing_crate_level_docs)]

// === Modules ===
// Public modules
/// External data source adapters (internal — use the public API modules).
pub(crate) mod adapters;
/// Error types and result definitions.
pub mod error;
/// Non-symbol-specific operations (search, lookup, screeners, market data, etc.).
pub mod finance;
pub mod edgar {
    //! SEC EDGAR API client (keyless — always available, no feature flag needed).
    //!
    //!
    //! Requires a one-time [`init`] call with a contact email address.
    pub use crate::adapters::edgar::{
        company_facts, filing_index, init, init_with_config, resolve_cik, search, submissions,
    };
}

// Internal modules
mod constants;
mod models;
mod providers;
pub(crate) mod rate_limiter;
mod scrapers;
mod ticker;
mod tickers;
mod utils;

// Feature-gated external data source modules
#[cfg(feature = "fred")]
pub mod fred {
    //! FRED economic data API (requires `fred` feature).
    //!
    //! Access 800k+ macroeconomic time series and US Treasury yield curve data.
    pub use crate::adapters::fred::{init, init_with_timeout, series, treasury_yields};
    pub use crate::models::economic::{MacroObservation, MacroSeries, TreasuryYield};
}

#[cfg(feature = "crypto")]
pub mod crypto {
    //! CoinGecko cryptocurrency data (requires `crypto` feature).
    pub use crate::adapters::coingecko::{CoinQuote, coin, coins};
}

#[cfg(feature = "openfigi")]
pub mod openfigi {
    //! Security-identifier resolution via OpenFIGI (requires `openfigi`
    //! feature, keyless).
    //!
    //! Resolves a CUSIP, ISIN, SEDOL, or FIGI to the instruments carrying it —
    //! the missing step for any dataset that identifies holdings by CUSIP
    //! rather than ticker, such as 13F filings.
    //!
    //! Lives here rather than behind the Providers API because resolution is
    //! not tied to a symbol handle and maps onto no
    //! [`Capability`](crate::Capability), the same reasoning that puts
    //! [`edgar`](crate::edgar) and [`fred`](crate::fred) at the crate root.
    //!
    //! No API key is required; `OPENFIGI_API_KEY` is optional and only raises
    //! the quota.
    //!
    //! ```no_run
    //! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    //! use finance_query::openfigi;
    //!
    //! // One CUSIP maps to every venue listing of the security.
    //! for listing in openfigi::resolve_cusip("037833100").await? {
    //!     println!("{:?} on {:?}", listing.ticker, listing.exchange_code);
    //! }
    //! # Ok(())
    //! # }
    //! ```

    use crate::error::Result;
    pub use crate::models::discovery::figi::{SecurityIdKind, SecurityMapping};

    /// Resolve a CUSIP to every instrument carrying it.
    ///
    /// Returns an empty list when the identifier is well-formed but matches
    /// nothing; a malformed identifier is an error.
    pub async fn resolve_cusip(cusip: &str) -> Result<Vec<SecurityMapping>> {
        crate::adapters::openfigi::resolve(SecurityIdKind::Cusip, cusip).await
    }

    /// Resolve an ISIN to every instrument carrying it.
    pub async fn resolve_isin(isin: &str) -> Result<Vec<SecurityMapping>> {
        crate::adapters::openfigi::resolve(SecurityIdKind::Isin, isin).await
    }

    /// Resolve a SEDOL to every instrument carrying it.
    pub async fn resolve_sedol(sedol: &str) -> Result<Vec<SecurityMapping>> {
        crate::adapters::openfigi::resolve(SecurityIdKind::Sedol, sedol).await
    }

    /// Resolve an identifier of any supported [`SecurityIdKind`].
    pub async fn resolve(kind: SecurityIdKind, id: &str) -> Result<Vec<SecurityMapping>> {
        crate::adapters::openfigi::resolve(kind, id).await
    }

    /// Resolve many identifiers of the same kind in as few requests as
    /// possible (OpenFIGI accepts 10 per request without a key).
    ///
    /// The result is positional: element `i` answers `ids[i]`, with an empty
    /// list where nothing matched.
    pub async fn resolve_many(
        kind: SecurityIdKind,
        ids: &[&str],
    ) -> Result<Vec<Vec<SecurityMapping>>> {
        crate::adapters::openfigi::resolve_many(kind, ids).await
    }
}

#[cfg(feature = "defi")]
pub mod defi {
    //! Market-wide DeFi data via DefiLlama (requires `defi` feature, keyless).
    //!
    //! Chain rankings and stablecoin supplies describe the market as a whole,
    //! not one asset, so there is no symbol handle to hang them off — they sit
    //! at the crate root the way [`edgar`](crate::edgar) and
    //! [`fred`](crate::fred) do.
    //!
    //! Protocol-shaped data *is* symbol-shaped and lives on
    //! [`CryptoCoin::tvl`](crate::CryptoCoin::tvl) instead.
    //!
    //! ```no_run
    //! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    //! use finance_query::defi;
    //!
    //! for chain in defi::chains().await?.into_iter().take(5) {
    //!     println!("{}: ${:?}", chain.name, chain.tvl);
    //! }
    //! # Ok(())
    //! # }
    //! ```

    use crate::error::Result;
    pub use crate::models::crypto::defi::{
        ChainAllocation, ChainTvl, ProtocolTvl, StablecoinSupply, TvlPoint,
    };

    /// Fetch aggregate total value locked for every chain, largest first.
    pub async fn chains() -> Result<Vec<ChainTvl>> {
        crate::adapters::defillama::chains().await
    }

    /// Fetch circulating supply for every tracked stablecoin, largest first.
    ///
    /// Supplies are denominated in the coin's pegged asset — read `peg_type`
    /// before summing across coins pegged to different currencies.
    pub async fn stablecoins() -> Result<Vec<StablecoinSupply>> {
        crate::adapters::defillama::stablecoins().await
    }
}

pub mod feeds;

#[cfg(feature = "risk")]
pub mod risk;

#[cfg(feature = "translation")]
pub mod translation;

// ============================================================================
// High-level API - Primary interface for most use cases
// ============================================================================
pub mod domains;
pub use providers::config::{Providers, ProvidersBuilder};
pub use providers::{Capability, Fetch, Operation, Provider};
pub use ticker::{ClientHandle, Ticker, TickerBuilder};

// Domain-specific query handles — constructable via Providers factory methods.
#[cfg(any(
    feature = "alphavantage",
    feature = "binance",
    feature = "crypto",
    feature = "defi",
    feature = "fmp",
    feature = "kraken",
    feature = "polygon"
))]
pub use domains::CryptoCoin;
#[cfg(any(
    feature = "alphavantage",
    feature = "fmp",
    feature = "frankfurter",
    feature = "polygon"
))]
pub use domains::ForexPair;
#[cfg(any(
    feature = "alphavantage",
    feature = "bls",
    feature = "fiscaldata",
    feature = "fred",
    feature = "polygon",
    feature = "worldbank"
))]
pub use domains::{EconomicCatalog, EconomicIndicator};

// Remaining Capability handles — indices, futures, commodities, filings, discovery
#[cfg(any(feature = "fmp", feature = "alphavantage"))]
pub use domains::Commodity;
#[cfg(any(feature = "fmp", feature = "polygon", feature = "alphavantage"))]
pub use domains::Discovery;
pub use domains::Filings;
#[cfg(feature = "polygon")]
pub use domains::FuturesContract;
#[cfg(any(feature = "polygon", feature = "fmp"))]
pub use domains::Index;
pub use domains::Snapshot;
#[cfg(any(feature = "fmp", feature = "polygon", feature = "alphavantage"))]
pub use domains::{Market, MarketCalendar};

// Provider-specific financial data functions
// (FMP, Polygon, Alpha Vantage — defined in the finance module)
#[cfg(feature = "polygon")]
pub use finance::symbol_sentiment;
#[cfg(feature = "fmp")]
pub use finance::{
    AnalystEstimate, AnalystRecommendation, InsiderTransaction, Period, analyst_estimates,
    analyst_recommendations, insider_trading,
};
#[cfg(feature = "alphavantage")]
pub use finance::{EarningsCalendarEntry, IpoCalendarEntry, earnings_calendar, ipo_calendar};

pub use tickers::{
    BatchCapitalGainsResponse, BatchChartsResponse, BatchDividendsResponse,
    BatchFinancialsResponse, BatchNewsResponse, BatchOptionsResponse, BatchQuotesResponse,
    BatchRecommendationsResponse, BatchSparksResponse, BatchSplitsResponse, Tickers,
    TickersBuilder,
};

#[cfg(feature = "indicators")]
pub use tickers::BatchIndicatorsResponse;

// ============================================================================
// Error types and results
// ============================================================================
// Capability-routed response types (DISCOVERY / CALENDAR / MARKET)
#[cfg(any(feature = "fmp", feature = "polygon", feature = "alphavantage"))]
pub use models::calendar::market::{CalendarDetail, CalendarKind, MarketCalendarEntry};
#[cfg(any(feature = "fmp", feature = "polygon", feature = "alphavantage"))]
pub use models::discovery::reference::{
    ExchangeInfo, ScreenerFilters, ScreenerMatch, SymbolDetails, SymbolMatch,
};
#[cfg(any(feature = "fmp", feature = "polygon", feature = "alphavantage"))]
pub use models::market::performance::{
    IndustryPe, MoverDirection, MoverQuote, SectorPe, SectorPerformance, SectorPerformanceHistory,
};

pub use error::{ErrorCategory, FinanceError, Result};

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
    calendar::{CalendarEvent, EventKind},
    chart::Chart,
    chart::spark::Spark,
    corporate::governance::{EmployeeCount, ExecutiveCompensation},
    corporate::news::News,
    corporate::press_release::PressRelease,
    corporate::recommendation::Recommendation,
    corporate::transcript::{Transcript, TranscriptWithMeta},
    discovery::lookup::LookupResults,
    discovery::screeners::ScreenerResults,
    discovery::search::SearchResults,
    discovery::trending::TrendingQuote,
    filings::{
        CompanyFacts, EdgarSearchResults, EdgarSubmissions, FilingSearchFilters, FilingSearchHit,
        FilingSection, FilingSectionForm, InsiderTrade, InstitutionalHolding, ProviderFiling,
        ProviderFilings, RiskFactor,
    },
    fundamentals::{
        EtfHolding, EtfProfile, FinancialRatiosTtm, FinancialStatement, KeyMetricsTtm,
        PriceTargetConsensus, PriceTargetSummary, RatingConsensus, ShareFloat, ShortInterest,
        ShortVolume,
    },
    market::currencies::Currency,
    market::exchanges::Exchange,
    market::hours::MarketHours,
    market::industries::IndustryData,
    market::market_summary::MarketSummaryQuote,
    market::sectors::SectorData,
    options::Options,
    quote::Quote,
    quote::snapshot::{AssetClass, MarketSnapshot},
    sentiment::{FearAndGreed, FearGreedLabel, SymbolSentiment},
};
// Offline VADER sentiment scoring (feature-gated)
#[cfg(feature = "sentiment")]
pub use models::sentiment::{Sentiment, SentimentLabel, analyze as analyze_sentiment};
// Multi-provider capability response types (feature-gated)
#[cfg(any(feature = "fmp", feature = "alphavantage"))]
pub use models::commodities::CommodityQuote;
#[cfg(any(
    feature = "alphavantage",
    feature = "binance",
    feature = "crypto",
    feature = "defi",
    feature = "fmp",
    feature = "kraken",
    feature = "polygon"
))]
pub use models::crypto::CryptoQuote;
#[cfg(any(
    feature = "alphavantage",
    feature = "bls",
    feature = "fiscaldata",
    feature = "fred",
    feature = "polygon",
    feature = "worldbank"
))]
pub use models::economic::{
    EconomicCategory, EconomicRelease, EconomicSeries, EconomicSeriesMatch,
};
#[cfg(any(
    feature = "alphavantage",
    feature = "fmp",
    feature = "frankfurter",
    feature = "polygon"
))]
pub use models::forex::ForexQuote;
#[cfg(feature = "polygon")]
pub use models::futures::FuturesQuote;
#[cfg(any(feature = "polygon", feature = "fmp"))]
pub use models::indices::{IndexConstituent, IndexConstituentChange, IndexQuote, MajorIndex};

// ============================================================================
// Nested types - Commonly accessed fields within response types
// ============================================================================
pub use models::{
    chart::{Candle, CapitalGain, ChartMeta, Dividend, DividendAnalytics, Split},
    corporate::recommendation::SimilarSymbol,
    discovery::lookup::LookupQuote,
    discovery::screeners::ScreenerQuote,
    discovery::search::{
        ResearchReport, ResearchReports, SearchNews, SearchNewsList, SearchQuote, SearchQuotes,
    },
    filings::filing_index::{EdgarFilingIndex, EdgarFilingIndexItem},
    filings::{
        CikEntry, EdgarFiling, EdgarFilingFile, EdgarFilingRecent, EdgarFilings, EdgarSearchHit,
        EdgarSearchHitsContainer, EdgarSearchSource, EdgarSearchTotal, FactConcept, FactUnit,
        FactsByTaxonomy,
    },
    market::hours::MarketTime,
    market::market_summary::SparkData,
    options::{Contracts, OptionChain, OptionContract, OptionsQuote},
    quote::FormattedValue,
};

// ============================================================================
// Query builders - Types for constructing custom screener queries
// ============================================================================
pub use constants::exchange_codes::ExchangeCode;
pub use constants::industries::Industry;
pub use models::discovery::screeners::{
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
// Format type parameters — phantom types for compile-time format selection
// ============================================================================

/// Compile-time format type parameters for [`Quote`](crate::Quote) and other
/// `FormattedValue`-bearing structs.
///
/// | Marker | `F::Value<f64>` | Access pattern |
/// |---|---|---|
/// | [`format::Both`]  | `FormattedValue<f64>` | `.raw` / `.fmt` / `.long_fmt` |
/// | [`format::Raw`]   | `f64`                 | direct (no unwrapping) |
/// | [`format::Pretty`] | `String`             | human-readable string |
///
/// ```no_run
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// use finance_query::{format, Ticker};
/// let ticker = Ticker::new("AAPL").await?;
/// let quote: finance_query::Quote<format::Raw> = ticker.quote().await?;
/// # Ok(())
/// # }
/// ```
pub mod format {
    pub use crate::models::format::{Both, Pretty, Raw};
}

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
