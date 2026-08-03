//! Symbol-specific data access from multiple providers.

use crate::adapters::edgar;
use crate::adapters::yahoo::client::{ClientConfig, YahooClient};
#[cfg(feature = "backtesting")]
use crate::backtesting;
use crate::constants::{Frequency, Interval, Region, StatementType, TimeRange};
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
use crate::utils::{CacheEntry, CacheMode, filter_by_range};
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
    cache_mode: CacheMode,
    include_logo: bool,
}

impl TickerBuilder {
    fn new(symbol: impl Into<String>) -> Self {
        Self {
            symbol: symbol.into().into(),
            config: ClientConfig::default(),
            shared_client: None,
            injected_providers: None,
            cache_mode: CacheMode::default(),
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
    /// Cache responses for `ttl` instead of for the handle's lifetime.
    pub fn cache(mut self, ttl: Duration) -> Self {
        self.cache_mode = CacheMode::Ttl(ttl);
        self
    }
    /// Disable caching — every call fetches fresh data.
    ///
    /// By default a `Ticker` caches each response for as long as the handle
    /// lives, so repeated accessor calls reuse one fetch.
    pub fn no_cache(mut self) -> Self {
        self.cache_mode = CacheMode::Off;
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
            cache_mode: self.cache_mode,
            include_logo: self.include_logo,
            #[cfg(feature = "translation")]
            translate_lang,
            quote_cache: Default::default(),
            quote_fetch: Arc::new(tokio::sync::Mutex::new(())),
            chart_cache: Default::default(),
            events_cache: Default::default(),
            news_cache: Default::default(),
            logo_cache: Default::default(),
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
/// Data is fetched on first access and cached for the lifetime of the handle.
/// Use the builder via [`Ticker::builder`] for custom configuration, including
/// [`cache`](TickerBuilder::cache) and [`no_cache`](TickerBuilder::no_cache).
pub struct Ticker {
    symbol: Arc<str>,
    providers: Arc<ProviderSet>,
    cache_mode: CacheMode,
    include_logo: bool,
    #[cfg(feature = "translation")]
    translate_lang: Option<crate::translation::Lang>,
    quote_cache: Cache<QuoteSummaryResponse>,
    quote_fetch: Arc<tokio::sync::Mutex<()>>,
    chart_cache: MapCache<(Interval, TimeRange), Chart>,
    events_cache: Cache<ChartEvents>,
    news_cache: Cache<Vec<News>>,
    logo_cache: Cache<(Option<String>, Option<String>)>,
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
        CacheEntry::is_fresh_entry(entry, self.cache_mode)
    }

    fn cache_insert<K: Eq + std::hash::Hash, V>(
        &self,
        map: &mut HashMap<K, CacheEntry<V>>,
        key: K,
        value: V,
    ) {
        crate::utils::cache_insert(
            map,
            key,
            value,
            self.cache_mode,
            crate::utils::EVICTION_THRESHOLD,
        );
    }

    /// Get full quote data, optionally including logo URLs.
    pub async fn quote<F>(&self) -> Result<Quote<F>>
    where
        F: Format,
        Quote<Both>: Into<Quote<F>>,
    {
        let logo_fut = async {
            if !self.include_logo {
                return (None, None);
            }
            if let Some(e) = self.logo_cache.read().await.as_ref()
                && self.is_cache_fresh(Some(e))
            {
                return e.value.clone();
            }
            let fetched = match self.providers.first_yahoo() {
                Ok(y) => y.get_logo_url(&self.symbol).await,
                Err(e) => Err(e),
            };
            // Only a successful lookup is cached. A symbol that genuinely has no
            // logo resolves to `(None, None)` and caches like any other answer;
            // a transport error does not, so one blip can't become permanent for
            // the handle's life.
            match fetched {
                Ok(logos) => {
                    if self.cache_mode.enabled() {
                        *self.logo_cache.write().await = Some(CacheEntry::new(logos.clone()));
                    }
                    logos
                }
                Err(_) => (None, None),
            }
        };

        let (cache, (logo_url, company_logo_url)) = tokio::join!(self.ensure_quote(), logo_fut);
        let cache = cache?;
        let summary = cache.as_ref().ok_or_else(|| {
            FinanceError::ApiError("Quote summary cache was empty after fetch".to_string())
        })?;
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
                async move {
                    p.as_chart()
                        .ok_or_else(|| p.not_supported(crate::providers::Operation::Chart))?
                        .fetch_chart(&sym, interval, range)
                        .await
                }
            })
            .await?;
        let chart = Self::chart_from_provider_data(data, Some(interval), Some(range));
        if self.cache_mode.enabled() {
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
                async move {
                    p.as_chart()
                        .ok_or_else(|| p.not_supported(crate::providers::Operation::ChartRange))?
                        .fetch_chart_range(&sym, interval, start, end)
                        .await
                }
            })
            .await?;
        Ok(Self::chart_from_provider_data(data, Some(interval), None))
    }

    async fn ensure_events(&self) -> Result<()> {
        {
            let cache = self.events_cache.read().await;
            if self.is_cache_fresh(cache.as_ref()) {
                return Ok(());
            }
        }
        let sym = self.symbol.clone();
        let events = self
            .providers
            .fetch(Capability::CORPORATE, move |p| {
                let sym = sym.clone();
                let p = p.clone();
                async move {
                    p.as_corporate()
                        .ok_or_else(|| p.not_supported(crate::providers::Operation::Events))?
                        .fetch_events(&sym)
                        .await
                }
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
                    let r = p
                        .as_corporate()
                        .ok_or_else(|| {
                            p.not_supported(crate::providers::Operation::Recommendations)
                        })?
                        .fetch_similar_symbols(&sym, limit)
                        .await?;
                    Ok((p.id(), r))
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
                async move {
                    p.as_corporate()
                        .ok_or_else(|| p.not_supported(crate::providers::Operation::News))?
                        .fetch_news(&sym)
                        .await
                }
            })
            .await?;
        let news = data;
        // Score titles before translation — VADER is English-lexicon based.
        #[cfg(feature = "sentiment")]
        let news = {
            let mut news = news;
            for article in news.iter_mut() {
                article.sentiment = Some(crate::models::sentiment::analyze(&article.title));
            }
            news
        };
        #[cfg(feature = "translation")]
        let news = {
            let mut news = news;
            self.translate_response(&mut news).await?;
            news
        };
        if self.cache_mode.enabled() {
            let mut c = self.news_cache.write().await;
            *c = Some(CacheEntry::new(news.clone()));
        }
        Ok(news)
    }

    /// Average sentiment across recent news headlines for this symbol.
    ///
    /// Positive = net bullish coverage, negative = net bearish. Returns a
    /// neutral, zero-confidence score when there are no headlines.
    ///
    /// Only available when the `sentiment` feature is enabled.
    #[cfg(feature = "sentiment")]
    pub async fn news_sentiment(&self) -> Result<crate::models::sentiment::Sentiment> {
        let news = self.news().await?;
        let scores: Vec<f64> = news
            .iter()
            .filter_map(|n| n.sentiment.as_ref().map(|s| s.score))
            .collect();
        Ok(crate::models::sentiment::aggregate(&scores)
            .unwrap_or_else(crate::models::sentiment::Sentiment::neutral))
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
                async move {
                    p.as_options()
                        .ok_or_else(|| p.not_supported(crate::providers::Operation::Options))?
                        .fetch_options(&sym, date)
                        .await
                }
            })
            .await?;
        if self.cache_mode.enabled() {
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
                async move {
                    p.as_fundamentals()
                        .ok_or_else(|| p.not_supported(crate::providers::Operation::Financials))?
                        .fetch_financials(&sym, stmt_type, frequency)
                        .await
                }
            })
            .await?;
        if self.cache_mode.enabled() {
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
        if self.cache_mode.enabled() {
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
        let subs = edgar::submissions_for_symbol(&self.symbol).await?;
        if self.cache_mode.enabled() {
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
        let facts = edgar::company_facts_for_symbol(&self.symbol).await?;
        if self.cache_mode.enabled() {
            let mut c = self.edgar_facts_cache.write().await;
            *c = Some(CacheEntry::new(facts.clone()));
        }
        Ok(facts)
    }

    /// Fetch SEC filings via the configured [`Capability::FILINGS`] provider.
    ///
    /// Routes through the provider system; EDGAR is always available as a fallback
    /// (auto-injected when no explicit FILINGS route is set). To prefer Polygon:
    /// `.route(Capability::FILINGS, [Provider::Polygon, Provider::Edgar])`.
    ///
    /// For the full EDGAR submissions response or structured XBRL data, use
    /// [`edgar_submissions`](Self::edgar_submissions) / [`edgar_company_facts`](Self::edgar_company_facts).
    pub async fn filings(&self) -> Result<ProviderFilings> {
        let symbol = self.symbol.clone();
        self.providers
            .fetch(Capability::FILINGS, move |p| {
                let symbol = symbol.clone();
                let p = p.clone();
                async move {
                    p.as_filings()
                        .ok_or_else(|| p.not_supported(crate::providers::Operation::Filings))?
                        .fetch_filings(&symbol)
                        .await
                }
            })
            .await
    }

    /// Fetch short-interest settlement reports via the configured
    /// [`Capability::FUNDAMENTALS`] provider. The default Yahoo route derives
    /// the current and prior-month snapshots from key statistics (keyless);
    /// route to Polygon for the full bi-monthly history:
    /// `.route(Capability::FUNDAMENTALS, [Provider::Polygon, Provider::Yahoo])`.
    pub async fn short_interest(&self) -> Result<Vec<crate::models::fundamentals::ShortInterest>> {
        let symbol = self.symbol.clone();
        self.providers
            .fetch(Capability::FUNDAMENTALS, move |p| {
                let symbol = symbol.clone();
                let p = p.clone();
                async move {
                    p.as_fundamentals()
                        .ok_or_else(|| p.not_supported(crate::providers::Operation::ShortInterest))?
                        .fetch_short_interest(&symbol)
                        .await
                }
            })
            .await
    }

    /// Fetch daily short-volume data via the configured
    /// [`Capability::FUNDAMENTALS`] provider (currently Polygon only).
    pub async fn short_volume(&self) -> Result<Vec<crate::models::fundamentals::ShortVolume>> {
        let symbol = self.symbol.clone();
        self.providers
            .fetch(Capability::FUNDAMENTALS, move |p| {
                let symbol = symbol.clone();
                let p = p.clone();
                async move {
                    p.as_fundamentals()
                        .ok_or_else(|| p.not_supported(crate::providers::Operation::ShortVolume))?
                        .fetch_short_volume(&symbol)
                        .await
                }
            })
            .await
    }

    /// Fetch share float and shares outstanding via the configured
    /// [`Capability::FUNDAMENTALS`] provider (Yahoo-derived on the default
    /// route; Polygon serves it too).
    pub async fn share_float(&self) -> Result<crate::models::fundamentals::ShareFloat> {
        let symbol = self.symbol.clone();
        self.providers
            .fetch(Capability::FUNDAMENTALS, move |p| {
                let symbol = symbol.clone();
                let p = p.clone();
                async move {
                    p.as_fundamentals()
                        .ok_or_else(|| p.not_supported(crate::providers::Operation::ShareFloat))?
                        .fetch_share_float(&symbol)
                        .await
                }
            })
            .await
    }

    /// Fetch the company's own press releases via the configured
    /// [`Capability::CORPORATE`] provider (currently FMP only). Distinct from
    /// [`news`](Self::news), which returns press coverage.
    pub async fn press_releases(
        &self,
        limit: u32,
    ) -> Result<Vec<crate::models::corporate::press_release::PressRelease>> {
        let symbol = self.symbol.clone();
        self.providers
            .fetch(Capability::CORPORATE, move |p| {
                let symbol = symbol.clone();
                let p = p.clone();
                async move {
                    p.as_corporate()
                        .ok_or_else(|| p.not_supported(crate::providers::Operation::PressReleases))?
                        .fetch_press_releases(&symbol, limit)
                        .await
                }
            })
            .await
    }

    /// Fetch the aggregated analyst price-target consensus (high/low/mean/median)
    /// via the configured [`Capability::FUNDAMENTALS`] provider (currently FMP
    /// only). Route with
    /// `.route(Capability::FUNDAMENTALS, [Provider::Fmp, Provider::Yahoo])`.
    pub async fn price_target_consensus(
        &self,
    ) -> Result<crate::models::fundamentals::PriceTargetConsensus> {
        let symbol = self.symbol.clone();
        self.providers
            .fetch(Capability::FUNDAMENTALS, move |p| {
                let symbol = symbol.clone();
                let p = p.clone();
                async move {
                    p.as_fundamentals()
                        .ok_or_else(|| {
                            p.not_supported(crate::providers::Operation::PriceTargetConsensus)
                        })?
                        .fetch_price_target_consensus(&symbol)
                        .await
                }
            })
            .await
    }

    /// Fetch price-target publication activity over trailing windows (last
    /// month/quarter/year/all time) via the configured
    /// [`Capability::FUNDAMENTALS`] provider (currently FMP only).
    pub async fn price_target_summary(
        &self,
    ) -> Result<crate::models::fundamentals::PriceTargetSummary> {
        let symbol = self.symbol.clone();
        self.providers
            .fetch(Capability::FUNDAMENTALS, move |p| {
                let symbol = symbol.clone();
                let p = p.clone();
                async move {
                    p.as_fundamentals()
                        .ok_or_else(|| {
                            p.not_supported(crate::providers::Operation::PriceTargetSummary)
                        })?
                        .fetch_price_target_summary(&symbol)
                        .await
                }
            })
            .await
    }

    /// Fetch the aggregated analyst rating consensus (grade distribution plus a
    /// headline label) via the configured [`Capability::FUNDAMENTALS`] provider
    /// (currently FMP only). Distinct from
    /// [`recommendations`](Self::recommendations), which returns similar symbols.
    pub async fn rating_consensus(&self) -> Result<crate::models::fundamentals::RatingConsensus> {
        let symbol = self.symbol.clone();
        self.providers
            .fetch(Capability::FUNDAMENTALS, move |p| {
                let symbol = symbol.clone();
                let p = p.clone();
                async move {
                    p.as_fundamentals()
                        .ok_or_else(|| {
                            p.not_supported(crate::providers::Operation::RatingConsensus)
                        })?
                        .fetch_rating_consensus(&symbol)
                        .await
                }
            })
            .await
    }

    /// Fetch the trailing-twelve-month key-metrics snapshot via the configured
    /// [`Capability::FUNDAMENTALS`] provider (currently FMP only).
    ///
    /// A TTM snapshot is a single always-current rollup, so callers do not need
    /// to fetch the latest fiscal period and reason about whether it is still
    /// current — see [`financials`](Self::financials) for the period series.
    pub async fn key_metrics_ttm(&self) -> Result<crate::models::fundamentals::KeyMetricsTtm> {
        let symbol = self.symbol.clone();
        self.providers
            .fetch(Capability::FUNDAMENTALS, move |p| {
                let symbol = symbol.clone();
                let p = p.clone();
                async move {
                    p.as_fundamentals()
                        .ok_or_else(|| p.not_supported(crate::providers::Operation::KeyMetricsTtm))?
                        .fetch_key_metrics_ttm(&symbol)
                        .await
                }
            })
            .await
    }

    /// Fetch the trailing-twelve-month financial-ratios snapshot via the
    /// configured [`Capability::FUNDAMENTALS`] provider (currently FMP only).
    pub async fn ratios_ttm(&self) -> Result<crate::models::fundamentals::FinancialRatiosTtm> {
        let symbol = self.symbol.clone();
        self.providers
            .fetch(Capability::FUNDAMENTALS, move |p| {
                let symbol = symbol.clone();
                let p = p.clone();
                async move {
                    p.as_fundamentals()
                        .ok_or_else(|| p.not_supported(crate::providers::Operation::RatiosTtm))?
                        .fetch_ratios_ttm(&symbol)
                        .await
                }
            })
            .await
    }

    /// Fetch reported executive compensation (most recent fiscal year first)
    /// via the configured [`Capability::CORPORATE`] provider (currently FMP
    /// only). Extracted from DEF 14A proxy statements, so it lags the filing.
    pub async fn executive_compensation(
        &self,
    ) -> Result<Vec<crate::models::corporate::governance::ExecutiveCompensation>> {
        let symbol = self.symbol.clone();
        self.providers
            .fetch(Capability::CORPORATE, move |p| {
                let symbol = symbol.clone();
                let p = p.clone();
                async move {
                    p.as_corporate()
                        .ok_or_else(|| {
                            p.not_supported(crate::providers::Operation::ExecutiveCompensation)
                        })?
                        .fetch_executive_compensation(&symbol)
                        .await
                }
            })
            .await
    }

    /// Fetch reported employee headcount history (most recent period first) via
    /// the configured [`Capability::CORPORATE`] provider (currently FMP only).
    /// Taken from 10-K cover pages, so it is annual.
    pub async fn employee_count(
        &self,
    ) -> Result<Vec<crate::models::corporate::governance::EmployeeCount>> {
        let symbol = self.symbol.clone();
        self.providers
            .fetch(Capability::CORPORATE, move |p| {
                let symbol = symbol.clone();
                let p = p.clone();
                async move {
                    p.as_corporate()
                        .ok_or_else(|| p.not_supported(crate::providers::Operation::EmployeeCount))?
                        .fetch_employee_count(&symbol)
                        .await
                }
            })
            .await
    }

    /// Fetch this fund's profile and portfolio holdings via the configured
    /// [`Capability::FUNDAMENTALS`] provider (currently Alpha Vantage only,
    /// and the only wired source of ETF holdings at all).
    ///
    /// Holdings come back heaviest-first. Errors for a symbol that is not a
    /// fund.
    pub async fn etf_profile(&self) -> Result<crate::models::fundamentals::EtfProfile> {
        let symbol = self.symbol.clone();
        self.providers
            .fetch(Capability::FUNDAMENTALS, move |p| {
                let symbol = symbol.clone();
                let p = p.clone();
                async move {
                    p.as_fundamentals()
                        .ok_or_else(|| p.not_supported(crate::providers::Operation::EtfProfile))?
                        .fetch_etf_profile(&symbol)
                        .await
                }
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
        Ok(indicators::compute_indicator(indicator, &chart)?)
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
        // Chart and dividends hit disjoint caches and disjoint capabilities
        // (CHART vs CORPORATE), so neither warms the other.
        let (chart, dividends) = tokio::join!(self.chart(interval, range), self.dividends(range));
        let chart = chart.map_err(|e| backtesting::BacktestError::ChartError(e.to_string()))?;
        let dividends = dividends.unwrap_or_default();
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
        let bench_fut = async {
            let bench_ticker = Ticker::new(benchmark).await?;
            bench_ticker.chart(interval, range).await
        };
        // `join!`, not `try_join!`: both charts are awaited to completion and the
        // errors resolved in a fixed order, so the surfaced error is always the
        // primary symbol's rather than whichever future happened to fail first.
        let (chart, bench_chart, dividends) = tokio::join!(
            self.chart(interval, range),
            bench_fut,
            self.dividends(range)
        );
        let chart = chart.map_err(|e| backtesting::BacktestError::ChartError(e.to_string()))?;
        let bench_chart =
            bench_chart.map_err(|e| backtesting::BacktestError::ChartError(e.to_string()))?;
        let dividends = dividends.unwrap_or_default();
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
        let bench_fut = async {
            let Some(sym) = benchmark else {
                return Result::Ok(None);
            };
            let bt = Ticker::new(sym).await?;
            let bench_chart = bt.chart(interval, range).await?;
            Result::Ok(Some(risk::candles_to_returns(&bench_chart.candles)))
        };
        // `join!`, not `try_join!`: resolving in a fixed order keeps the primary
        // symbol's error as the surfaced one, matching the previous sequential
        // `self.chart(..).await?` ordering.
        let (chart, bench_returns) = tokio::join!(self.chart(interval, range), bench_fut);
        let chart = chart?;
        let bench_returns = bench_returns?;
        Ok(risk::compute_risk_summary(
            &chart.candles,
            bench_returns.as_deref(),
        ))
    }

    /// Aggregate upcoming financial events for this ticker into a single
    /// time-sorted list.
    ///
    /// Combines earnings, ex-dividend and dividend-payment dates with standard
    /// monthly options expirations, plus — when the `fred` feature is enabled —
    /// a curated set of major economic releases (CPI, NFP, GDP, …). Limited to
    /// the forward window `[now, now + range]` and sorted ascending by
    /// timestamp.
    ///
    /// Options are best-effort: a symbol with no listed options contributes no
    /// expiration events rather than failing the call.
    pub async fn calendar(
        &self,
        range: TimeRange,
    ) -> Result<Vec<crate::models::calendar::CalendarEvent>> {
        let now = chrono::Utc::now().timestamp();
        let window = (now, now + range.approx_duration_secs());

        // The FRED economic-release fetch is independent of the per-symbol
        // quote/options work, so run all three concurrently.
        #[cfg(feature = "fred")]
        let (calendar_events, options, releases) = tokio::join!(
            self.calendar_events(),
            self.options(None),
            crate::adapters::fred::release_dates(),
        );
        #[cfg(not(feature = "fred"))]
        let (calendar_events, options) = tokio::join!(self.calendar_events(), self.options(None));

        let calendar_events = calendar_events?;
        let options = options.ok();

        let mut events = crate::models::calendar::build_symbol_events(
            &self.symbol,
            calendar_events.as_ref(),
            options.as_ref(),
            window,
        );

        #[cfg(feature = "fred")]
        if let Ok(releases) = releases {
            events.extend(crate::models::calendar::build_economic_events(
                releases, window,
            ));
        }

        crate::models::calendar::sort_events(&mut events);
        Ok(events)
    }

    async fn ensure_quote(
        &self,
    ) -> Result<tokio::sync::RwLockReadGuard<'_, Option<CacheEntry<QuoteSummaryResponse>>>> {
        {
            let cache = self.quote_cache.read().await;
            if self.is_cache_fresh(cache.as_ref()) {
                return Ok(cache);
            }
        }
        let _guard = self.quote_fetch.lock().await;
        {
            let cache = self.quote_cache.read().await;
            if self.is_cache_fresh(cache.as_ref()) {
                return Ok(cache);
            }
        }
        let sym = self.symbol.clone();
        let summary = self
            .providers
            .fetch(Capability::QUOTE, move |p| {
                let sym = sym.clone();
                let p = p.clone();
                async move {
                    p.as_quote()
                        .ok_or_else(|| p.not_supported(crate::providers::Operation::Quote))?
                        .fetch_quote(&sym)
                        .await
                }
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::providers::mock::{CountingProvider, provider_set};

    #[tokio::test]
    async fn default_caches_quote_across_accessors() {
        let provider = CountingProvider::new();
        let ticker = Ticker::builder("AAPL")
            .with_provider_set(provider_set(Arc::clone(&provider)))
            .build()
            .await
            .unwrap();

        let _ = ticker.price().await.unwrap();
        let _ = ticker.summary_detail().await.unwrap();
        let _ = ticker.asset_profile().await.unwrap();

        assert_eq!(provider.quotes(), 1);
    }

    #[tokio::test]
    async fn no_cache_refetches_every_accessor() {
        let provider = CountingProvider::new();
        let ticker = Ticker::builder("AAPL")
            .with_provider_set(provider_set(Arc::clone(&provider)))
            .no_cache()
            .build()
            .await
            .unwrap();

        let _ = ticker.price().await.unwrap();
        let _ = ticker.summary_detail().await.unwrap();
        let _ = ticker.asset_profile().await.unwrap();

        assert_eq!(provider.quotes(), 3);
    }

    #[tokio::test]
    async fn charts_cache_per_interval_and_range() {
        let provider = CountingProvider::new();
        let ticker = Ticker::builder("AAPL")
            .with_provider_set(provider_set(Arc::clone(&provider)))
            .build()
            .await
            .unwrap();

        let _ = ticker
            .chart(Interval::OneDay, TimeRange::OneMonth)
            .await
            .unwrap();
        let _ = ticker
            .chart(Interval::OneDay, TimeRange::OneMonth)
            .await
            .unwrap();
        assert_eq!(provider.charts(), 1);

        let _ = ticker
            .chart(Interval::OneDay, TimeRange::OneYear)
            .await
            .unwrap();
        assert_eq!(provider.charts(), 2);
    }

    #[tokio::test]
    async fn unresolved_logo_is_not_cached() {
        let provider = CountingProvider::new();
        let ticker = Ticker::builder("AAPL")
            .with_provider_set(provider_set(Arc::clone(&provider)))
            .logo()
            .build()
            .await
            .unwrap();

        let _: Quote<crate::format::Raw> = ticker.quote().await.unwrap();
        assert!(
            ticker.logo_cache.read().await.is_none(),
            "an unresolved logo must not be cached, or one blip is permanent"
        );
    }

    #[tokio::test(start_paused = true)]
    async fn ttl_expires() {
        let provider = CountingProvider::new();
        let ticker = Ticker::builder("AAPL")
            .with_provider_set(provider_set(Arc::clone(&provider)))
            .cache(Duration::from_secs(60))
            .build()
            .await
            .unwrap();

        let _ = ticker
            .chart(Interval::OneDay, TimeRange::OneMonth)
            .await
            .unwrap();
        tokio::time::advance(Duration::from_secs(120)).await;
        let _ = ticker
            .chart(Interval::OneDay, TimeRange::OneMonth)
            .await
            .unwrap();

        assert_eq!(provider.charts(), 2);
    }
}
