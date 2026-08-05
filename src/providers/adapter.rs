//! Provider adapter traits — one trait per [`Capability`].
//!
//! A provider bridge implements [`ProviderCore`] plus the capability traits it
//! actually serves, then overrides the matching `as_*` accessors on
//! [`ProviderAdapter`] to return `Some(self)`. `ProviderAdapter::capabilities`
//! is *derived* from those accessors, so a provider's declared capability set
//! can never drift from what it implements.
//!
//! Within a capability trait, the primary operation is required (implementing
//! the trait without it is unrepresentable) while secondary operations default
//! to [`FinanceError::NotSupported`]; dispatch ([`super::ProviderSet::fetch`])
//! skips `NotSupported` and falls through to the next routed provider, so
//! ragged per-operation coverage degrades gracefully.

use super::{Capability, Operation, Provider};
use crate::error::{FinanceError, Result};
use crate::models::quote::QuoteSummaryResponse;

/// Identity shared by every capability trait: the provider id and the
/// `NotSupported` error constructor used by default method bodies.
pub(crate) trait ProviderCore: Send + Sync {
    fn id(&self) -> Provider;

    fn not_supported(&self, operation: Operation) -> FinanceError {
        operation.not_supported(self.id())
    }
}

// ── Capability traits (always compiled) ─────────────────────────────

/// [`Capability::QUOTE`] — single and batch equity quotes.
#[async_trait::async_trait]
pub(crate) trait QuoteProvider: ProviderCore {
    async fn fetch_quote(&self, symbol: &str) -> Result<QuoteSummaryResponse>;

    /// Fetch quotes for multiple symbols in a single request.
    /// Returns `(symbol, QuoteSummaryResponse)` pairs — only partially populated
    /// (price module only) since batch endpoints don't return full quoteSummary data.
    async fn fetch_quotes_batch(&self, _: &[&str]) -> Result<Vec<(String, QuoteSummaryResponse)>> {
        Err(self.not_supported(Operation::QuotesBatch))
    }

    /// Fetch a snapshot for symbols spanning several asset classes in one
    /// request. Rows the provider could not resolve are returned with
    /// `error` set rather than dropped.
    #[cfg(feature = "polygon")]
    async fn fetch_unified_snapshot(
        &self,
        _symbols: &[&str],
    ) -> Result<Vec<crate::models::quote::snapshot::MarketSnapshot>> {
        Err(self.not_supported(Operation::UnifiedSnapshot))
    }
}

/// [`Capability::CHART`] — historical OHLCV candles and sparklines.
#[async_trait::async_trait]
pub(crate) trait ChartProvider: ProviderCore {
    async fn fetch_chart(
        &self,
        symbol: &str,
        interval: crate::Interval,
        range: crate::TimeRange,
    ) -> Result<crate::models::chart::Chart>;

    async fn fetch_chart_range(
        &self,
        _symbol: &str,
        _interval: crate::Interval,
        _start: i64,
        _end: i64,
    ) -> Result<crate::models::chart::Chart> {
        Err(self.not_supported(Operation::ChartRange))
    }

    /// Fetch lightweight sparkline data for multiple symbols in a single request.
    /// Returns successfully-parsed `(symbol, Spark)` pairs; callers fill in
    /// missing-symbol errors for any symbol absent from the result.
    async fn fetch_spark(
        &self,
        _symbols: &[&str],
        _interval: crate::Interval,
        _range: crate::TimeRange,
    ) -> Result<Vec<(String, crate::models::chart::spark::Spark)>> {
        Err(self.not_supported(Operation::Spark))
    }
}

/// [`Capability::FUNDAMENTALS`] — financial statements and share-supply data.
#[async_trait::async_trait]
pub(crate) trait FundamentalsProvider: ProviderCore {
    async fn fetch_financials(
        &self,
        symbol: &str,
        stmt_type: crate::StatementType,
        frequency: crate::Frequency,
    ) -> Result<crate::models::fundamentals::FinancialStatement>;

    /// Fetch bi-monthly short-interest settlement reports.
    async fn fetch_short_interest(
        &self,
        _symbol: &str,
    ) -> Result<Vec<crate::models::fundamentals::ShortInterest>> {
        Err(self.not_supported(Operation::ShortInterest))
    }

    /// Fetch daily short-volume data.
    async fn fetch_short_volume(
        &self,
        _symbol: &str,
    ) -> Result<Vec<crate::models::fundamentals::ShortVolume>> {
        Err(self.not_supported(Operation::ShortVolume))
    }

    /// Fetch share float and shares outstanding.
    async fn fetch_share_float(
        &self,
        _symbol: &str,
    ) -> Result<crate::models::fundamentals::ShareFloat> {
        Err(self.not_supported(Operation::ShareFloat))
    }

    /// Fetch the aggregated analyst price-target consensus.
    async fn fetch_price_target_consensus(
        &self,
        _symbol: &str,
    ) -> Result<crate::models::fundamentals::PriceTargetConsensus> {
        Err(self.not_supported(Operation::PriceTargetConsensus))
    }

    /// Fetch price-target publication activity over trailing windows.
    async fn fetch_price_target_summary(
        &self,
        _symbol: &str,
    ) -> Result<crate::models::fundamentals::PriceTargetSummary> {
        Err(self.not_supported(Operation::PriceTargetSummary))
    }

    /// Fetch the aggregated analyst rating consensus.
    async fn fetch_rating_consensus(
        &self,
        _symbol: &str,
    ) -> Result<crate::models::fundamentals::RatingConsensus> {
        Err(self.not_supported(Operation::RatingConsensus))
    }

    /// Fetch the trailing-twelve-month key-metrics snapshot.
    async fn fetch_key_metrics_ttm(
        &self,
        _symbol: &str,
    ) -> Result<crate::models::fundamentals::KeyMetricsTtm> {
        Err(self.not_supported(Operation::KeyMetricsTtm))
    }

    /// Fetch the trailing-twelve-month ratios snapshot.
    async fn fetch_ratios_ttm(
        &self,
        _symbol: &str,
    ) -> Result<crate::models::fundamentals::FinancialRatiosTtm> {
        Err(self.not_supported(Operation::RatiosTtm))
    }

    /// Fetch an ETF's profile and portfolio holdings.
    async fn fetch_etf_profile(
        &self,
        _symbol: &str,
    ) -> Result<crate::models::fundamentals::EtfProfile> {
        Err(self.not_supported(Operation::EtfProfile))
    }
}

/// [`Capability::CORPORATE`] — news, corporate events, similar-symbol
/// recommendations.
#[async_trait::async_trait]
pub(crate) trait CorporateProvider: ProviderCore {
    async fn fetch_news(&self, symbol: &str) -> Result<Vec<crate::models::corporate::news::News>>;

    async fn fetch_events(&self, symbol: &str)
    -> Result<crate::models::chart::events::ChartEvents>;

    async fn fetch_similar_symbols(
        &self,
        _symbol: &str,
        _limit: u32,
    ) -> Result<Vec<crate::models::corporate::recommendation::SimilarSymbol>> {
        Err(self.not_supported(Operation::Recommendations))
    }

    /// Fetch the company's own press releases.
    async fn fetch_press_releases(
        &self,
        _symbol: &str,
        _limit: u32,
    ) -> Result<Vec<crate::models::corporate::press_release::PressRelease>> {
        Err(self.not_supported(Operation::PressReleases))
    }

    /// Fetch reported executive compensation, most recent fiscal year first.
    async fn fetch_executive_compensation(
        &self,
        _symbol: &str,
    ) -> Result<Vec<crate::models::corporate::governance::ExecutiveCompensation>> {
        Err(self.not_supported(Operation::ExecutiveCompensation))
    }

    /// Fetch reported employee headcount history, most recent period first.
    async fn fetch_employee_count(
        &self,
        _symbol: &str,
    ) -> Result<Vec<crate::models::corporate::governance::EmployeeCount>> {
        Err(self.not_supported(Operation::EmployeeCount))
    }
}

/// [`Capability::OPTIONS`] — options chains.
#[async_trait::async_trait]
pub(crate) trait OptionsProvider: ProviderCore {
    async fn fetch_options(
        &self,
        symbol: &str,
        date: Option<i64>,
    ) -> Result<crate::models::options::Options>;
}

/// [`Capability::FILINGS`] — SEC filing data.
#[async_trait::async_trait]
pub(crate) trait FilingsProvider: ProviderCore {
    async fn fetch_filings(&self, symbol: &str) -> Result<crate::models::filings::ProviderFilings>;

    /// Fetch the sectioned text of one filing by accession number.
    async fn fetch_filing_sections(
        &self,
        _accession_number: &str,
        _form: crate::models::filings::FilingSectionForm,
    ) -> Result<Vec<crate::models::filings::FilingSection>> {
        Err(self.not_supported(Operation::FilingSections))
    }

    /// Fetch risk factors extracted from a symbol's SEC filings.
    async fn fetch_risk_factors(
        &self,
        _symbol: &str,
    ) -> Result<Vec<crate::models::filings::RiskFactor>> {
        Err(self.not_supported(Operation::RiskFactors))
    }

    /// Search filing *text* rather than looking filings up by filer.
    ///
    /// `symbol` scopes the search to one filer; the provider resolves it to
    /// whatever identifier its own index is keyed by (a CIK, for EDGAR).
    /// `None` searches every filer.
    async fn fetch_filing_search(
        &self,
        _symbol: Option<&str>,
        _query: &str,
        _filters: &crate::models::filings::FilingSearchFilters,
    ) -> Result<Vec<crate::models::filings::FilingSearchHit>> {
        Err(self.not_supported(Operation::FilingSearch))
    }

    /// Fetch insider transactions reported on Forms 3/4/5, newest filing first.
    async fn fetch_insider_trades(
        &self,
        _symbol: &str,
        _limit: u32,
    ) -> Result<Vec<crate::models::filings::InsiderTrade>> {
        Err(self.not_supported(Operation::InsiderTrades))
    }

    /// Fetch the latest reported 13F institutional holdings for a filer.
    async fn fetch_institutional_holdings(
        &self,
        _symbol: &str,
    ) -> Result<Vec<crate::models::filings::InstitutionalHolding>> {
        Err(self.not_supported(Operation::InstitutionalHoldings))
    }
}

// ── Capability traits (feature-gated) ───────────────────────────────

/// [`Capability::DISCOVERY`] — symbol search, reference data, exchanges,
/// screeners.
#[cfg(any(feature = "fmp", feature = "polygon", feature = "alphavantage"))]
#[async_trait::async_trait]
pub(crate) trait DiscoveryProvider: ProviderCore {
    /// Search the provider's symbol universe by free-text query.
    async fn fetch_symbol_search(
        &self,
        query: &str,
        limit: u32,
    ) -> Result<Vec<crate::models::discovery::reference::SymbolMatch>>;

    /// Fetch detailed reference data for a single symbol.
    async fn fetch_symbol_details(
        &self,
        _symbol: &str,
    ) -> Result<crate::models::discovery::reference::SymbolDetails> {
        Err(self.not_supported(Operation::SymbolDetails))
    }

    /// Fetch the provider's tradable exchange listing.
    async fn fetch_exchanges(
        &self,
    ) -> Result<Vec<crate::models::discovery::reference::ExchangeInfo>> {
        Err(self.not_supported(Operation::Exchanges))
    }

    /// Run a screener query over the provider's universe.
    async fn fetch_screener(
        &self,
        _filters: &crate::models::discovery::reference::ScreenerFilters,
    ) -> Result<Vec<crate::models::discovery::reference::ScreenerMatch>> {
        Err(self.not_supported(Operation::Screener))
    }

    /// Fetch the provider's whole listed-security universe.
    ///
    /// `active = false` asks for delisted securities instead. Unlike
    /// [`fetch_symbol_search`](Self::fetch_symbol_search) this is an unfiltered
    /// dump, so expect thousands of rows in one response.
    async fn fetch_listing_status(
        &self,
        _active: bool,
    ) -> Result<Vec<crate::models::discovery::reference::SymbolMatch>> {
        Err(self.not_supported(Operation::ListingStatus))
    }
}

/// [`Capability::CALENDAR`] — market-wide calendars.
#[cfg(any(feature = "fmp", feature = "polygon", feature = "alphavantage"))]
#[async_trait::async_trait]
pub(crate) trait CalendarProvider: ProviderCore {
    /// Fetch a market-wide calendar over `[from, to]` (`YYYY-MM-DD` dates).
    ///
    /// One method rather than one per kind — providers serve all kinds from the
    /// same calendar family, and `kind.operation()` still reports the precise
    /// [`Operation`] in `NotSupported` errors.
    async fn fetch_market_calendar(
        &self,
        kind: crate::models::calendar::market::CalendarKind,
        from: &str,
        to: &str,
    ) -> Result<Vec<crate::models::calendar::market::MarketCalendarEntry>>;
}

/// [`Capability::MARKET`] — sector/industry performance and movers.
///
/// Movers is the required primary (every current implementor serves it);
/// the sector/industry statistics default to `NotSupported` since coverage
/// is ragged (FMP serves all of them; Yahoo and Alpha Vantage only movers).
#[async_trait::async_trait]
pub(crate) trait MarketProvider: ProviderCore {
    /// Fetch the market movers list for `direction`.
    async fn fetch_market_movers(
        &self,
        direction: crate::models::market::performance::MoverDirection,
    ) -> Result<Vec<crate::models::market::performance::MoverQuote>>;

    /// Fetch aggregate performance for every sector.
    async fn fetch_sector_performance(
        &self,
    ) -> Result<Vec<crate::models::market::performance::SectorPerformance>> {
        Err(self.not_supported(Operation::SectorPerformance))
    }

    /// Fetch historical aggregate sector performance, most recent first.
    async fn fetch_sector_performance_history(
        &self,
        _limit: u32,
    ) -> Result<Vec<crate::models::market::performance::SectorPerformanceHistory>> {
        Err(self.not_supported(Operation::SectorPerformanceHistory))
    }

    /// Fetch sector price/earnings ratios.
    async fn fetch_sector_pe(&self) -> Result<Vec<crate::models::market::performance::SectorPe>> {
        Err(self.not_supported(Operation::SectorPerformance))
    }

    /// Fetch industry price/earnings ratios.
    async fn fetch_industry_pe(
        &self,
    ) -> Result<Vec<crate::models::market::performance::IndustryPe>> {
        Err(self.not_supported(Operation::SectorPerformance))
    }
}

/// [`Capability::CRYPTO`] — cryptocurrency quotes.
#[cfg(any(
    feature = "alphavantage",
    feature = "binance",
    feature = "crypto",
    feature = "defi",
    feature = "fmp",
    feature = "kraken",
    feature = "polygon"
))]
#[async_trait::async_trait]
pub(crate) trait CryptoProvider: ProviderCore {
    async fn fetch_crypto_quote(
        &self,
        id: &str,
        vs_currency: &str,
    ) -> Result<crate::models::crypto::CryptoQuote>;

    /// Fetch total value locked in a DeFi protocol.
    #[cfg(feature = "defi")]
    async fn fetch_protocol_tvl(
        &self,
        _protocol: &str,
    ) -> Result<crate::models::crypto::defi::ProtocolTvl> {
        Err(self.not_supported(Operation::ProtocolTvl))
    }

    /// Fetch a DeFi protocol's TVL history, oldest first.
    #[cfg(feature = "defi")]
    async fn fetch_protocol_tvl_history(
        &self,
        _protocol: &str,
    ) -> Result<Vec<crate::models::crypto::defi::TvlPoint>> {
        Err(self.not_supported(Operation::ProtocolTvlHistory))
    }
}

/// [`Capability::ECONOMIC`] — macro-economic data series.
#[cfg(any(
    feature = "alphavantage",
    feature = "bls",
    feature = "fiscaldata",
    feature = "fred",
    feature = "polygon",
    feature = "worldbank"
))]
#[async_trait::async_trait]
pub(crate) trait EconomicProvider: ProviderCore {
    async fn fetch_economic_series(
        &self,
        series_id: &str,
    ) -> Result<crate::models::economic::EconomicSeries>;

    /// Fetch a series as it stood on `date` (`YYYY-MM-DD`) rather than as
    /// currently revised — the point-in-time view backtests need.
    async fn fetch_economic_series_as_of(
        &self,
        _series_id: &str,
        _date: &str,
    ) -> Result<crate::models::economic::EconomicSeries> {
        Err(self.not_supported(Operation::EconomicSeriesAsOf))
    }

    /// Search the provider's series catalog by free text.
    async fn fetch_economic_search(
        &self,
        _query: &str,
        _limit: u32,
    ) -> Result<Vec<crate::models::economic::EconomicSeriesMatch>> {
        Err(self.not_supported(Operation::EconomicSearch))
    }

    /// List the child categories of `parent_id` in the series category tree.
    async fn fetch_economic_categories(
        &self,
        _parent_id: i64,
    ) -> Result<Vec<crate::models::economic::EconomicCategory>> {
        Err(self.not_supported(Operation::EconomicCategories))
    }

    /// List the provider's scheduled data releases.
    async fn fetch_economic_releases(
        &self,
    ) -> Result<Vec<crate::models::economic::EconomicRelease>> {
        Err(self.not_supported(Operation::EconomicReleases))
    }
}

/// [`Capability::FOREX`] — currency-pair quotes.
#[cfg(any(
    feature = "alphavantage",
    feature = "fmp",
    feature = "frankfurter",
    feature = "polygon"
))]
#[async_trait::async_trait]
pub(crate) trait ForexProvider: ProviderCore {
    async fn fetch_forex_quote(
        &self,
        from: &str,
        to: &str,
    ) -> Result<crate::models::forex::ForexQuote>;
}

/// [`Capability::INDICES`] — stock market index quotes.
#[cfg(any(feature = "polygon", feature = "fmp"))]
#[async_trait::async_trait]
pub(crate) trait IndicesProvider: ProviderCore {
    async fn fetch_indices_quote(&self, symbol: &str)
    -> Result<crate::models::indices::IndexQuote>;

    /// Fetch the current constituents of a major index.
    async fn fetch_index_constituents(
        &self,
        _index: crate::models::indices::MajorIndex,
    ) -> Result<Vec<crate::models::indices::IndexConstituent>> {
        Err(self.not_supported(Operation::IndexConstituents))
    }

    /// Fetch historical constituent changes of a major index.
    async fn fetch_index_constituent_changes(
        &self,
        _index: crate::models::indices::MajorIndex,
    ) -> Result<Vec<crate::models::indices::IndexConstituentChange>> {
        Err(self.not_supported(Operation::IndexConstituentChanges))
    }
}

/// [`Capability::FUTURES`] — futures contract quotes.
#[cfg(any(feature = "polygon", feature = "cftc"))]
#[async_trait::async_trait]
pub(crate) trait FuturesProvider: ProviderCore {
    async fn fetch_futures_quote(
        &self,
        symbol: &str,
    ) -> Result<crate::models::futures::FuturesQuote>;

    /// Fetch weekly CFTC Commitments of Traders positioning for a futures
    /// symbol, broken down by trader category.
    #[cfg(feature = "cftc")]
    async fn fetch_commitments_of_traders(
        &self,
        _symbol: &str,
    ) -> Result<crate::models::futures::cot::CommitmentsOfTraders> {
        Err(self.not_supported(Operation::CommitmentsOfTraders))
    }
}

/// [`Capability::COMMODITIES`] — commodity price quotes.
#[cfg(any(feature = "fmp", feature = "alphavantage"))]
#[async_trait::async_trait]
pub(crate) trait CommoditiesProvider: ProviderCore {
    async fn fetch_commodities_quote(
        &self,
        symbol: &str,
    ) -> Result<crate::models::commodities::CommodityQuote>;
}

// ── The adapter trait ───────────────────────────────────────────────

/// A configured provider as seen by [`super::ProviderSet`] dispatch: lifecycle
/// plus one `as_*` accessor per capability. Override an accessor to
/// `Some(self)` for each capability trait the provider implements.
#[async_trait::async_trait]
pub(crate) trait ProviderAdapter: ProviderCore {
    /// Initialize this provider. Called once during construction.
    async fn initialize(&self) -> Result<()> {
        Ok(())
    }

    fn as_quote(&self) -> Option<&dyn QuoteProvider> {
        None
    }
    fn as_chart(&self) -> Option<&dyn ChartProvider> {
        None
    }
    fn as_fundamentals(&self) -> Option<&dyn FundamentalsProvider> {
        None
    }
    fn as_corporate(&self) -> Option<&dyn CorporateProvider> {
        None
    }
    fn as_options(&self) -> Option<&dyn OptionsProvider> {
        None
    }
    fn as_filings(&self) -> Option<&dyn FilingsProvider> {
        None
    }
    #[cfg(any(feature = "fmp", feature = "polygon", feature = "alphavantage"))]
    fn as_discovery(&self) -> Option<&dyn DiscoveryProvider> {
        None
    }
    #[cfg(any(feature = "fmp", feature = "polygon", feature = "alphavantage"))]
    fn as_calendar(&self) -> Option<&dyn CalendarProvider> {
        None
    }
    fn as_market(&self) -> Option<&dyn MarketProvider> {
        None
    }
    #[cfg(any(
        feature = "alphavantage",
        feature = "binance",
        feature = "crypto",
        feature = "defi",
        feature = "fmp",
        feature = "kraken",
        feature = "polygon"
    ))]
    fn as_crypto(&self) -> Option<&dyn CryptoProvider> {
        None
    }
    #[cfg(any(
        feature = "alphavantage",
        feature = "bls",
        feature = "fiscaldata",
        feature = "fred",
        feature = "polygon",
        feature = "worldbank"
    ))]
    fn as_economic(&self) -> Option<&dyn EconomicProvider> {
        None
    }
    #[cfg(any(
        feature = "alphavantage",
        feature = "fmp",
        feature = "frankfurter",
        feature = "polygon"
    ))]
    fn as_forex(&self) -> Option<&dyn ForexProvider> {
        None
    }
    #[cfg(any(feature = "polygon", feature = "fmp"))]
    fn as_indices(&self) -> Option<&dyn IndicesProvider> {
        None
    }
    #[cfg(any(feature = "polygon", feature = "cftc"))]
    fn as_futures(&self) -> Option<&dyn FuturesProvider> {
        None
    }
    #[cfg(any(feature = "fmp", feature = "alphavantage"))]
    fn as_commodities(&self) -> Option<&dyn CommoditiesProvider> {
        None
    }

    /// Derived from the accessors — a provider supports a capability iff the
    /// matching `as_*` accessor returns `Some`. Never override.
    fn capabilities(&self) -> Capability {
        let mut caps = Capability::NONE;
        if self.as_quote().is_some() {
            caps = caps | Capability::QUOTE;
        }
        if self.as_chart().is_some() {
            caps = caps | Capability::CHART;
        }
        if self.as_fundamentals().is_some() {
            caps = caps | Capability::FUNDAMENTALS;
        }
        if self.as_corporate().is_some() {
            caps = caps | Capability::CORPORATE;
        }
        if self.as_options().is_some() {
            caps = caps | Capability::OPTIONS;
        }
        if self.as_filings().is_some() {
            caps = caps | Capability::FILINGS;
        }
        if self.as_market().is_some() {
            caps = caps | Capability::MARKET;
        }
        #[cfg(any(feature = "fmp", feature = "polygon", feature = "alphavantage"))]
        {
            if self.as_discovery().is_some() {
                caps = caps | Capability::DISCOVERY;
            }
            if self.as_calendar().is_some() {
                caps = caps | Capability::CALENDAR;
            }
        }
        #[cfg(any(
            feature = "alphavantage",
            feature = "binance",
            feature = "crypto",
            feature = "defi",
            feature = "fmp",
            feature = "kraken",
            feature = "polygon"
        ))]
        if self.as_crypto().is_some() {
            caps = caps | Capability::CRYPTO;
        }
        #[cfg(any(
            feature = "alphavantage",
            feature = "bls",
            feature = "fiscaldata",
            feature = "fred",
            feature = "polygon",
            feature = "worldbank"
        ))]
        if self.as_economic().is_some() {
            caps = caps | Capability::ECONOMIC;
        }
        #[cfg(any(
            feature = "alphavantage",
            feature = "fmp",
            feature = "frankfurter",
            feature = "polygon"
        ))]
        if self.as_forex().is_some() {
            caps = caps | Capability::FOREX;
        }
        #[cfg(any(feature = "polygon", feature = "fmp"))]
        if self.as_indices().is_some() {
            caps = caps | Capability::INDICES;
        }
        #[cfg(any(feature = "polygon", feature = "cftc"))]
        if self.as_futures().is_some() {
            caps = caps | Capability::FUTURES;
        }
        #[cfg(any(feature = "fmp", feature = "alphavantage"))]
        if self.as_commodities().is_some() {
            caps = caps | Capability::COMMODITIES;
        }
        caps
    }
}
