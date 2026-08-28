//! The [`Operation`] enum: one variant per adapter method.

use super::{Capability, Provider};
use crate::error::FinanceError;

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
