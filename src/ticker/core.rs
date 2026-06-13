//! Symbol-specific data access from multiple providers.

use crate::adapters::yahoo::client::{ClientConfig, YahooClient};
#[cfg(feature = "backtesting")]
use crate::backtesting;
use crate::constants::{Frequency, Interval, Region, StatementType, TimeRange};
use crate::edgar;
use crate::error::{FinanceError, Result};
use crate::format::Both;
#[cfg(any(feature = "backtesting", feature = "indicators"))]
use crate::indicators;
use crate::models::chart::events::ChartEvents;
use crate::models::chart::{CapitalGain, Chart, Dividend, DividendAnalytics, Split};
use crate::models::corporate::news::News;
use crate::models::corporate::recommendation::Recommendation;
use crate::models::filings::{CompanyFacts, EdgarSubmissions, ProviderFilings};
use crate::models::format::Format;
use crate::models::fundamentals::FinancialStatement;
use crate::models::options::Options;
use crate::models::quote::{
    AssetProfile, CalendarEvents, DefaultKeyStatistics, Earnings, EarningsHistory, EarningsTrend,
    EquityPerformance, FinancialData, FundOwnership, FundPerformance, FundProfile, IndexTrend,
    IndustryTrend, InsiderHolders, InsiderTransactions, InstitutionOwnership,
    MajorHoldersBreakdown, NetSharePurchaseActivity, Price, Quote, QuoteSummaryResponse,
    QuoteTypeData, RecommendationTrend, SecFilings, SectorTrend, SummaryDetail, SummaryProfile,
    TopHoldings, UpgradeDowngradeHistory,
};

use crate::providers::types::recommendation_from_similar;
use crate::providers::yahoo::YahooProvider;
use crate::providers::{
    Capability, Fetch, Provider, ProviderAdapter, ProviderSet, Routes, build_providers,
};
#[cfg(feature = "risk")]
use crate::risk;
use crate::utils::{CacheEntry, EVICTION_THRESHOLD, filter_by_range};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;

type Cache<T> = Arc<RwLock<Option<CacheEntry<T>>>>;
type MapCache<K, V> = Arc<RwLock<HashMap<K, CacheEntry<V>>>>;

/// Opaque handle to a shared Yahoo Finance client session.
///
/// Allows multiple [`Ticker`] and [`Tickers`](crate::Tickers) instances to share
/// one authenticated session, avoiding redundant auth handshakes.
///
/// Obtain via [`Ticker::client_handle`] or [`Tickers::client_handle`], then
/// pass to other builders via `.client(handle)`.
///
/// # Example
///
/// ```no_run
/// use finance_query::Ticker;
///
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let aapl = Ticker::new("AAPL").await?;
/// let handle = aapl.client_handle();
///
/// let msft = Ticker::builder("MSFT").client(handle.clone()).build().await?;
/// let googl = Ticker::builder("GOOGL").client(handle).build().await?;
/// # Ok(())
/// # }
/// ```
#[derive(Clone)]
pub struct ClientHandle(pub(crate) Arc<YahooClient>);
/// Builder for constructing a [`Ticker`] with optional configuration.
///
/// Construct via [`Ticker::builder`]. All builder methods are optional;
/// call [`build`](TickerBuilder::build) to finalize.
pub struct TickerBuilder {
    symbol: Arc<str>,
    config: ClientConfig,
    shared_client: Option<ClientHandle>,
    injected_providers: Option<Arc<ProviderSet>>,
    cache_ttl: Option<Duration>,
    include_logo: bool,
}

impl TickerBuilder {
    fn new(symbol: impl Into<String>) -> Self {
        Self {
            symbol: symbol.into().into(),
            config: ClientConfig::default(),
            shared_client: None,
            injected_providers: None,
            cache_ttl: None,
            include_logo: false,
        }
    }
    /// Set the region (automatically sets correct lang and region).
    pub fn region(mut self, region: Region) -> Self {
        self.config.lang = region.lang().to_string();
        self.config.region = region.region().to_string();
        self
    }
    /// Set the language code (e.g., "en-US", "ja-JP").
    pub fn lang(mut self, lang: impl Into<String>) -> Self {
        self.config.lang = lang.into();
        self
    }
    /// Set the region code (e.g., "US", "JP").
    pub fn region_code(mut self, r: impl Into<String>) -> Self {
        self.config.region = r.into();
        self
    }
    /// Set the HTTP request timeout.
    pub fn timeout(mut self, t: Duration) -> Self {
        self.config.timeout = t;
        self
    }
    /// Set the proxy URL.
    pub fn proxy(mut self, p: impl Into<String>) -> Self {
        self.config.proxy = Some(p.into());
        self
    }
    #[allow(dead_code)]
    pub(crate) fn config(mut self, c: ClientConfig) -> Self {
        self.config = c;
        self
    }
    /// Pre-inject a shared provider set (used by [`Providers::stock`](crate::Providers::stock)).
    pub(crate) fn with_provider_set(mut self, set: Arc<ProviderSet>) -> Self {
        self.injected_providers = Some(set);
        self
    }
    /// Share an existing authenticated session instead of creating a new one.
    ///
    /// Avoids redundant auth handshakes when creating multiple `Ticker` instances.
    /// Obtain a handle from any existing `Ticker` via [`Ticker::client_handle`].
    ///
    /// When set, the builder's `config`, `timeout`, `proxy`, `lang`, and `region`
    /// settings are ignored — the shared session's configuration is used instead.
    pub fn client(mut self, handle: ClientHandle) -> Self {
        self.shared_client = Some(handle);
        self
    }
    /// Enable response caching with a time-to-live.
    pub fn cache(mut self, ttl: Duration) -> Self {
        self.cache_ttl = Some(ttl);
        self
    }
    /// Include company logo URLs in quote responses.
    pub fn logo(mut self) -> Self {
        self.include_logo = true;
        self
    }

    /// Build the Ticker instance.
    pub async fn build(self) -> Result<Ticker> {
        #[cfg(feature = "translation")]
        let translate_lang = {
            let lang = crate::translation::Lang::parse(&self.config.lang)?;
            (!lang.is_english()).then_some(lang)
        };
        let providers = if let Some(set) = self.injected_providers {
            set
        } else if let Some(handle) = self.shared_client {
            let yahoo = YahooProvider::from_client(handle.0);
            let client = yahoo.client_arc();
            Arc::new(ProviderSet::new(
                vec![Arc::new(yahoo) as Arc<dyn ProviderAdapter>],
                Some(client),
                Routes::new(Fetch::Sequential),
            ))
        } else {
            Arc::new(
                build_providers(
                    &[Provider::Yahoo],
                    &self.config,
                    Routes::new(Fetch::Sequential),
                )
                .await?,
            )
        };
        Ok(Ticker {
            symbol: self.symbol,
            providers,
            cache_ttl: self.cache_ttl,
            include_logo: self.include_logo,
            #[cfg(feature = "translation")]
            translate_lang,
            quote_cache: Default::default(),
            quote_fetch: Arc::new(tokio::sync::Mutex::new(())),
            chart_cache: Default::default(),
            events_cache: Default::default(),
            news_cache: Default::default(),
            options_cache: Default::default(),
            financials_cache: Default::default(),
            #[cfg(feature = "indicators")]
            indicators_cache: Default::default(),
            edgar_submissions_cache: Default::default(),
            edgar_facts_cache: Default::default(),
        })
    }
}

/// The primary entry point for querying financial data for a single symbol.
///
/// Data is fetched on first access and cached. Use the builder pattern
/// via [`Ticker::builder`] for custom configuration.
pub struct Ticker {
    symbol: Arc<str>,
    providers: Arc<ProviderSet>,
    cache_ttl: Option<Duration>,
    include_logo: bool,
    #[cfg(feature = "translation")]
    translate_lang: Option<crate::translation::Lang>,
    quote_cache: Cache<QuoteSummaryResponse>,
    quote_fetch: Arc<tokio::sync::Mutex<()>>,
    chart_cache: MapCache<(Interval, TimeRange), Chart>,
    events_cache: Cache<ChartEvents>,
    news_cache: Cache<Vec<News>>,
    options_cache: MapCache<Option<i64>, Options>,
    financials_cache: MapCache<(StatementType, Frequency), FinancialStatement>,
    #[cfg(feature = "indicators")]
    indicators_cache: MapCache<(Interval, TimeRange), indicators::IndicatorsSummary>,
    edgar_submissions_cache: Cache<EdgarSubmissions>,
    edgar_facts_cache: Cache<CompanyFacts>,
}

impl Ticker {
    /// Creates a new ticker with default configuration.
    pub async fn new(symbol: impl Into<String>) -> Result<Self> {
        Self::builder(symbol).build().await
    }
    /// Creates a new builder for Ticker.
    pub fn builder(symbol: impl Into<String>) -> TickerBuilder {
        TickerBuilder::new(symbol)
    }
    /// Returns the ticker symbol.
    pub fn symbol(&self) -> &str {
        &self.symbol
    }

    /// Returns a handle to the underlying Yahoo Finance session.
    ///
    /// Pass to other builders via `.client(handle)` to share the authenticated
    /// session without a new auth handshake.
    ///
    /// # Panics
    ///
    /// Panics if this ticker was created via [`Providers`](crate::Providers) with
    /// no Yahoo provider configured. For session sharing across multiple tickers,
    /// prefer [`Providers::ticker`](crate::Providers::ticker) instead.
    pub fn client_handle(&self) -> ClientHandle {
        ClientHandle(
            self.providers
                .first_yahoo()
                .expect("client_handle requires a Yahoo session; use Providers::ticker() for multi-provider tickers"),
        )
    }

    #[allow(dead_code)]
    pub(crate) fn provider_set(&self) -> &Arc<ProviderSet> {
        &self.providers
    }

    /// Translate a response value when a non-English language is configured
    /// (no-op otherwise).
    #[cfg(feature = "translation")]
    pub(crate) async fn translate_response<T: crate::translation::Translatable>(
        &self,
        value: &mut T,
    ) -> Result<()> {
        if let Some(lang) = &self.translate_lang {
            crate::translation::translate_with(value, lang).await?;
        }
        Ok(())
    }

    fn is_cache_fresh<T>(&self, entry: Option<&CacheEntry<T>>) -> bool {
        CacheEntry::is_fresh_with_ttl(entry, self.cache_ttl)
    }

    /// Like `is_cache_fresh`, but works on the shared-cache pattern
    /// where the entry is populated on first fetch.
    /// When no TTL is configured, never treats entries as fresh.
    fn is_shared_cache_fresh<T>(&self, entry: Option<&CacheEntry<T>>) -> bool {
        match (self.cache_ttl, entry) {
            (Some(ttl), Some(e)) => e.is_fresh(ttl),
            _ => false,
        }
    }
    fn cache_insert<K: Eq + std::hash::Hash, V>(
        &self,
        map: &mut HashMap<K, CacheEntry<V>>,
        key: K,
        value: V,
    ) {
        if let Some(ttl) = self.cache_ttl {
            if map.len() >= EVICTION_THRESHOLD {
                map.retain(|_, entry| entry.is_fresh(ttl));
            }
            map.insert(key, CacheEntry::new(value));
        }
    }

    /// Get full quote data, optionally including logo URLs.
    pub async fn quote<F>(&self) -> Result<Quote<F>>
    where
        F: Format,
        Quote<Both>: Into<Quote<F>>,
    {
        let cache = self.ensure_quote().await?;
        let summary = cache.as_ref().ok_or_else(|| {
            FinanceError::ApiError("Quote summary cache was empty after fetch".to_string())
        })?;
        let (logo_url, company_logo_url) = if self.include_logo {
            if let Ok(yahoo) = self.providers.first_yahoo() {
                let logos = yahoo.get_logo_url(&self.symbol).await;
                (logos.0, logos.1)
            } else {
                (None, None)
            }
        } else {
            (None, None)
        };
        let quote = Quote::from_response(&summary.value, logo_url, company_logo_url);
        #[cfg(feature = "translation")]
        let quote = {
            drop(cache);
            let mut quote = quote;
            self.translate_response(&mut quote).await?;
            quote
        };
        Ok(quote.into())
    }

    fn chart_from_provider_data(
        mut data: Chart,
        interval: Option<Interval>,
        range: Option<TimeRange>,
    ) -> Chart {
        data.interval = interval;
        data.range = range;
        data
    }

    /// Get historical OHLCV chart data.
    pub async fn chart(&self, interval: Interval, range: TimeRange) -> Result<Chart> {
        {
            let cache = self.chart_cache.read().await;
            if let Some(entry) = cache.get(&(interval, range))
                && self.is_cache_fresh(Some(entry))
            {
                return Ok(entry.value.clone());
            }
        }
        let sym = self.symbol.clone();
        let data = self
            .providers
            .fetch(Capability::CHART, move |p| {
                let sym = sym.clone();
                let p = p.clone();
                async move { p.fetch_chart(&sym, interval, range).await }
            })
            .await?;
        let chart = Self::chart_from_provider_data(data, Some(interval), Some(range));
        if self.cache_ttl.is_some() {
            let mut cache = self.chart_cache.write().await;
            self.cache_insert(&mut cache, (interval, range), chart.clone());
        }
        Ok(chart)
    }

    /// Get chart data for a custom start/end timestamp range.
    pub async fn chart_range(&self, interval: Interval, start: i64, end: i64) -> Result<Chart> {
        if start >= end {
            return Err(FinanceError::InvalidParameter {
                param: "end".into(),
                reason: format!("end ({end}) must be > start ({start})"),
            });
        }
        let sym = self.symbol.clone();
        let data = self
            .providers
            .fetch(Capability::CHART, move |p| {
                let sym = sym.clone();
                let p = p.clone();
                async move { p.fetch_chart_range(&sym, interval, start, end).await }
            })
            .await?;
        Ok(Self::chart_from_provider_data(data, Some(interval), None))
    }

    async fn ensure_events(&self) -> Result<()> {
        {
            let cache = self.events_cache.read().await;
            if self.is_shared_cache_fresh(cache.as_ref()) {
                return Ok(());
            }
        }
        let sym = self.symbol.clone();
        let events = self
            .providers
            .fetch(Capability::CORPORATE, move |p| {
                let sym = sym.clone();
                let p = p.clone();
                async move { p.fetch_events(&sym).await }
            })
            .await?;
        let mut cache = self.events_cache.write().await;
        *cache = Some(CacheEntry::new(events));
        Ok(())
    }

    /// Get dividend history.
    pub async fn dividends(&self, range: TimeRange) -> Result<Vec<Dividend>> {
        self.ensure_events().await?;
        let cache = self.events_cache.read().await;
        let all = cache
            .as_ref()
            .map(|e| e.value.to_dividends())
            .unwrap_or_default();
        Ok(filter_by_range(all, range))
    }
    /// Compute dividend analytics for the requested time range.
    pub async fn dividend_analytics(&self, range: TimeRange) -> Result<DividendAnalytics> {
        let divs = self.dividends(range).await?;
        Ok(DividendAnalytics::from_dividends(&divs))
    }
    /// Get stock split history.
    pub async fn splits(&self, range: TimeRange) -> Result<Vec<Split>> {
        self.ensure_events().await?;
        let cache = self.events_cache.read().await;
        let all = cache
            .as_ref()
            .map(|e| e.value.to_splits())
            .unwrap_or_default();
        Ok(filter_by_range(all, range))
    }
    /// Get capital gains distribution history.
    pub async fn capital_gains(&self, range: TimeRange) -> Result<Vec<CapitalGain>> {
        self.ensure_events().await?;
        let cache = self.events_cache.read().await;
        let all = cache
            .as_ref()
            .map(|e| e.value.to_capital_gains())
            .unwrap_or_default();
        Ok(filter_by_range(all, range))
    }

    /// Get analyst recommendations and similar symbols.
    pub async fn recommendations(&self, limit: u32) -> Result<Recommendation> {
        if limit == 0 {
            return Err(FinanceError::InvalidParameter {
                param: "limit".into(),
                reason: "limit must be > 0".into(),
            });
        }
        let sym = self.symbol.clone();
        let (provider_id, items) = self
            .providers
            .fetch(Capability::CORPORATE, move |p| {
                let sym = sym.clone();
                let p = p.clone();
                async move {
                    let r = p.fetch_similar_symbols(&sym, limit).await?;
                    let provider = Provider::from_id_str(p.id()).ok_or_else(|| {
                        FinanceError::InternalError(format!("unknown provider id: {}", p.id()))
                    })?;
                    Ok((provider, r))
                }
            })
            .await?;
        Ok(recommendation_from_similar(
            self.symbol.to_string(),
            Some(provider_id),
            items,
            Some(limit),
        ))
    }

    /// Get news articles for this symbol.
    pub async fn news(&self) -> Result<Vec<News>> {
        {
            let cache = self.news_cache.read().await;
            if let Some(e) = cache.as_ref()
                && self.is_cache_fresh(Some(e))
            {
                return Ok(e.value.clone());
            }
        }
        let sym = self.symbol.clone();
        let data = self
            .providers
            .fetch(Capability::CORPORATE, move |p| {
                let sym = sym.clone();
                let p = p.clone();
                async move { p.fetch_news(&sym).await }
            })
            .await?;
        let news = data;
        #[cfg(feature = "translation")]
        let news = {
            let mut news = news;
            self.translate_response(&mut news).await?;
            news
        };
        if self.cache_ttl.is_some() {
            let mut c = self.news_cache.write().await;
            *c = Some(CacheEntry::new(news.clone()));
        }
        Ok(news)
    }

    /// Get the options chain.
    pub async fn options(&self, date: Option<i64>) -> Result<Options> {
        {
            let cache = self.options_cache.read().await;
            if let Some(e) = cache.get(&date)
                && self.is_cache_fresh(Some(e))
            {
                return Ok(e.value.clone());
            }
        }
        let sym = self.symbol.clone();
        let opts = self
            .providers
            .fetch(Capability::OPTIONS, move |p| {
                let sym = sym.clone();
                let p = p.clone();
                async move { p.fetch_options(&sym, date).await }
            })
            .await?;
        if self.cache_ttl.is_some() {
            let mut c = self.options_cache.write().await;
            self.cache_insert(&mut c, date, opts.clone());
        }
        Ok(opts)
    }

    /// Get financial statements.
    pub async fn financials(
        &self,
        stmt_type: StatementType,
        frequency: Frequency,
    ) -> Result<FinancialStatement> {
        let key = (stmt_type, frequency);
        {
            let cache = self.financials_cache.read().await;
            if let Some(e) = cache.get(&key)
                && self.is_cache_fresh(Some(e))
            {
                return Ok(e.value.clone());
            }
        }
        let sym = self.symbol.clone();
        let stmt = self
            .providers
            .fetch(Capability::FUNDAMENTALS, move |p| {
                let sym = sym.clone();
                let p = p.clone();
                async move { p.fetch_financials(&sym, stmt_type, frequency).await }
            })
            .await?;
        if self.cache_ttl.is_some() {
            let mut c = self.financials_cache.write().await;
            self.cache_insert(&mut c, key, stmt.clone());
        }
        Ok(stmt)
    }

    #[cfg(feature = "indicators")]
    /// Calculate all technical indicators from chart data.
    pub async fn indicators(
        &self,
        interval: Interval,
        range: TimeRange,
    ) -> Result<indicators::IndicatorsSummary> {
        {
            let cache = self.indicators_cache.read().await;
            if let Some(e) = cache.get(&(interval, range))
                && self.is_cache_fresh(Some(e))
            {
                return Ok(e.value.clone());
            }
        }
        let chart = self.chart(interval, range).await?;
        let ind = indicators::summary::calculate_indicators(&chart.candles);
        if self.cache_ttl.is_some() {
            let mut c = self.indicators_cache.write().await;
            self.cache_insert(&mut c, (interval, range), ind.clone());
        }
        Ok(ind)
    }

    /// Get SEC EDGAR filing history for this symbol.
    ///
    /// Always uses EDGAR directly — this is an EDGAR-specific API (CIK-based submission
    /// history and XBRL company facts) that no other provider replicates. For routable
    /// provider-agnostic filing data use [`filings`](Self::filings) instead.
    pub async fn edgar_submissions(&self) -> Result<EdgarSubmissions> {
        {
            let cache = self.edgar_submissions_cache.read().await;
            if let Some(e) = cache.as_ref()
                && self.is_cache_fresh(Some(e))
            {
                return Ok(e.value.clone());
            }
        }
        let cik = edgar::resolve_cik(&self.symbol).await?;
        let subs = edgar::submissions(cik).await?;
        if self.cache_ttl.is_some() {
            let mut c = self.edgar_submissions_cache.write().await;
            *c = Some(CacheEntry::new(subs.clone()));
        }
        Ok(subs)
    }

    /// Get SEC EDGAR company facts (structured XBRL financial data).
    ///
    /// Always uses EDGAR directly — XBRL `us-gaap`/`ifrs`/`dei` fact data is unique
    /// to the SEC's EDGAR API. For routable filing data use [`filings`](Self::filings).
    pub async fn edgar_company_facts(&self) -> Result<CompanyFacts> {
        {
            let cache = self.edgar_facts_cache.read().await;
            if let Some(e) = cache.as_ref()
                && self.is_cache_fresh(Some(e))
            {
                return Ok(e.value.clone());
            }
        }
        let cik = edgar::resolve_cik(&self.symbol).await?;
        let facts = edgar::company_facts(cik).await?;
        if self.cache_ttl.is_some() {
            let mut c = self.edgar_facts_cache.write().await;
            *c = Some(CacheEntry::new(facts.clone()));
        }
        Ok(facts)
    }

    /// Fetch SEC filings via the configured [`Capability::FILINGS`] provider.
    ///
    /// Routes through the provider system; EDGAR is always available as a fallback
    /// (auto-injected when no explicit FILINGS route is set). To prefer Polygon:
    /// `.route(Capability::FILINGS, &[Provider::Polygon, Provider::Edgar])`.
    ///
    /// For the full EDGAR submissions response or structured XBRL data, use
    /// [`edgar_submissions`](Self::edgar_submissions) / [`edgar_company_facts`](Self::edgar_company_facts).
    pub async fn filings(&self) -> Result<ProviderFilings> {
        let symbol = self.symbol.clone();
        self.providers
            .fetch(Capability::FILINGS, move |p| {
                let symbol = symbol.clone();
                let p = p.clone();
                async move { p.fetch_filings(&symbol).await }
            })
            .await
    }

    #[cfg(feature = "indicators")]
    /// Calculate a specific technical indicator over a time range.
    pub async fn indicator(
        &self,
        indicator: indicators::Indicator,
        interval: Interval,
        range: TimeRange,
    ) -> Result<indicators::IndicatorResult> {
        let chart = self.chart(interval, range).await?;
        let o = chart.open_prices();
        let h = chart.high_prices();
        let l = chart.low_prices();
        let c = chart.close_prices();
        let v = chart.volumes();
        use indicators::{Indicator, IndicatorResult};
        Ok(match indicator {
            Indicator::Sma(p) => IndicatorResult::Series(chart.sma(p)),
            Indicator::Ema(p) => IndicatorResult::Series(chart.ema(p)),
            Indicator::Rsi(p) => IndicatorResult::Series(chart.rsi(p)?),
            Indicator::Macd { fast, slow, signal } => {
                IndicatorResult::Macd(chart.macd(fast, slow, signal)?)
            }
            Indicator::Bollinger { period, std_dev } => {
                IndicatorResult::Bollinger(chart.bollinger_bands(period, std_dev)?)
            }
            Indicator::Atr(p) => IndicatorResult::Series(chart.atr(p)?),
            Indicator::Vwap => IndicatorResult::Series(crate::indicators::vwap(&h, &l, &c, &v)?),
            Indicator::Wma(p) => IndicatorResult::Series(crate::indicators::wma(&c, p)?),
            Indicator::Obv => IndicatorResult::Series(crate::indicators::obv(&c, &v)?),
            Indicator::Dema(p) => IndicatorResult::Series(crate::indicators::dema(&c, p)?),
            Indicator::Tema(p) => IndicatorResult::Series(crate::indicators::tema(&c, p)?),
            Indicator::Hma(p) => IndicatorResult::Series(crate::indicators::hma(&c, p)?),
            Indicator::Vwma(p) => IndicatorResult::Series(crate::indicators::vwma(&c, &v, p)?),
            Indicator::Alma {
                period,
                offset,
                sigma,
            } => IndicatorResult::Series(crate::indicators::alma(&c, period, offset, sigma)?),
            Indicator::McginleyDynamic(p) => {
                IndicatorResult::Series(crate::indicators::mcginley_dynamic(&c, p)?)
            }
            Indicator::Stochastic {
                k_period,
                k_slow,
                d_period,
            } => IndicatorResult::Stochastic(crate::indicators::stochastic(
                &h, &l, &c, k_period, k_slow, d_period,
            )?),
            Indicator::StochasticRsi {
                rsi_period,
                stoch_period,
                k_period,
                d_period,
            } => IndicatorResult::Stochastic(crate::indicators::stochastic_rsi(
                &c,
                rsi_period,
                stoch_period,
                k_period,
                d_period,
            )?),
            Indicator::Cci(p) => IndicatorResult::Series(crate::indicators::cci(&h, &l, &c, p)?),
            Indicator::WilliamsR(p) => {
                IndicatorResult::Series(crate::indicators::williams_r(&h, &l, &c, p)?)
            }
            Indicator::Roc(p) => IndicatorResult::Series(crate::indicators::roc(&c, p)?),
            Indicator::Momentum(p) => IndicatorResult::Series(crate::indicators::momentum(&c, p)?),
            Indicator::Cmo(p) => IndicatorResult::Series(crate::indicators::cmo(&c, p)?),
            Indicator::AwesomeOscillator { fast, slow } => {
                IndicatorResult::Series(crate::indicators::awesome_oscillator(&h, &l, fast, slow)?)
            }
            Indicator::CoppockCurve {
                long_roc,
                short_roc,
                wma_period,
            } => IndicatorResult::Series(crate::indicators::coppock_curve(
                &c, long_roc, short_roc, wma_period,
            )?),
            Indicator::Adx(p) => IndicatorResult::Series(crate::indicators::adx(&h, &l, &c, p)?),
            Indicator::Aroon(p) => IndicatorResult::Aroon(crate::indicators::aroon(&h, &l, p)?),
            Indicator::Supertrend { period, multiplier } => IndicatorResult::SuperTrend(
                crate::indicators::supertrend(&h, &l, &c, period, multiplier)?,
            ),
            Indicator::Ichimoku {
                conversion,
                base,
                lagging,
                displacement,
            } => IndicatorResult::Ichimoku(crate::indicators::ichimoku(
                &h,
                &l,
                &c,
                conversion,
                base,
                lagging,
                displacement,
            )?),
            Indicator::ParabolicSar { step, max } => {
                IndicatorResult::Series(crate::indicators::parabolic_sar(&h, &l, &c, step, max)?)
            }
            Indicator::BullBearPower(p) => {
                IndicatorResult::BullBearPower(crate::indicators::bull_bear_power(&h, &l, &c, p)?)
            }
            Indicator::ElderRay(p) => {
                IndicatorResult::ElderRay(crate::indicators::elder_ray(&h, &l, &c, p)?)
            }
            Indicator::KeltnerChannels {
                period,
                multiplier,
                atr_period,
            } => IndicatorResult::Keltner(crate::indicators::keltner_channels(
                &h, &l, &c, period, atr_period, multiplier,
            )?),
            Indicator::DonchianChannels(p) => {
                IndicatorResult::Donchian(crate::indicators::donchian_channels(&h, &l, p)?)
            }
            Indicator::TrueRange => {
                IndicatorResult::Series(crate::indicators::true_range(&h, &l, &c)?)
            }
            Indicator::ChoppinessIndex(p) => {
                IndicatorResult::Series(crate::indicators::choppiness_index(&h, &l, &c, p)?)
            }
            Indicator::Mfi(p) => {
                IndicatorResult::Series(crate::indicators::mfi(&h, &l, &c, &v, p)?)
            }
            Indicator::Cmf(p) => {
                IndicatorResult::Series(crate::indicators::cmf(&h, &l, &c, &v, p)?)
            }
            Indicator::ChaikinOscillator => {
                IndicatorResult::Series(crate::indicators::chaikin_oscillator(&h, &l, &c, &v)?)
            }
            Indicator::AccumulationDistribution => IndicatorResult::Series(
                crate::indicators::accumulation_distribution(&h, &l, &c, &v)?,
            ),
            Indicator::BalanceOfPower(p) => {
                IndicatorResult::Series(crate::indicators::balance_of_power(&o, &h, &l, &c, p)?)
            }
        })
    }

    #[cfg(feature = "backtesting")]
    /// Run a backtest with the given strategy and configuration.
    pub async fn backtest<S: backtesting::Strategy>(
        &self,
        strategy: S,
        interval: Interval,
        range: TimeRange,
        config: Option<backtesting::BacktestConfig>,
    ) -> backtesting::Result<backtesting::BacktestResult> {
        let config = config.unwrap_or_default();
        config.validate()?;
        let chart = self
            .chart(interval, range)
            .await
            .map_err(|e| backtesting::BacktestError::ChartError(e.to_string()))?;
        let dividends = self.dividends(range).await.unwrap_or_default();
        backtesting::BacktestEngine::new(config).run_with_dividends(
            &self.symbol,
            &chart.candles,
            strategy,
            &dividends,
        )
    }

    #[cfg(feature = "backtesting")]
    /// Run a backtest and compare performance against a benchmark symbol.
    pub async fn backtest_with_benchmark<S: backtesting::Strategy>(
        &self,
        strategy: S,
        interval: Interval,
        range: TimeRange,
        config: Option<backtesting::BacktestConfig>,
        benchmark: &str,
    ) -> backtesting::Result<backtesting::BacktestResult> {
        let config = config.unwrap_or_default();
        config.validate()?;
        let bench_ticker = Ticker::new(benchmark)
            .await
            .map_err(|e| backtesting::BacktestError::ChartError(e.to_string()))?;
        let (chart, bench_chart) = tokio::try_join!(
            self.chart(interval, range),
            bench_ticker.chart(interval, range)
        )
        .map_err(|e| backtesting::BacktestError::ChartError(e.to_string()))?;
        let dividends = self.dividends(range).await.unwrap_or_default();
        backtesting::BacktestEngine::new(config).run_with_benchmark(
            &self.symbol,
            &chart.candles,
            strategy,
            &dividends,
            benchmark,
            &bench_chart.candles,
        )
    }

    #[cfg(feature = "risk")]
    /// Compute a risk summary for this symbol.
    pub async fn risk(
        &self,
        interval: Interval,
        range: TimeRange,
        benchmark: Option<&str>,
    ) -> Result<risk::RiskSummary> {
        let chart = self.chart(interval, range).await?;
        let bench_returns = if let Some(sym) = benchmark {
            let bt = Ticker::new(sym).await?;
            Some(risk::candles_to_returns(
                &bt.chart(interval, range).await?.candles,
            ))
        } else {
            None
        };
        Ok(risk::compute_risk_summary(
            &chart.candles,
            bench_returns.as_deref(),
        ))
    }

    async fn ensure_quote(
        &self,
    ) -> Result<tokio::sync::RwLockReadGuard<'_, Option<CacheEntry<QuoteSummaryResponse>>>> {
        {
            let cache = self.quote_cache.read().await;
            if self.is_shared_cache_fresh(cache.as_ref()) {
                return Ok(cache);
            }
        }
        let _guard = self.quote_fetch.lock().await;
        {
            let cache = self.quote_cache.read().await;
            if self.is_shared_cache_fresh(cache.as_ref()) {
                return Ok(cache);
            }
        }
        let sym = self.symbol.clone();
        let summary = self
            .providers
            .fetch(Capability::QUOTE, move |p| {
                let sym = sym.clone();
                let p = p.clone();
                async move { p.fetch_quote(&sym).await }
            })
            .await?;
        {
            let mut cache = self.quote_cache.write().await;
            *cache = Some(CacheEntry::new(summary));
        }
        Ok(self.quote_cache.read().await)
    }
}

super::macros::define_quote_accessors! {
    price -> Price, price,
    summary_detail -> SummaryDetail, summary_detail,
    financial_data -> FinancialData, financial_data,
    key_stats -> DefaultKeyStatistics, default_key_statistics,
    asset_profile -> AssetProfile, asset_profile,
    calendar_events -> CalendarEvents, calendar_events,
    earnings -> Earnings, earnings,
    earnings_trend -> EarningsTrend, earnings_trend,
    earnings_history -> EarningsHistory, earnings_history,
    recommendation_trend -> RecommendationTrend, recommendation_trend,
    insider_holders -> InsiderHolders, insider_holders,
    insider_transactions -> InsiderTransactions, insider_transactions,
    institution_ownership -> InstitutionOwnership, institution_ownership,
    fund_ownership -> FundOwnership, fund_ownership,
    major_holders -> MajorHoldersBreakdown, major_holders_breakdown,
    share_purchase_activity -> NetSharePurchaseActivity, net_share_purchase_activity,
    quote_type -> QuoteTypeData, quote_type,
    summary_profile -> SummaryProfile, summary_profile,
    sec_filings -> SecFilings, sec_filings,
    grading_history -> UpgradeDowngradeHistory, upgrade_downgrade_history,
    fund_performance -> FundPerformance, fund_performance,
    fund_profile -> FundProfile, fund_profile,
    top_holdings -> TopHoldings, top_holdings,
    index_trend -> IndexTrend, index_trend,
    industry_trend -> IndustryTrend, industry_trend,
    sector_trend -> SectorTrend, sector_trend,
    equity_performance -> EquityPerformance, equity_performance,
}
