//! Multi-provider financial data aggregation.

pub(crate) mod adapter;
pub mod config;
pub(crate) mod health;
pub(crate) mod retry;

#[cfg(test)]
pub(crate) mod mock;

#[cfg(feature = "alphavantage")]
pub(crate) mod alphavantage;
#[cfg(feature = "binance")]
pub(crate) mod binance;
#[cfg(feature = "bls")]
pub(crate) mod bls;
#[cfg(feature = "cftc")]
pub(crate) mod cftc;
#[cfg(feature = "crypto")]
pub(crate) mod coingecko;
#[cfg(any(feature = "housetrades", feature = "senatetrades"))]
pub(crate) mod congresstrades;
#[cfg(feature = "defi")]
pub(crate) mod defillama;
pub(crate) mod edgar;
#[cfg(feature = "finra")]
pub(crate) mod finra;
#[cfg(feature = "fiscaldata")]
pub(crate) mod fiscaldata;
#[cfg(feature = "fmp")]
pub(crate) mod fmp;
#[cfg(feature = "frankfurter")]
pub(crate) mod frankfurter;
#[cfg(feature = "fred")]
pub(crate) mod fred;
#[cfg(feature = "gdelt")]
pub(crate) mod gdelt;
#[cfg(feature = "kraken")]
pub(crate) mod kraken;
#[cfg(feature = "polygon")]
pub(crate) mod polygon;
pub(crate) mod types;
#[cfg(feature = "worldbank")]
pub(crate) mod worldbank;
pub(crate) mod yahoo;

use crate::adapters::yahoo::client::{ClientConfig, YahooClient};
use crate::error::{FinanceError, Result};
use futures::stream::StreamExt;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

pub(crate) use adapter::*;
/// Not part of the stable public API — see [`ProviderAdapter`].
#[doc(hidden)]
pub use adapter::{ChartProvider, ProviderAdapter, ProviderCore, QuoteProvider};
pub(crate) use health::HealthTracker;
pub use health::ProviderHealth;
pub use retry::RetryPolicy;

/// Typed identifier for a financial data provider.
///
/// Variants are feature-gated: unavailable providers are excluded at compile time.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Provider {
    #[default]
    /// Yahoo Finance (always available).
    Yahoo,
    /// Polygon.io (requires `polygon` feature).
    #[cfg(feature = "polygon")]
    Polygon,
    /// Financial Modeling Prep (requires `fmp` feature).
    #[cfg(feature = "fmp")]
    Fmp,
    /// Alpha Vantage (requires `alphavantage` feature).
    #[cfg(feature = "alphavantage")]
    AlphaVantage,
    /// CoinGecko cryptocurrency data (requires `crypto` feature).
    #[cfg(feature = "crypto")]
    CoinGecko,
    /// FRED economic data (requires `fred` feature).
    #[cfg(feature = "fred")]
    Fred,
    /// World Bank Open Data global macro indicators (requires `worldbank` feature, keyless).
    #[cfg(feature = "worldbank")]
    WorldBank,
    /// US Treasury FiscalData (requires `fiscaldata` feature, keyless).
    #[cfg(feature = "fiscaldata")]
    FiscalData,
    /// US Bureau of Labor Statistics (requires `bls` feature; keyless v1,
    /// keyed v2 when `BLS_API_KEY` is set).
    #[cfg(feature = "bls")]
    Bls,
    /// Frankfurter ECB reference exchange rates (requires `frankfurter` feature, keyless).
    #[cfg(feature = "frankfurter")]
    Frankfurter,
    /// Binance public market data (requires `binance` feature, keyless).
    #[cfg(feature = "binance")]
    Binance,
    /// Kraken public market data (requires `kraken` feature, keyless).
    #[cfg(feature = "kraken")]
    Kraken,
    /// FINRA daily short-sale volume (requires `finra` feature, keyless).
    #[cfg(feature = "finra")]
    Finra,
    /// DefiLlama DeFi TVL data (requires `defi` feature, keyless).
    #[cfg(feature = "defi")]
    DefiLlama,
    /// GDELT DOC 2.0 global news search (requires `gdelt` feature, keyless).
    #[cfg(feature = "gdelt")]
    Gdelt,
    /// CFTC Commitments of Traders futures positioning (requires `cftc`
    /// feature, keyless).
    #[cfg(feature = "cftc")]
    Cftc,
    /// Combined House + Senate PTR stock-trade disclosures (requires the
    /// `housetrades` and/or `senatetrades` feature, keyless).
    #[cfg(any(feature = "housetrades", feature = "senatetrades"))]
    CongressTrades,
    /// SEC EDGAR filings (always available, keyless).
    Edgar,
}

impl Provider {
    /// Parse a provider id string back to the typed variant.
    /// Returns `None` if the string doesn't match any known provider.
    /// Prefer this over string conversion to avoid panics.
    pub fn from_id_str(s: &str) -> Option<Self> {
        match s {
            "yahoo" => Some(Self::Yahoo),
            #[cfg(feature = "polygon")]
            "polygon" => Some(Self::Polygon),
            #[cfg(feature = "fmp")]
            "fmp" => Some(Self::Fmp),
            #[cfg(feature = "alphavantage")]
            "alphavantage" => Some(Self::AlphaVantage),
            #[cfg(feature = "crypto")]
            "coingecko" => Some(Self::CoinGecko),
            #[cfg(feature = "fred")]
            "fred" => Some(Self::Fred),
            #[cfg(feature = "worldbank")]
            "worldbank" => Some(Self::WorldBank),
            #[cfg(feature = "fiscaldata")]
            "fiscaldata" => Some(Self::FiscalData),
            #[cfg(feature = "bls")]
            "bls" => Some(Self::Bls),
            #[cfg(feature = "frankfurter")]
            "frankfurter" => Some(Self::Frankfurter),
            #[cfg(feature = "binance")]
            "binance" => Some(Self::Binance),
            #[cfg(feature = "kraken")]
            "kraken" => Some(Self::Kraken),
            #[cfg(feature = "finra")]
            "finra" => Some(Self::Finra),
            #[cfg(feature = "defi")]
            "defillama" => Some(Self::DefiLlama),
            #[cfg(feature = "gdelt")]
            "gdelt" => Some(Self::Gdelt),
            #[cfg(feature = "cftc")]
            "cftc" => Some(Self::Cftc),
            #[cfg(any(feature = "housetrades", feature = "senatetrades"))]
            "congresstrades" => Some(Self::CongressTrades),
            "edgar" => Some(Self::Edgar),
            _ => None,
        }
    }

    /// String identifier matching [`ProviderCore::id`].
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Yahoo => "yahoo",
            #[cfg(feature = "polygon")]
            Self::Polygon => "polygon",
            #[cfg(feature = "fmp")]
            Self::Fmp => "fmp",
            #[cfg(feature = "alphavantage")]
            Self::AlphaVantage => "alphavantage",
            #[cfg(feature = "crypto")]
            Self::CoinGecko => "coingecko",
            #[cfg(feature = "fred")]
            Self::Fred => "fred",
            #[cfg(feature = "worldbank")]
            Self::WorldBank => "worldbank",
            #[cfg(feature = "fiscaldata")]
            Self::FiscalData => "fiscaldata",
            #[cfg(feature = "bls")]
            Self::Bls => "bls",
            #[cfg(feature = "frankfurter")]
            Self::Frankfurter => "frankfurter",
            #[cfg(feature = "binance")]
            Self::Binance => "binance",
            #[cfg(feature = "kraken")]
            Self::Kraken => "kraken",
            #[cfg(feature = "finra")]
            Self::Finra => "finra",
            #[cfg(feature = "defi")]
            Self::DefiLlama => "defillama",
            #[cfg(feature = "gdelt")]
            Self::Gdelt => "gdelt",
            #[cfg(feature = "cftc")]
            Self::Cftc => "cftc",
            #[cfg(any(feature = "housetrades", feature = "senatetrades"))]
            Self::CongressTrades => "congresstrades",
            Self::Edgar => "edgar",
        }
    }

    /// Every provider variant compiled into this build, regardless of whether
    /// it's actually configured/initialized. Used to compute error-message
    /// hints (see [`Capability::candidate_providers`]) without needing a live
    /// `ProviderSet`.
    pub(crate) fn all() -> Vec<Self> {
        let mut v = vec![Self::Yahoo];
        #[cfg(feature = "polygon")]
        v.push(Self::Polygon);
        #[cfg(feature = "fmp")]
        v.push(Self::Fmp);
        #[cfg(feature = "alphavantage")]
        v.push(Self::AlphaVantage);
        #[cfg(feature = "crypto")]
        v.push(Self::CoinGecko);
        #[cfg(feature = "fred")]
        v.push(Self::Fred);
        #[cfg(feature = "worldbank")]
        v.push(Self::WorldBank);
        #[cfg(feature = "fiscaldata")]
        v.push(Self::FiscalData);
        #[cfg(feature = "bls")]
        v.push(Self::Bls);
        #[cfg(feature = "frankfurter")]
        v.push(Self::Frankfurter);
        #[cfg(feature = "binance")]
        v.push(Self::Binance);
        #[cfg(feature = "kraken")]
        v.push(Self::Kraken);
        #[cfg(feature = "finra")]
        v.push(Self::Finra);
        #[cfg(feature = "defi")]
        v.push(Self::DefiLlama);
        #[cfg(feature = "gdelt")]
        v.push(Self::Gdelt);
        #[cfg(feature = "cftc")]
        v.push(Self::Cftc);
        #[cfg(any(feature = "housetrades", feature = "senatetrades"))]
        v.push(Self::CongressTrades);
        v.push(Self::Edgar);
        v
    }

    /// Capability bitflags for this provider variant, derived from each
    /// adapter's `as_*` accessor overrides — implementing a capability trait
    /// and declaring it can no longer drift apart. Yahoo is the one exception:
    /// constructing `YahooProvider` needs a live auth handshake, so its set is
    /// a const declared beside its accessor overrides (`yahoo::CAPS`).
    pub(crate) fn capabilities(self) -> Capability {
        match self {
            Self::Yahoo => yahoo::CAPS,
            #[cfg(feature = "polygon")]
            Self::Polygon => ProviderAdapter::capabilities(&polygon::PolygonProvider),
            #[cfg(feature = "fmp")]
            Self::Fmp => ProviderAdapter::capabilities(&fmp::FmpProvider),
            #[cfg(feature = "alphavantage")]
            Self::AlphaVantage => {
                ProviderAdapter::capabilities(&alphavantage::AlphaVantageProvider)
            }
            #[cfg(feature = "crypto")]
            Self::CoinGecko => ProviderAdapter::capabilities(&coingecko::CoinGeckoProvider),
            #[cfg(feature = "fred")]
            Self::Fred => ProviderAdapter::capabilities(&fred::FredProvider),
            #[cfg(feature = "worldbank")]
            Self::WorldBank => ProviderAdapter::capabilities(&worldbank::WorldBankProvider),
            #[cfg(feature = "fiscaldata")]
            Self::FiscalData => ProviderAdapter::capabilities(&fiscaldata::FiscalDataProvider),
            #[cfg(feature = "bls")]
            Self::Bls => ProviderAdapter::capabilities(&bls::BlsProvider),
            #[cfg(feature = "frankfurter")]
            Self::Frankfurter => ProviderAdapter::capabilities(&frankfurter::FrankfurterProvider),
            #[cfg(feature = "binance")]
            Self::Binance => ProviderAdapter::capabilities(&binance::BinanceProvider),
            #[cfg(feature = "kraken")]
            Self::Kraken => ProviderAdapter::capabilities(&kraken::KrakenProvider),
            #[cfg(feature = "finra")]
            Self::Finra => ProviderAdapter::capabilities(&finra::FinraProvider),
            #[cfg(feature = "defi")]
            Self::DefiLlama => ProviderAdapter::capabilities(&defillama::DefiLlamaProvider),
            #[cfg(feature = "gdelt")]
            Self::Gdelt => ProviderAdapter::capabilities(&gdelt::GdeltProvider),
            #[cfg(feature = "cftc")]
            Self::Cftc => ProviderAdapter::capabilities(&cftc::CftcProvider),
            #[cfg(any(feature = "housetrades", feature = "senatetrades"))]
            Self::CongressTrades => {
                ProviderAdapter::capabilities(&congresstrades::CongressTradesProvider)
            }
            Self::Edgar => ProviderAdapter::capabilities(&edgar::EdgarProvider),
        }
    }
}

impl std::fmt::Display for Provider {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
/// How providers are queried.
pub enum Fetch {
    /// Try providers in priority order; first success wins.
    Sequential,
    /// Fire all providers concurrently; first success wins.
    Parallel,
}

/// Capability bits that a provider can declare.
///
/// Route a capability to specific providers using `.route(Capability::QUOTE, [Provider::Fmp])`.
/// If no route is configured for a capability, all providers declaring that capability are used.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Capability(u32);

impl Capability {
    /// Equity quote data — price, volume, market cap, fundamentals summary.
    pub const QUOTE: Self = Self(1 << 0);
    /// Historical OHLCV chart data across intervals and ranges.
    pub const CHART: Self = Self(1 << 1);
    /// Financial statements — income, balance sheet, cash flow.
    pub const FUNDAMENTALS: Self = Self(1 << 2);
    /// Corporate events — news, recommendations, SEC filings metadata.
    pub const CORPORATE: Self = Self(1 << 3);
    /// Options chains and contract data.
    pub const OPTIONS: Self = Self(1 << 4);
    /// Symbol discovery — search, screeners, exchange and ticker reference data.
    pub const DISCOVERY: Self = Self(1 << 5);

    /// Cryptocurrency quotes and market data.
    pub const CRYPTO: Self = Self(1 << 6);
    /// Macro-economic data series (FRED, GDP, CPI, etc.).
    pub const ECONOMIC: Self = Self(1 << 7);
    /// Market-wide calendars — earnings, IPOs, dividends, splits, economic events.
    pub const CALENDAR: Self = Self(1 << 8);

    /// Foreign exchange currency pair quotes.
    pub const FOREX: Self = Self(1 << 9);
    /// Stock market index quotes (S&P 500, NASDAQ, etc.).
    pub const INDICES: Self = Self(1 << 10);
    /// Futures contract quotes.
    pub const FUTURES: Self = Self(1 << 11);
    /// Commodity price quotes (gold, oil, etc.).
    pub const COMMODITIES: Self = Self(1 << 12);
    /// Market-wide statistics — sector/industry performance and movers.
    pub const MARKET: Self = Self(1 << 13);

    /// SEC EDGAR filing data.
    pub const FILINGS: Self = Self(1 << 14);

    /// The empty capability set — starting point for derived accumulation.
    pub(crate) const NONE: Self = Self(0);

    /// Const-context union, for capability-set consts ([`std::ops::BitOr`]
    /// isn't const-callable).
    pub(crate) const fn union(self, other: Self) -> Self {
        Self(self.0 | other.0)
    }

    /// Returns `true` if this capability set includes all bits in `other`.
    pub const fn contains(self, other: Self) -> bool {
        (self.0 & other.0) == other.0
    }

    /// Every single-bit capability paired with its name — the one place names
    /// and bits meet. `name()`, `Display`, and the bit-uniqueness test all
    /// derive from this table.
    const ALL: [(Self, &'static str); 15] = [
        (Self::QUOTE, "quote"),
        (Self::CHART, "chart"),
        (Self::FUNDAMENTALS, "fundamentals"),
        (Self::CORPORATE, "corporate"),
        (Self::OPTIONS, "options"),
        (Self::DISCOVERY, "discovery"),
        (Self::CRYPTO, "crypto"),
        (Self::ECONOMIC, "economic"),
        (Self::CALENDAR, "calendar"),
        (Self::MARKET, "market"),
        (Self::FOREX, "forex"),
        (Self::INDICES, "indices"),
        (Self::FUTURES, "futures"),
        (Self::COMMODITIES, "commodities"),
        (Self::FILINGS, "filings"),
    ];

    /// Returns a short lowercase name for this capability (e.g., `"quote"`, `"chart"`).
    ///
    /// Returns `"unknown"` for combined capability flags or unrecognised bits;
    /// [`Display`](std::fmt::Display) spells combined sets out instead (e.g.
    /// `"quote|chart"`).
    pub fn name(self) -> &'static str {
        Self::ALL
            .iter()
            .find(|(cap, _)| cap.0 == self.0)
            .map(|(_, name)| *name)
            .unwrap_or("unknown")
    }
}

impl std::fmt::Display for Capability {
    /// Single capabilities print their [`name`](Capability::name); combined
    /// sets are spelled out `|`-separated (e.g. `"quote|chart"`) rather than
    /// collapsing to `"unknown"`.
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut remaining = self.0;
        let mut first = true;
        for (cap, name) in Self::ALL {
            if remaining & cap.0 != 0 {
                if !first {
                    f.write_str("|")?;
                }
                f.write_str(name)?;
                first = false;
                remaining &= !cap.0;
            }
        }
        if first || remaining != 0 {
            if !first {
                f.write_str("|")?;
            }
            f.write_str("unknown")?;
        }
        Ok(())
    }
}

impl Capability {
    /// Providers whose `capabilities()` declare this capability, regardless of
    /// which providers are actually configured/feature-enabled in this build.
    ///
    /// Purely informational — used to make [`crate::FinanceError::NotSupported`]/
    /// [`crate::FinanceError::NoProviderAvailable`] point at what would need to
    /// be enabled (feature flag) and/or routed (`Providers::builder().route(...)`).
    pub(crate) fn candidate_providers(self) -> Vec<Provider> {
        Provider::all()
            .into_iter()
            .filter(|p| p.capabilities().contains(self))
            .collect()
    }
}

impl std::ops::BitOr for Capability {
    type Output = Self;
    fn bitor(self, rhs: Self) -> Self {
        Self(self.0 | rhs.0)
    }
}

/// A single provider-adapter operation — finer-grained than [`Capability`]
/// (e.g. `Chart`, `ChartRange`, and `Spark` all fall under `Capability::CHART`).
///
/// Used in [`crate::FinanceError::NotSupported`] to say exactly which method a
/// provider doesn't implement; [`Operation::capability`] recovers the coarser
/// bit for computing which other providers could satisfy it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum Operation {
    /// Single-symbol quote.
    Quote,
    /// Historical OHLCV chart over an interval/range.
    Chart,
    /// Historical OHLCV chart over a custom timestamp range.
    ChartRange,
    /// Financial statements (income/balance/cash flow).
    Financials,
    /// Symbol news.
    News,
    /// Similar-symbol recommendations.
    Recommendations,
    /// Options chain.
    Options,
    /// Corporate calendar events (earnings, dividends, splits).
    Events,
    /// Batch quotes for multiple symbols in one request.
    QuotesBatch,
    /// Lightweight sparkline data for multiple symbols in one request.
    Spark,
    /// Cryptocurrency quote.
    CryptoQuote,
    /// Market-wide crypto news.
    CryptoNews,
    /// Macro-economic data series.
    EconomicSeries,
    /// Foreign exchange currency pair quote.
    ForexQuote,
    /// Market-wide forex news.
    ForexNews,
    /// Stock market index quote.
    IndicesQuote,
    /// Futures contract quote.
    FuturesQuote,
    /// Commodity price quote.
    CommoditiesQuote,
    /// SEC EDGAR filing data.
    Filings,
    /// Symbol search by free-text query.
    SymbolSearch,
    /// Detailed reference data for one symbol.
    SymbolDetails,
    /// Tradable exchange listing.
    Exchanges,
    /// Screener query over the provider's universe.
    Screener,
    /// Market-wide earnings calendar.
    EarningsCalendar,
    /// Market-wide IPO calendar.
    IpoCalendar,
    /// Market-wide dividend calendar.
    DividendCalendar,
    /// Market-wide stock split calendar.
    SplitCalendar,
    /// Market-wide economic event calendar.
    EconomicCalendar,
    /// Sector and industry performance statistics.
    SectorPerformance,
    /// Market movers — gainers, losers, most active.
    MarketMovers,
    /// Historical sector performance.
    SectorPerformanceHistory,
    /// Market-wide holiday calendar.
    HolidayCalendar,
    /// Live exchange open/closed status.
    MarketStatus,
    /// Current constituents of a major index.
    IndexConstituents,
    /// Historical constituent changes of a major index.
    IndexConstituentChanges,
    /// Short interest (settlement-date positions).
    ShortInterest,
    /// Daily short volume.
    ShortVolume,
    /// Share float and shares outstanding.
    ShareFloat,
    /// Sectioned text of an SEC filing.
    FilingSections,
    /// Risk factors extracted from SEC filings.
    RiskFactors,
    /// Company press releases.
    PressReleases,
    /// Earnings call transcript, provider-neutral shape.
    EarningsTranscript,
    /// Total value locked in a DeFi protocol.
    #[cfg(feature = "defi")]
    ProtocolTvl,
    /// Historical total value locked in a DeFi protocol.
    #[cfg(feature = "defi")]
    ProtocolTvlHistory,
    /// Weekly CFTC Commitments of Traders futures positioning.
    #[cfg(feature = "cftc")]
    CommitmentsOfTraders,
    /// Aggregated analyst price-target consensus.
    PriceTargetConsensus,
    /// Price-target publication activity over trailing windows.
    PriceTargetSummary,
    /// Aggregated analyst rating consensus.
    RatingConsensus,
    /// Trailing-twelve-month key-metrics snapshot.
    KeyMetricsTtm,
    /// Trailing-twelve-month ratios snapshot.
    RatiosTtm,
    /// Reported executive compensation.
    ExecutiveCompensation,
    /// Reported employee headcount.
    EmployeeCount,
    /// Company identity/classification profile.
    CompanyProfile,
    /// Cross-market snapshot for symbols spanning several asset classes.
    UnifiedSnapshot,
    /// Full-text search over filing content.
    FilingSearch,
    /// Insider transactions reported on Forms 3/4/5.
    InsiderTrades,
    /// Institutional holdings reported on Form 13F-HR.
    InstitutionalHoldings,
    /// Congressional (senate) stock-trade disclosures.
    CongressionalTrades,
    /// SEC fails-to-deliver data.
    FailsToDeliver,
    /// A macro series as it stood on a past date (vintage/ALFRED view).
    EconomicSeriesAsOf,
    /// Free-text search over the macro series catalog.
    EconomicSearch,
    /// Macro series category browsing.
    EconomicCategories,
    /// Scheduled macro data releases.
    EconomicReleases,
    /// ETF profile and portfolio holdings.
    EtfProfile,
    /// Earnings-surprise history.
    EarningsSurprises,
    /// Raw per-analyst grade-action history.
    GradingHistory,
    /// The provider's whole listed-security universe.
    ListingStatus,
    /// Grouped daily OHLCV bars for every stock ticker on one date.
    GroupedDaily,
    /// Grouped daily OHLCV bars for every crypto ticker on one date.
    CryptoGroupedDaily,
    /// Grouped daily OHLCV bars for every forex ticker on one date.
    ForexGroupedDaily,
    /// Coins/nfts/categories trending in the last 24h.
    #[cfg(feature = "crypto")]
    CryptoTrending,
    /// Aggregate global cryptocurrency market statistics.
    #[cfg(feature = "crypto")]
    CryptoGlobal,
}

impl Operation {
    /// Short lowercase identifier (e.g. `"chart_range"`, `"crypto_quote"`).
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Quote => "quote",
            Self::Chart => "chart",
            Self::ChartRange => "chart_range",
            Self::Financials => "financials",
            Self::News => "news",
            Self::Recommendations => "recommendations",
            Self::Options => "options",
            Self::Events => "events",
            Self::QuotesBatch => "quotes_batch",
            Self::Spark => "spark",
            Self::CryptoQuote => "crypto_quote",
            Self::CryptoNews => "crypto_news",
            Self::EconomicSeries => "economic_series",
            Self::ForexQuote => "forex_quote",
            Self::ForexNews => "forex_news",
            Self::IndicesQuote => "indices_quote",
            Self::FuturesQuote => "futures_quote",
            Self::CommoditiesQuote => "commodities_quote",
            Self::Filings => "filings",
            Self::SymbolSearch => "symbol_search",
            Self::SymbolDetails => "symbol_details",
            Self::Exchanges => "exchanges",
            Self::Screener => "screener",
            Self::EarningsCalendar => "earnings_calendar",
            Self::IpoCalendar => "ipo_calendar",
            Self::DividendCalendar => "dividend_calendar",
            Self::SplitCalendar => "split_calendar",
            Self::EconomicCalendar => "economic_calendar",
            Self::SectorPerformance => "sector_performance",
            Self::MarketMovers => "market_movers",
            Self::SectorPerformanceHistory => "sector_performance_history",
            Self::HolidayCalendar => "holiday_calendar",
            Self::MarketStatus => "market_status",
            Self::IndexConstituents => "index_constituents",
            Self::IndexConstituentChanges => "index_constituent_changes",
            Self::ShortInterest => "short_interest",
            Self::ShortVolume => "short_volume",
            Self::ShareFloat => "share_float",
            Self::FilingSections => "filing_sections",
            Self::RiskFactors => "risk_factors",
            Self::PressReleases => "press_releases",
            Self::EarningsTranscript => "earnings_transcript",
            #[cfg(feature = "defi")]
            Self::ProtocolTvl => "protocol_tvl",
            #[cfg(feature = "defi")]
            Self::ProtocolTvlHistory => "protocol_tvl_history",
            #[cfg(feature = "cftc")]
            Self::CommitmentsOfTraders => "commitments_of_traders",
            Self::PriceTargetConsensus => "price_target_consensus",
            Self::PriceTargetSummary => "price_target_summary",
            Self::RatingConsensus => "rating_consensus",
            Self::KeyMetricsTtm => "key_metrics_ttm",
            Self::RatiosTtm => "ratios_ttm",
            Self::ExecutiveCompensation => "executive_compensation",
            Self::EmployeeCount => "employee_count",
            Self::CompanyProfile => "company_profile",
            Self::UnifiedSnapshot => "unified_snapshot",
            Self::FilingSearch => "filing_search",
            Self::InsiderTrades => "insider_trades",
            Self::InstitutionalHoldings => "institutional_holdings",
            Self::CongressionalTrades => "congressional_trades",
            Self::FailsToDeliver => "fails_to_deliver",
            Self::EconomicSeriesAsOf => "economic_series_as_of",
            Self::EconomicSearch => "economic_search",
            Self::EconomicCategories => "economic_categories",
            Self::EconomicReleases => "economic_releases",
            Self::EtfProfile => "etf_profile",
            Self::EarningsSurprises => "earnings_surprises",
            Self::GradingHistory => "grading_history",
            Self::ListingStatus => "listing_status",
            Self::GroupedDaily => "grouped_daily",
            Self::CryptoGroupedDaily => "crypto_grouped_daily",
            Self::ForexGroupedDaily => "forex_grouped_daily",
            #[cfg(feature = "crypto")]
            Self::CryptoTrending => "crypto_trending",
            #[cfg(feature = "crypto")]
            Self::CryptoGlobal => "crypto_global",
        }
    }

    /// The coarser [`Capability`] bit this operation falls under.
    pub fn capability(self) -> Capability {
        match self {
            Self::Quote | Self::QuotesBatch | Self::UnifiedSnapshot => Capability::QUOTE,
            Self::Chart
            | Self::ChartRange
            | Self::Spark
            | Self::GroupedDaily
            | Self::CryptoGroupedDaily
            | Self::ForexGroupedDaily => Capability::CHART,
            Self::Financials
            | Self::ShortInterest
            | Self::ShortVolume
            | Self::ShareFloat
            | Self::PriceTargetConsensus
            | Self::PriceTargetSummary
            | Self::RatingConsensus
            | Self::KeyMetricsTtm
            | Self::RatiosTtm
            | Self::EtfProfile
            | Self::EarningsSurprises
            | Self::GradingHistory
            | Self::CompanyProfile => Capability::FUNDAMENTALS,
            Self::News
            | Self::Recommendations
            | Self::Events
            | Self::PressReleases
            | Self::ExecutiveCompensation
            | Self::EmployeeCount
            | Self::EarningsTranscript => Capability::CORPORATE,
            Self::Options => Capability::OPTIONS,
            Self::CryptoQuote | Self::CryptoNews => Capability::CRYPTO,
            #[cfg(feature = "defi")]
            Self::ProtocolTvl | Self::ProtocolTvlHistory => Capability::CRYPTO,
            #[cfg(feature = "crypto")]
            Self::CryptoTrending | Self::CryptoGlobal => Capability::CRYPTO,
            Self::EconomicSeries
            | Self::EconomicSeriesAsOf
            | Self::EconomicSearch
            | Self::EconomicCategories
            | Self::EconomicReleases => Capability::ECONOMIC,
            Self::ForexQuote | Self::ForexNews => Capability::FOREX,
            Self::IndicesQuote | Self::IndexConstituents | Self::IndexConstituentChanges => {
                Capability::INDICES
            }
            Self::FuturesQuote => Capability::FUTURES,
            #[cfg(feature = "cftc")]
            Self::CommitmentsOfTraders => Capability::FUTURES,
            Self::CommoditiesQuote => Capability::COMMODITIES,
            Self::Filings
            | Self::FilingSections
            | Self::RiskFactors
            | Self::FilingSearch
            | Self::InsiderTrades
            | Self::InstitutionalHoldings
            | Self::CongressionalTrades
            | Self::FailsToDeliver => Capability::FILINGS,
            Self::SymbolSearch
            | Self::SymbolDetails
            | Self::Exchanges
            | Self::Screener
            | Self::ListingStatus => Capability::DISCOVERY,
            Self::EarningsCalendar
            | Self::IpoCalendar
            | Self::DividendCalendar
            | Self::SplitCalendar
            | Self::EconomicCalendar
            | Self::HolidayCalendar
            | Self::MarketStatus => Capability::CALENDAR,
            Self::SectorPerformance | Self::MarketMovers | Self::SectorPerformanceHistory => {
                Capability::MARKET
            }
        }
    }

    /// The error reported when `provider` does not serve this operation.
    ///
    /// Adapters reach for this when they detect the gap before dispatch has a
    /// `&dyn ProviderAdapter` to hand (e.g. a symbol an exchange cannot name);
    /// `ProviderCore::not_supported` is the same error built from an instance.
    pub(crate) fn not_supported(self, provider: Provider) -> FinanceError {
        FinanceError::NotSupported {
            provider,
            operation: self,
            candidates: self.capability().candidate_providers(),
        }
    }
}

impl std::fmt::Display for Operation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// Per-capability provider routing table.
///
/// Maps each [`Capability`] to an ordered list of [`Provider`]s to try.
/// When a capability has no entry, all providers declaring that capability are used.
#[derive(Debug)]
pub struct Routes {
    pub(crate) map: HashMap<Capability, Vec<Provider>>,
    pub(crate) fetch: Fetch,
}

impl Routes {
    /// An empty route table (every capability falls back to its default
    /// candidate providers) using the given concurrency [`Fetch`] mode.
    pub fn new(fetch: Fetch) -> Self {
        Self {
            map: HashMap::new(),
            fetch,
        }
    }
}

/// Not part of the stable public API — exposed only so `benches/soothfast.rs`
/// can inject a canned, network-free adapter for gating `Ticker`/`Tickers`
/// hot paths. No semver guarantees; may change or move without notice.
#[doc(hidden)]
pub struct ProviderSet {
    providers: Vec<Arc<dyn ProviderAdapter>>,
    yahoo_client: Option<Arc<YahooClient>>,
    routes: Routes,
    /// Opt-in retry policy. `None` (the default) preserves the prior
    /// behavior exactly: a `RateLimited` error is treated like any
    /// other failure and dispatch moves straight to the next candidate.
    retry_policy: Option<RetryPolicy>,
    /// In-memory recent success/failure tracker, always active (cheap to
    /// maintain) and surfaced on request via [`ProviderSet::health`].
    health: HealthTracker,
}

impl std::fmt::Debug for ProviderSet {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ProviderSet")
            .field(
                "providers",
                &self.providers.iter().map(|p| p.id()).collect::<Vec<_>>(),
            )
            .field("has_yahoo_client", &self.yahoo_client.is_some())
            .field("routes", &self.routes)
            .finish()
    }
}

impl ProviderSet {
    /// `YahooClient` stays crate-private (benches pass `None`); the leak in
    /// this doc-hidden constructor's signature is deliberate.
    #[allow(private_interfaces)]
    pub fn new(
        providers: Vec<Arc<dyn ProviderAdapter>>,
        yahoo_client: Option<Arc<YahooClient>>,
        routes: Routes,
    ) -> Self {
        Self {
            providers,
            yahoo_client,
            routes,
            retry_policy: None,
            health: HealthTracker::new(),
        }
    }

    /// Opt into [`RetryPolicy`]-driven retry of `RateLimited` errors.
    /// `None` (the default from [`new`](Self::new)) preserves the prior
    /// no-retry behavior exactly.
    pub(crate) fn with_retry_policy(mut self, policy: Option<RetryPolicy>) -> Self {
        self.retry_policy = policy;
        self
    }

    /// Snapshot recent health for every configured provider, in the order
    /// they were added (see [`crate::Providers::health`]).
    pub(crate) fn health(&self) -> Vec<ProviderHealth> {
        self.providers
            .iter()
            .map(|p| {
                let mut snapshot = self.health.snapshot(p.id());
                snapshot.requests_remaining_estimate = p.rate_limit_remaining();
                snapshot
            })
            .collect()
    }

    /// Returns the providers to use for a given capability, respecting any
    /// explicit route configured via `.route()`. When no route is configured,
    /// defaults to Yahoo for all capabilities and EDGAR for filings.
    fn candidates_for(&self, cap: Capability) -> Vec<&Arc<dyn ProviderAdapter>> {
        if let Some(provider_ids) = self.routes.map.get(&cap) {
            provider_ids
                .iter()
                .filter_map(|id| self.providers.iter().find(|p| p.id() == *id))
                .collect()
        } else if cap == Capability::FILINGS {
            // Default: EDGAR (keyless SEC filings) first, then Yahoo
            let mut v: Vec<&Arc<dyn ProviderAdapter>> = self
                .providers
                .iter()
                .filter(|p| p.id() == Provider::Edgar)
                .collect();
            v.extend(self.providers.iter().filter(|p| p.id() == Provider::Yahoo));
            v
        } else {
            // Default: Yahoo only
            self.providers
                .iter()
                .filter(|p| p.id() == Provider::Yahoo)
                .collect()
        }
    }

    fn no_provider(cap: Capability) -> FinanceError {
        FinanceError::NoProviderAvailable {
            operation: cap,
            candidates: cap.candidate_providers(),
        }
    }

    /// Real provider failures outrank `NotSupported` (which just means "next
    /// candidate"), but when *every* candidate lacked the operation, surface
    /// the precise per-operation `NotSupported` instead of collapsing to a
    /// capability-level `NoProviderAvailable`.
    fn finish_err(
        cap: Capability,
        last: Option<FinanceError>,
        unsupported: Option<FinanceError>,
    ) -> FinanceError {
        last.or(unsupported)
            .unwrap_or_else(|| Self::no_provider(cap))
    }

    /// Call `f(p)`, retrying in place on `FinanceError::RateLimited` per
    /// `self.retry_policy`. With no policy configured this is
    /// exactly `f(p).await` — zero behavior change for existing callers.
    async fn call_with_retry<T, F, Fut>(&self, p: &Arc<dyn ProviderAdapter>, f: &F) -> Result<T>
    where
        F: Fn(&Arc<dyn ProviderAdapter>) -> Fut,
        Fut: std::future::Future<Output = Result<T>>,
    {
        let Some(policy) = self.retry_policy.as_ref() else {
            return f(p).await;
        };
        let mut seed = crate::backoff::seed_from_time();
        let mut attempt: u32 = 0;
        loop {
            match f(p).await {
                Ok(v) => return Ok(v),
                Err(FinanceError::RateLimited { retry_after })
                    if attempt + 1 < policy.max_attempts =>
                {
                    let delay =
                        policy.delay_for(attempt, retry_after.map(Duration::from_secs), &mut seed);
                    tokio::time::sleep(delay).await;
                    attempt += 1;
                }
                Err(e) => return Err(e),
            }
        }
    }

    /// Record a candidate's outcome in the health tracker. `NotSupported`
    /// isn't a provider failure (it just means "try the next candidate"), so
    /// it's excluded from health accounting entirely.
    fn record_health<T>(&self, provider: Provider, result: &Result<T>) {
        match result {
            Ok(_) => self.health.record(provider, true, None),
            Err(FinanceError::NotSupported { .. }) => {}
            Err(e) => self.health.record(provider, false, Some(e.to_string())),
        }
    }

    pub(crate) async fn fetch<T, F, Fut>(&self, cap: Capability, f: F) -> Result<T>
    where
        F: Fn(&Arc<dyn ProviderAdapter>) -> Fut,
        Fut: std::future::Future<Output = Result<T>>,
    {
        let candidates = self.candidates_for(cap);
        if candidates.is_empty() {
            return Err(Self::no_provider(cap));
        }
        match self.routes.fetch {
            Fetch::Sequential => {
                let mut last = None;
                let mut unsupported = None;
                for p in &candidates {
                    let result = self.call_with_retry(p, &f).await;
                    self.record_health(p.id(), &result);
                    match result {
                        Ok(v) => return Ok(v),
                        Err(e @ FinanceError::NotSupported { .. }) => unsupported = Some(e),
                        Err(e) => last = Some(e),
                    }
                }
                Err(Self::finish_err(cap, last, unsupported))
            }
            Fetch::Parallel => {
                let mut futs = futures::stream::FuturesUnordered::new();
                for p in &candidates {
                    let id = p.id();
                    // Capture `&f` (Copy) rather than `f` itself — `f: F` isn't
                    // necessarily `Copy`, and `async move` would otherwise try
                    // (and fail) to move it out on every loop iteration.
                    let f_ref = &f;
                    futs.push(async move {
                        let result = self.call_with_retry(p, f_ref).await;
                        self.record_health(id, &result);
                        result
                    });
                }
                let mut last = None;
                let mut unsupported = None;
                while let Some(r) = futs.next().await {
                    match r {
                        Ok(v) => return Ok(v),
                        Err(e @ FinanceError::NotSupported { .. }) => unsupported = Some(e),
                        Err(e) => last = Some(e),
                    }
                }
                Err(Self::finish_err(cap, last, unsupported))
            }
        }
    }

    pub(crate) fn first_yahoo(&self) -> Result<Arc<YahooClient>> {
        self.yahoo_client.as_ref().map(Arc::clone).ok_or_else(|| {
            FinanceError::NoProviderAvailable {
                operation: Capability::QUOTE,
                candidates: vec![Provider::Yahoo],
            }
        })
    }
}

#[allow(dead_code)] // used by fmp, polygon, alphavantage feature-gated providers
pub(crate) fn json_value_to_f64(value: serde_json::Value) -> Option<f64> {
    value
        .as_f64()
        .or_else(|| value.as_i64().map(|v| v as f64))
        .or_else(|| value.as_u64().map(|v| v as f64))
        .or_else(|| value.as_str().and_then(|s| s.parse::<f64>().ok()))
        .or_else(|| {
            value
                .get("raw")
                .and_then(|raw| raw.as_f64().or_else(|| raw.as_i64().map(|v| v as f64)))
        })
}

#[allow(dead_code)] // used by fmp, polygon, alphavantage feature-gated providers
pub(crate) fn build_financial_statement(
    symbol: String,
    statement_type: String,
    frequency: String,
    provider_id: Provider,
    data: std::collections::HashMap<String, std::collections::HashMap<String, serde_json::Value>>,
) -> crate::models::fundamentals::FinancialStatement {
    let statement = data
        .into_iter()
        .filter_map(|(metric, values)| {
            let values: std::collections::HashMap<String, f64> = values
                .into_iter()
                .filter_map(|(date, value)| json_value_to_f64(value).map(|v| (date, v)))
                .collect();
            if values.is_empty() {
                None
            } else {
                Some((metric, values))
            }
        })
        .collect();
    crate::models::fundamentals::FinancialStatement {
        symbol,
        statement_type,
        frequency,
        statement,
        provider_id: Some(provider_id),
    }
}

pub(crate) fn build_options(
    symbol: String,
    provider_id: Provider,
    expiration_dates: Vec<i64>,
    calls: Vec<crate::models::options::OptionContract>,
    puts: Vec<crate::models::options::OptionContract>,
) -> crate::models::options::Options {
    use std::collections::BTreeMap;

    let mut chains_by_expiration: BTreeMap<
        i64,
        (
            Vec<crate::models::options::OptionContract>,
            Vec<crate::models::options::OptionContract>,
        ),
    > = BTreeMap::new();

    for contract in calls {
        let exp = contract.expiration.unwrap_or(0);
        chains_by_expiration
            .entry(exp)
            .or_default()
            .0
            .push(contract);
    }
    for contract in puts {
        let exp = contract.expiration.unwrap_or(0);
        chains_by_expiration
            .entry(exp)
            .or_default()
            .1
            .push(contract);
    }

    let option_chains: Vec<crate::models::options::response::OptionChainData> =
        chains_by_expiration
            .into_iter()
            .map(
                |(expiration, (c, p))| crate::models::options::response::OptionChainData {
                    expiration_date: expiration,
                    has_mini_options: None,
                    calls: Some(c),
                    puts: Some(p),
                },
            )
            .collect();

    let expiration_dates = if expiration_dates.is_empty() {
        option_chains
            .iter()
            .map(|chain| chain.expiration_date)
            .collect()
    } else {
        let mut v: Vec<i64> = expiration_dates;
        v.sort_unstable();
        v.dedup();
        v
    };

    let mut strikes: Vec<f64> = option_chains
        .iter()
        .flat_map(|chain| {
            chain
                .calls
                .as_deref()
                .unwrap_or_default()
                .iter()
                .map(|c| c.strike)
                .chain(
                    chain
                        .puts
                        .as_deref()
                        .unwrap_or_default()
                        .iter()
                        .map(|p| p.strike),
                )
        })
        .collect();
    strikes.sort_by(|a, b| a.total_cmp(b));
    strikes.dedup_by(|a, b| a.total_cmp(b).is_eq());

    let result = crate::models::options::response::OptionChainResult {
        underlying_symbol: Some(symbol),
        expiration_dates: Some(expiration_dates),
        strikes: Some(strikes),
        has_mini_options: None,
        quote: None,
        options: option_chains,
    };

    crate::models::options::Options {
        option_chain: crate::models::options::response::OptionChainContainer {
            result: vec![result],
            error: None,
        },
        provider_id: Some(provider_id),
    }
}

#[allow(dead_code)] // used by fmp feature-gated provider
pub(crate) fn range_to_dates(range: crate::TimeRange) -> (String, String) {
    use chrono::{Datelike, Utc};
    let end = Utc::now();
    if range == crate::TimeRange::YearToDate {
        let year = end.year();
        let start = chrono::NaiveDate::from_ymd_opt(year, 1, 1)
            .and_then(|d| d.and_hms_opt(0, 0, 0))
            .map(|dt| dt.and_utc())
            .unwrap_or(end);
        return (
            start.format("%Y-%m-%d").to_string(),
            end.format("%Y-%m-%d").to_string(),
        );
    }
    let days = match range {
        crate::TimeRange::OneDay => 1,
        crate::TimeRange::FiveDays => 5,
        crate::TimeRange::OneMonth => 30,
        crate::TimeRange::ThreeMonths => 90,
        crate::TimeRange::SixMonths => 180,
        crate::TimeRange::OneYear => 365,
        crate::TimeRange::TwoYears => 730,
        crate::TimeRange::FiveYears => 1825,
        crate::TimeRange::TenYears => 3650,
        crate::TimeRange::Max => 36500,
        crate::TimeRange::YearToDate => unreachable!("YTD handled by early return above"),
    };
    let start = end - chrono::Duration::days(days);
    (
        start.format("%Y-%m-%d").to_string(),
        end.format("%Y-%m-%d").to_string(),
    )
}

/// Keyed on identity, not `capabilities().contains(FILINGS)`: a provider may
/// advertise FILINGS only to reach a secondary op (Alpha Vantage does, for
/// insider trades), which would otherwise suppress the real filings source.
fn needs_edgar_injection(ids: &[Provider]) -> bool {
    !ids.contains(&Provider::Edgar)
}

pub(crate) async fn build_providers(
    ids: &[Provider],
    config: &ClientConfig,
    routes: Routes,
) -> Result<ProviderSet> {
    use yahoo::YahooProvider;
    let mut providers: Vec<Arc<dyn ProviderAdapter>> = Vec::new();
    let mut yahoo_client: Option<Arc<YahooClient>> = None;
    for &id in ids {
        let adapter: Arc<dyn ProviderAdapter> = match id {
            Provider::Yahoo => {
                let yp = YahooProvider::new(config).await?;
                yahoo_client = Some(yp.client_arc());
                Arc::new(yp)
            }
            #[cfg(feature = "polygon")]
            Provider::Polygon => Arc::new(polygon::PolygonProvider),
            #[cfg(feature = "fmp")]
            Provider::Fmp => Arc::new(fmp::FmpProvider),
            #[cfg(feature = "alphavantage")]
            Provider::AlphaVantage => Arc::new(alphavantage::AlphaVantageProvider),
            #[cfg(feature = "crypto")]
            Provider::CoinGecko => Arc::new(coingecko::CoinGeckoProvider),
            #[cfg(feature = "fred")]
            Provider::Fred => Arc::new(fred::FredProvider),
            #[cfg(feature = "worldbank")]
            Provider::WorldBank => Arc::new(worldbank::WorldBankProvider),
            #[cfg(feature = "fiscaldata")]
            Provider::FiscalData => Arc::new(fiscaldata::FiscalDataProvider),
            #[cfg(feature = "bls")]
            Provider::Bls => Arc::new(bls::BlsProvider),
            #[cfg(feature = "frankfurter")]
            Provider::Frankfurter => Arc::new(frankfurter::FrankfurterProvider),
            #[cfg(feature = "binance")]
            Provider::Binance => Arc::new(binance::BinanceProvider),
            #[cfg(feature = "kraken")]
            Provider::Kraken => Arc::new(kraken::KrakenProvider),
            #[cfg(feature = "finra")]
            Provider::Finra => Arc::new(finra::FinraProvider),
            #[cfg(feature = "defi")]
            Provider::DefiLlama => Arc::new(defillama::DefiLlamaProvider),
            #[cfg(feature = "gdelt")]
            Provider::Gdelt => Arc::new(gdelt::GdeltProvider),
            #[cfg(feature = "cftc")]
            Provider::Cftc => Arc::new(cftc::CftcProvider),
            #[cfg(any(feature = "housetrades", feature = "senatetrades"))]
            Provider::CongressTrades => Arc::new(congresstrades::CongressTradesProvider),
            Provider::Edgar => Arc::new(edgar::EdgarProvider),
        };
        adapter.initialize().await?;
        providers.push(adapter);
    }
    if needs_edgar_injection(ids) {
        providers.push(Arc::new(edgar::EdgarProvider));
    }
    Ok(ProviderSet::new(providers, yahoo_client, routes))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A CHART-capable provider that does not implement spark — exercises the
    /// default trait method and proves spark now dispatches through the set.
    struct NoSparkProvider;

    impl ProviderCore for NoSparkProvider {
        fn id(&self) -> Provider {
            Provider::Yahoo
        }
    }

    #[async_trait::async_trait]
    impl ChartProvider for NoSparkProvider {
        async fn fetch_chart(
            &self,
            _: &str,
            _: crate::Interval,
            _: crate::TimeRange,
        ) -> Result<crate::models::chart::Chart> {
            Err(FinanceError::ApiError(
                "not exercised by these tests".into(),
            ))
        }
    }

    #[async_trait::async_trait]
    impl ProviderAdapter for NoSparkProvider {
        fn as_chart(&self) -> Option<&dyn ChartProvider> {
            Some(self)
        }
    }

    #[test]
    fn capabilities_derive_from_accessors() {
        let caps = ProviderAdapter::capabilities(&NoSparkProvider);
        assert_eq!(caps, Capability::CHART);
        assert!(!caps.contains(Capability::QUOTE));
    }

    #[tokio::test]
    async fn fetch_spark_defaults_to_not_supported() {
        let err = NoSparkProvider
            .fetch_spark(
                &["AAPL"],
                crate::Interval::OneDay,
                crate::TimeRange::FiveDays,
            )
            .await
            .unwrap_err();
        assert!(matches!(
            err,
            FinanceError::NotSupported {
                operation: Operation::Spark,
                ..
            }
        ));
    }

    #[tokio::test]
    async fn spark_routes_through_provider_set() {
        // The CHART default route resolves to the "yahoo"-id provider; routing a
        // provider that lacks spark must surface an error rather than silently
        // hitting a hardcoded Yahoo client.
        let set = ProviderSet::new(
            vec![Arc::new(NoSparkProvider)],
            None,
            Routes::new(Fetch::Sequential),
        );
        let result = set
            .fetch(Capability::CHART, |p| {
                let p = p.clone();
                async move {
                    p.as_chart()
                        .ok_or_else(|| p.not_supported(Operation::Spark))?
                        .fetch_spark(
                            &["AAPL"],
                            crate::Interval::OneDay,
                            crate::TimeRange::FiveDays,
                        )
                        .await
                }
            })
            .await;
        assert!(result.is_err());
    }

    // `Provider::capabilities()` derives from the adapters' accessor overrides
    // for every unit-struct provider, so those can't drift by construction.
    // Yahoo is the lone hand-declared set (constructing `YahooProvider` needs a
    // live auth handshake); this pins the const to the accessor overrides in
    // `yahoo.rs` — update both together.
    #[test]
    fn yahoo_caps_const_matches_declared_capabilities() {
        let expected = Capability::QUOTE
            .union(Capability::CHART)
            .union(Capability::FUNDAMENTALS)
            .union(Capability::CORPORATE)
            .union(Capability::OPTIONS)
            .union(Capability::MARKET)
            .union(Capability::INDICES)
            .union(Capability::COMMODITIES)
            .union(Capability::DISCOVERY)
            .union(Capability::CALENDAR);
        assert_eq!(yahoo::CAPS, expected);
    }

    /// Polygon gained CALENDAR (holidays) and AV gained MARKET (movers) via
    /// accessor overrides; `Provider::capabilities()` must reflect that.
    #[cfg(feature = "polygon")]
    #[test]
    fn polygon_derives_calendar_capability() {
        assert!(
            Provider::Polygon
                .capabilities()
                .contains(Capability::CALENDAR)
        );
    }

    #[cfg(feature = "alphavantage")]
    #[test]
    fn alphavantage_derives_market_capability() {
        assert!(
            Provider::AlphaVantage
                .capabilities()
                .contains(Capability::MARKET)
        );
    }

    #[test]
    fn capability_bits_are_distinct_single_bits() {
        for (i, (a, name_a)) in Capability::ALL.iter().enumerate() {
            assert_eq!(a.0.count_ones(), 1, "{name_a} is not a single bit");
            for (b, name_b) in &Capability::ALL[i + 1..] {
                assert_ne!(a.0, b.0, "{name_a} and {name_b} share a bit");
            }
        }
    }

    #[test]
    fn display_spells_out_combined_capabilities() {
        assert_eq!(Capability::QUOTE.to_string(), "quote");
        assert_eq!(
            (Capability::QUOTE | Capability::CHART).to_string(),
            "quote|chart"
        );
        assert_eq!(
            (Capability::FILINGS | Capability::CORPORATE).to_string(),
            "corporate|filings"
        );
        assert_eq!(Capability::NONE.to_string(), "unknown");
        // name() keeps its documented single-bit contract.
        assert_eq!((Capability::QUOTE | Capability::CHART).name(), "unknown");
    }

    #[tokio::test]
    async fn all_candidates_unsupported_surfaces_precise_operation() {
        // NoSparkProvider supports CHART but not spark: the final error must
        // name the spark operation, not collapse to NoProviderAvailable(CHART).
        let set = ProviderSet::new(
            vec![Arc::new(NoSparkProvider)],
            None,
            Routes::new(Fetch::Sequential),
        );
        let err = set
            .fetch(Capability::CHART, |p| {
                let p = p.clone();
                async move {
                    p.as_chart()
                        .ok_or_else(|| p.not_supported(Operation::Spark))?
                        .fetch_spark(
                            &["AAPL"],
                            crate::Interval::OneDay,
                            crate::TimeRange::FiveDays,
                        )
                        .await
                }
            })
            .await
            .unwrap_err();
        assert!(matches!(
            err,
            FinanceError::NotSupported {
                operation: Operation::Spark,
                ..
            }
        ));
    }

    #[tokio::test]
    async fn real_errors_outrank_not_supported_in_final_error() {
        // Two candidates: one lacking the op (NotSupported), one failing for
        // real. The real failure is the actionable error and must win.
        struct FailingChartProvider;
        impl ProviderCore for FailingChartProvider {
            fn id(&self) -> Provider {
                Provider::Edgar
            }
        }
        #[async_trait::async_trait]
        impl ChartProvider for FailingChartProvider {
            async fn fetch_chart(
                &self,
                _: &str,
                _: crate::Interval,
                _: crate::TimeRange,
            ) -> Result<crate::models::chart::Chart> {
                Err(FinanceError::ApiError("upstream 500".into()))
            }
            async fn fetch_spark(
                &self,
                _: &[&str],
                _: crate::Interval,
                _: crate::TimeRange,
            ) -> Result<Vec<(String, crate::models::chart::spark::Spark)>> {
                Err(FinanceError::ApiError("upstream 500".into()))
            }
        }
        #[async_trait::async_trait]
        impl ProviderAdapter for FailingChartProvider {
            fn as_chart(&self) -> Option<&dyn ChartProvider> {
                Some(self)
            }
        }

        let mut routes = Routes::new(Fetch::Sequential);
        routes
            .map
            .insert(Capability::CHART, vec![Provider::Yahoo, Provider::Edgar]);
        let set = ProviderSet::new(
            vec![Arc::new(NoSparkProvider), Arc::new(FailingChartProvider)],
            None,
            routes,
        );
        let err = set
            .fetch(Capability::CHART, |p| {
                let p = p.clone();
                async move {
                    p.as_chart()
                        .ok_or_else(|| p.not_supported(Operation::Spark))?
                        .fetch_spark(
                            &["AAPL"],
                            crate::Interval::OneDay,
                            crate::TimeRange::FiveDays,
                        )
                        .await
                }
            })
            .await
            .unwrap_err();
        assert!(matches!(err, FinanceError::ApiError(_)), "got {err:?}");
    }

    #[test]
    fn edgar_is_injected_unless_configured_explicitly() {
        assert!(needs_edgar_injection(&[Provider::Yahoo]));
        assert!(!needs_edgar_injection(&[Provider::Yahoo, Provider::Edgar]));
    }

    /// Alpha Vantage advertises FILINGS for insider trades only, so a
    /// capability-based check would drop EDGAR and break `filings()`.
    #[test]
    #[cfg(feature = "alphavantage")]
    fn a_filings_advertising_provider_does_not_suppress_edgar() {
        assert!(
            ProviderAdapter::capabilities(&alphavantage::AlphaVantageProvider)
                .contains(Capability::FILINGS)
        );
        assert!(needs_edgar_injection(&[Provider::AlphaVantage]));
    }

    /// A [`ChartProvider`] that returns `RateLimited` for its first
    /// `fail_times` calls, then succeeds — for exercising [`RetryPolicy`].
    struct FlakyChartProvider {
        id: Provider,
        fail_times: usize,
        calls: std::sync::atomic::AtomicUsize,
    }

    impl FlakyChartProvider {
        fn new(id: Provider, fail_times: usize) -> Self {
            Self {
                id,
                fail_times,
                calls: std::sync::atomic::AtomicUsize::new(0),
            }
        }

        fn call_count(&self) -> usize {
            self.calls.load(std::sync::atomic::Ordering::SeqCst)
        }
    }

    impl ProviderCore for FlakyChartProvider {
        fn id(&self) -> Provider {
            self.id
        }
    }

    #[async_trait::async_trait]
    impl ChartProvider for FlakyChartProvider {
        async fn fetch_chart(
            &self,
            symbol: &str,
            _: crate::Interval,
            _: crate::TimeRange,
        ) -> Result<crate::models::chart::Chart> {
            let n = self.calls.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
            if n < self.fail_times {
                return Err(FinanceError::RateLimited {
                    retry_after: Some(0),
                });
            }
            Ok(crate::models::chart::Chart {
                symbol: symbol.to_string(),
                meta: Default::default(),
                candles: Vec::new(),
                interval: None,
                range: None,
                provider_id: Some(self.id),
            })
        }
    }

    #[async_trait::async_trait]
    impl ProviderAdapter for FlakyChartProvider {
        fn as_chart(&self) -> Option<&dyn ChartProvider> {
            Some(self)
        }
    }

    async fn fetch_chart_via(set: &ProviderSet) -> Result<crate::models::chart::Chart> {
        set.fetch(Capability::CHART, |p| {
            let p = p.clone();
            async move {
                p.as_chart()
                    .ok_or_else(|| p.not_supported(Operation::Chart))?
                    .fetch_chart("AAPL", crate::Interval::OneDay, crate::TimeRange::FiveDays)
                    .await
            }
        })
        .await
    }

    #[tokio::test]
    async fn no_retry_policy_means_a_single_attempt_per_candidate() {
        let provider = Arc::new(FlakyChartProvider::new(Provider::Yahoo, usize::MAX));
        let set = ProviderSet::new(
            vec![provider.clone() as Arc<dyn ProviderAdapter>],
            None,
            Routes::new(Fetch::Sequential),
        );
        let err = fetch_chart_via(&set).await.unwrap_err();
        assert!(matches!(err, FinanceError::RateLimited { .. }));
        assert_eq!(
            provider.call_count(),
            1,
            "no retry policy set: exactly one attempt"
        );
    }

    #[tokio::test]
    async fn retry_policy_retries_rate_limited_then_succeeds() {
        let provider = Arc::new(FlakyChartProvider::new(Provider::Yahoo, 2));
        let set = ProviderSet::new(
            vec![provider.clone() as Arc<dyn ProviderAdapter>],
            None,
            Routes::new(Fetch::Sequential),
        )
        .with_retry_policy(Some(RetryPolicy::new(5)));

        let chart = fetch_chart_via(&set)
            .await
            .expect("should eventually succeed");
        assert_eq!(chart.symbol, "AAPL");
        // 2 failures + 1 success = 3 calls.
        assert_eq!(provider.call_count(), 3);
    }

    #[tokio::test]
    async fn retry_policy_exhausts_and_falls_through_to_next_provider() {
        let always_limited = Arc::new(FlakyChartProvider::new(Provider::Yahoo, usize::MAX));
        let succeeds = Arc::new(FlakyChartProvider::new(Provider::Edgar, 0));
        let mut routes = Routes::new(Fetch::Sequential);
        routes
            .map
            .insert(Capability::CHART, vec![Provider::Yahoo, Provider::Edgar]);
        let set = ProviderSet::new(
            vec![
                always_limited.clone() as Arc<dyn ProviderAdapter>,
                succeeds.clone() as Arc<dyn ProviderAdapter>,
            ],
            None,
            routes,
        )
        .with_retry_policy(Some(RetryPolicy::new(3)));

        let chart = fetch_chart_via(&set).await.expect("falls through to Edgar");
        assert_eq!(chart.provider_id, Some(Provider::Edgar));
        // Exhausts all 3 attempts on the first candidate before falling through.
        assert_eq!(always_limited.call_count(), 3);
        assert_eq!(succeeds.call_count(), 1);
    }

    #[tokio::test]
    async fn health_reflects_failures_and_recovers_after_success() {
        let provider = Arc::new(FlakyChartProvider::new(Provider::Yahoo, 1));
        let set = ProviderSet::new(
            vec![provider.clone() as Arc<dyn ProviderAdapter>],
            None,
            Routes::new(Fetch::Sequential),
        );

        // First call fails (RateLimited), recorded as a failure.
        assert!(fetch_chart_via(&set).await.is_err());
        let health = set.health();
        assert_eq!(health.len(), 1);
        assert_eq!(health[0].provider, Provider::Yahoo);
        assert_eq!(health[0].recent_failures, 1);
        assert!(health[0].last_error.is_some());

        // Second call succeeds, clearing last_error.
        assert!(fetch_chart_via(&set).await.is_ok());
        let health = set.health();
        assert_eq!(health[0].recent_successes, 1);
        assert!(health[0].last_error.is_none());
    }

    /// A provider that only implements `as_quote`, so a CHART dispatch hits
    /// its `NotSupported` path — which must not count against its health.
    struct QuoteOnlyProvider;

    impl ProviderCore for QuoteOnlyProvider {
        fn id(&self) -> Provider {
            Provider::Yahoo
        }
    }

    #[async_trait::async_trait]
    impl ProviderAdapter for QuoteOnlyProvider {}

    #[tokio::test]
    async fn not_supported_is_excluded_from_health_accounting() {
        let set = ProviderSet::new(
            vec![Arc::new(QuoteOnlyProvider) as Arc<dyn ProviderAdapter>],
            None,
            Routes::new(Fetch::Sequential),
        );
        // Goes through the real `record_health`, not a copy of its match.
        assert!(fetch_chart_via(&set).await.is_err());

        let health = set.health();
        assert!(health[0].is_healthy);
        assert_eq!(health[0].recent_successes, 0);
        assert_eq!(health[0].recent_failures, 0);
        assert!(health[0].last_error.is_none());
    }
}
