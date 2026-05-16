//! Symbol-specific data access from multiple providers.

use crate::constants::{Interval, TimeRange};
use crate::error::{FinanceError, Result};
use crate::models::chart::Chart;
use crate::models::chart::events::ChartEvents;
use crate::models::chart::{CapitalGain, Dividend, Split};
use crate::models::corporate::recommendation::Recommendation;
use crate::models::fundamentals::FinancialStatement;
use crate::models::options::Options;
use crate::models::quote::Module;
use crate::models::quote::{
    AssetProfile, CalendarEvents, DefaultKeyStatistics, Earnings, EarningsHistory, EarningsTrend,
    EquityPerformance, FinancialData, FundOwnership, FundPerformance, FundProfile, IndexTrend,
    IndustryTrend, InsiderHolders, InsiderTransactions, InstitutionOwnership,
    MajorHoldersBreakdown, NetSharePurchaseActivity, Price, Quote, QuoteSummaryResponse,
    QuoteTypeData, RecommendationTrend, SecFilings, SectorTrend, SummaryDetail, SummaryProfile,
    TopHoldings, UpgradeDowngradeHistory,
};
use crate::providers::{Capability, Fetch, Merge, Prefer, Provider, ProviderSet, build_providers};
use crate::utils::{CacheEntry, EVICTION_THRESHOLD, filter_by_range};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;

type Cache<T> = Arc<RwLock<Option<CacheEntry<T>>>>;
type MapCache<K, V> = Arc<RwLock<HashMap<K, CacheEntry<V>>>>;
/// Builder for constructing a [`Ticker`] with optional configuration.
///
/// Construct via [`Ticker::builder`]. All builder methods are optional;
/// call [`build`](TickerBuilder::build) to finalize.
pub struct TickerBuilder {
    symbol: Arc<str>,
    config: crate::adapters::yahoo::client::ClientConfig,
    provider_ids: Option<Vec<Provider>>,
    fetch: Fetch,
    merge: Option<Arc<dyn Merge>>,
    cache_ttl: Option<Duration>,
    include_logo: bool,
    value_format: crate::constants::ValueFormat,
}

impl TickerBuilder {
    fn new(symbol: impl Into<String>) -> Self {
        Self {
            symbol: symbol.into().into(),
            config: crate::adapters::yahoo::client::ClientConfig::default(),
            provider_ids: None,
            fetch: Fetch::Sequential,
            merge: None,
            cache_ttl: None,
            include_logo: false,
            value_format: crate::constants::ValueFormat::default(),
        }
    }
    /// Set the region (automatically sets correct lang and region).
    pub fn region(mut self, region: crate::constants::Region) -> Self {
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
    pub(crate) fn config(mut self, c: crate::adapters::yahoo::client::ClientConfig) -> Self {
        self.config = c;
        self
    }
    /// Configure which providers to use, in priority order. Default: `[Yahoo]`.
    pub fn providers(mut self, ids: &[Provider]) -> Self {
        self.provider_ids = Some(ids.to_vec());
        self
    }
    /// Configure how providers are queried. Default: `Sequential`.
    pub fn fetch(mut self, mode: Fetch) -> Self {
        self.fetch = mode;
        self
    }
    /// Configure how results from multiple providers are combined. Default: `Prefer`.
    pub fn merge(mut self, policy: impl Merge + 'static) -> Self {
        self.merge = Some(Arc::new(policy));
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

    /// Set the value format for [`quote_value`](Ticker::quote_value) responses.
    ///
    /// Controls how [`FormattedValue`](crate::FormattedValue) fields are
    /// serialized in the returned JSON:
    /// - [`ValueFormat::Raw`] — plain numeric values **(default)**; best for
    ///   programmatic use and calculations
    /// - [`ValueFormat::Pretty`] — formatted strings (e.g. `"$123.45"`, `"14.78B"`)
    /// - [`ValueFormat::Both`] — full `{ raw, fmt, longFmt }` object
    pub fn format(mut self, format: crate::constants::ValueFormat) -> Self {
        self.value_format = format;
        self
    }

    /// Build the Ticker instance.
    pub async fn build(self) -> Result<Ticker> {
        let ids = self.provider_ids.unwrap_or_else(|| vec![Provider::Yahoo]);
        let providers = Arc::new(
            build_providers(
                &ids,
                &self.config,
                self.fetch,
                self.merge.unwrap_or_else(|| Arc::new(Prefer)),
            )
            .await?,
        );
        Ok(Ticker {
            symbol: self.symbol,
            providers,
            cache_ttl: self.cache_ttl,
            include_logo: self.include_logo,
            value_format: self.value_format,
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
    value_format: crate::constants::ValueFormat,
    quote_cache: Cache<QuoteSummaryResponse>,
    quote_fetch: Arc<tokio::sync::Mutex<()>>,
    chart_cache: MapCache<(Interval, TimeRange), Chart>,
    events_cache: Cache<ChartEvents>,
    news_cache: Cache<Vec<crate::models::corporate::news::News>>,
    options_cache: MapCache<Option<i64>, Options>,
    financials_cache: MapCache<
        (crate::constants::StatementType, crate::constants::Frequency),
        FinancialStatement,
    >,
    #[cfg(feature = "indicators")]
    indicators_cache: MapCache<(Interval, TimeRange), crate::indicators::IndicatorsSummary>,
    edgar_submissions_cache: Cache<crate::models::filings::EdgarSubmissions>,
    edgar_facts_cache: Cache<crate::models::filings::CompanyFacts>,
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
    #[allow(dead_code)]
    pub(crate) fn provider_set(&self) -> &Arc<ProviderSet> {
        &self.providers
    }

    fn is_cache_fresh<T>(&self, entry: Option<&CacheEntry<T>>) -> bool {
        CacheEntry::is_fresh_with_ttl(entry, self.cache_ttl)
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

    fn quote_summary_url(&self) -> String {
        format!(
            "{}?modules={}",
            crate::adapters::yahoo::endpoints::api::quote_summary(&self.symbol),
            Module::all()
                .iter()
                .map(|m| m.as_str())
                .collect::<Vec<_>>()
                .join(",")
        )
    }

    /// Get full quote data, optionally including logo URLs.
    pub async fn quote(&self) -> Result<Quote> {
        let (cache, logo_pair) = if self.include_logo {
            let sym = self.symbol.clone();
            let yahoo = self.providers.first_yahoo().ok();
            let logo_future = async move {
                if let Some(y) = yahoo {
                    Some(y.get_logo_url(&sym).await)
                } else {
                    None
                }
            };
            let (cache_result, logos) = tokio::join!(self.ensure_quote_summary(), logo_future);
            (cache_result?, logos.unwrap_or((None, None)))
        } else {
            (self.ensure_quote_summary().await?, (None, None))
        };
        let entry = cache.as_ref().ok_or_else(|| FinanceError::SymbolNotFound {
            symbol: Some(self.symbol.to_string()),
            context: "Quote summary not loaded".into(),
        })?;
        let mut q = Quote::from_response(&entry.value, logo_pair.0, logo_pair.1);
        q.provider_id = Some(Provider::Yahoo);
        Ok(q)
    }

    /// Get quote data as a flat JSON value, with [`FormattedValue`](crate::FormattedValue)
    /// fields transformed according to the format configured via
    /// [`TickerBuilder::format`].
    ///
    /// By default (`Raw`), every `FormattedValue` field is flattened to its
    /// plain numeric value — no `.raw`/`.fmt` unwrapping needed. Use
    /// [`ValueFormat::Pretty`] for human-readable strings or
    /// [`ValueFormat::Both`] to preserve the full object.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use finance_query::{Ticker, ValueFormat};
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let ticker = Ticker::new("AAPL").await?;
    /// let json = ticker.quote_value().await?;
    /// // json["regularMarketPrice"] == 182.63  (plain f64, no unwrapping)
    /// # Ok(())
    /// # }
    /// ```
    pub async fn quote_value(&self) -> crate::error::Result<serde_json::Value> {
        let quote = self.quote().await?;
        let json = serde_json::to_value(&quote).map_err(FinanceError::JsonParseError)?;
        Ok(self.value_format.transform(json))
    }

    fn chart_from_provider_data(
        data: crate::providers::types::ChartData,
        interval: Option<Interval>,
        range: Option<TimeRange>,
    ) -> Chart {
        data.into_chart(interval, range)
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
        let data = if self.providers.merger().wants_all() {
            let mut results = self
                .providers
                .try_fetch_all(Capability::CHART, move |p| {
                    let sym = sym.clone();
                    let p = p.clone();
                    async move { p.fetch_chart(&sym, interval, range).await }
                })
                .await?;
            let primary = results.remove(0);
            results.into_iter().fold(primary, |acc, fb| {
                self.providers.merger().merge_chart(acc, fb)
            })
        } else {
            self.providers
                .try_fetch(Capability::CHART, move |p| {
                    let sym = sym.clone();
                    let p = p.clone();
                    async move { p.fetch_chart(&sym, interval, range).await }
                })
                .await?
        };
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
            .try_fetch(Capability::CHART, move |p| {
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
            if self.is_cache_fresh(cache.as_ref()) {
                return Ok(());
            }
        }
        let sym = self.symbol.clone();
        let events = self
            .providers
            .try_fetch(Capability::CORPORATE, move |p| {
                let sym = sym.clone();
                let p = p.clone();
                async move { p.fetch_events(&sym).await }
            })
            .await?;
        let mut cache = self.events_cache.write().await;
        *cache = Some(CacheEntry::new(events.into_chart_events()));
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
    pub async fn dividend_analytics(
        &self,
        range: TimeRange,
    ) -> Result<crate::models::chart::DividendAnalytics> {
        let divs = self.dividends(range).await?;
        Ok(crate::models::chart::DividendAnalytics::from_dividends(
            &divs,
        ))
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
            .try_fetch(Capability::CORPORATE, move |p| {
                let sym = sym.clone();
                let p = p.clone();
                async move {
                    let r = p.fetch_similar_symbols(&sym, limit).await?;
                    Ok((p.id(), r))
                }
            })
            .await?;
        Ok(crate::providers::types::recommendation_from_similar(
            self.symbol.to_string(),
            Some(provider_id.into()),
            items,
            Some(limit),
        ))
    }

    /// Get news articles for this symbol.
    pub async fn news(&self) -> Result<Vec<crate::models::corporate::news::News>> {
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
            .try_fetch(Capability::CORPORATE, move |p| {
                let sym = sym.clone();
                let p = p.clone();
                async move { p.fetch_news(&sym).await }
            })
            .await?;
        let news: Vec<crate::models::corporate::news::News> =
            data.into_iter().map(Into::into).collect();
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
        let opts = if self.providers.merger().wants_all() {
            let mut results = self
                .providers
                .try_fetch_all(Capability::OPTIONS, move |p| {
                    let sym = sym.clone();
                    let p = p.clone();
                    async move { p.fetch_options(&sym, date).await }
                })
                .await?;
            let primary = results.remove(0);
            results
                .into_iter()
                .fold(primary, |acc, fb| {
                    self.providers.merger().merge_options(acc, fb)
                })
                .into_options()
        } else {
            self.providers
                .try_fetch(Capability::OPTIONS, move |p| {
                    let sym = sym.clone();
                    let p = p.clone();
                    async move { p.fetch_options(&sym, date).await }
                })
                .await?
                .into_options()
        };
        if self.cache_ttl.is_some() {
            let mut c = self.options_cache.write().await;
            self.cache_insert(&mut c, date, opts.clone());
        }
        Ok(opts)
    }

    /// Get financial statements.
    pub async fn financials(
        &self,
        stmt_type: crate::constants::StatementType,
        frequency: crate::constants::Frequency,
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
        let stmt = if self.providers.merger().wants_all() {
            let mut results = self
                .providers
                .try_fetch_all(Capability::FUNDAMENTALS, move |p| {
                    let sym = sym.clone();
                    let p = p.clone();
                    async move { p.fetch_financials(&sym, stmt_type, frequency).await }
                })
                .await?;
            let primary = results.remove(0);
            results
                .into_iter()
                .fold(primary, |acc, fb| {
                    self.providers.merger().merge_financials(acc, fb)
                })
                .into_financial_statement()
        } else {
            self.providers
                .try_fetch(Capability::FUNDAMENTALS, move |p| {
                    let sym = sym.clone();
                    let p = p.clone();
                    async move { p.fetch_financials(&sym, stmt_type, frequency).await }
                })
                .await?
                .into_financial_statement()
        };
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
    ) -> Result<crate::indicators::IndicatorsSummary> {
        {
            let cache = self.indicators_cache.read().await;
            if let Some(e) = cache.get(&(interval, range))
                && self.is_cache_fresh(Some(e))
            {
                return Ok(e.value.clone());
            }
        }
        let chart = self.chart(interval, range).await?;
        let ind = crate::indicators::summary::calculate_indicators(&chart.candles);
        if self.cache_ttl.is_some() {
            let mut c = self.indicators_cache.write().await;
            self.cache_insert(&mut c, (interval, range), ind.clone());
        }
        Ok(ind)
    }

    /// Get SEC EDGAR filing history for this symbol.
    pub async fn edgar_submissions(&self) -> Result<crate::models::filings::EdgarSubmissions> {
        {
            let cache = self.edgar_submissions_cache.read().await;
            if let Some(e) = cache.as_ref()
                && self.is_cache_fresh(Some(e))
            {
                return Ok(e.value.clone());
            }
        }
        let cik = crate::edgar::resolve_cik(&self.symbol).await?;
        let subs = crate::edgar::submissions(cik).await?;
        if self.cache_ttl.is_some() {
            let mut c = self.edgar_submissions_cache.write().await;
            *c = Some(CacheEntry::new(subs.clone()));
        }
        Ok(subs)
    }

    /// Get SEC EDGAR company facts (structured XBRL financial data).
    pub async fn edgar_company_facts(&self) -> Result<crate::models::filings::CompanyFacts> {
        {
            let cache = self.edgar_facts_cache.read().await;
            if let Some(e) = cache.as_ref()
                && self.is_cache_fresh(Some(e))
            {
                return Ok(e.value.clone());
            }
        }
        let cik = crate::edgar::resolve_cik(&self.symbol).await?;
        let facts = crate::edgar::company_facts(cik).await?;
        if self.cache_ttl.is_some() {
            let mut c = self.edgar_facts_cache.write().await;
            *c = Some(CacheEntry::new(facts.clone()));
        }
        Ok(facts)
    }

    #[cfg(feature = "indicators")]
    /// Calculate a specific technical indicator over a time range.
    pub async fn indicator(
        &self,
        indicator: crate::indicators::Indicator,
        interval: Interval,
        range: TimeRange,
    ) -> Result<crate::indicators::IndicatorResult> {
        let chart = self.chart(interval, range).await?;
        use crate::indicators::{Indicator, IndicatorResult};
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
            Indicator::Vwap => {
                let (h, l, c, v) = (
                    chart.high_prices(),
                    chart.low_prices(),
                    chart.close_prices(),
                    chart.volumes(),
                );
                IndicatorResult::Series(crate::indicators::vwap(&h, &l, &c, &v)?)
            }
            Indicator::Wma(p) => {
                IndicatorResult::Series(crate::indicators::wma(&chart.close_prices(), p)?)
            }
            Indicator::Obv => IndicatorResult::Series(crate::indicators::obv(
                &chart.close_prices(),
                &chart.volumes(),
            )?),
            _ => {
                return Err(FinanceError::NotSupported {
                    provider: "ticker",
                    operation: "indicator",
                });
            }
        })
    }

    #[cfg(feature = "backtesting")]
    /// Run a backtest with the given strategy and configuration.
    pub async fn backtest<S: crate::backtesting::Strategy>(
        &self,
        strategy: S,
        interval: Interval,
        range: TimeRange,
        config: Option<crate::backtesting::BacktestConfig>,
    ) -> crate::backtesting::Result<crate::backtesting::BacktestResult> {
        let config = config.unwrap_or_default();
        config.validate()?;
        let chart = self
            .chart(interval, range)
            .await
            .map_err(|e| crate::backtesting::BacktestError::ChartError(e.to_string()))?;
        let dividends = self.dividends(range).await.unwrap_or_default();
        crate::backtesting::BacktestEngine::new(config).run_with_dividends(
            &self.symbol,
            &chart.candles,
            strategy,
            &dividends,
        )
    }

    #[cfg(feature = "backtesting")]
    /// Run a backtest and compare performance against a benchmark symbol.
    pub async fn backtest_with_benchmark<S: crate::backtesting::Strategy>(
        &self,
        strategy: S,
        interval: Interval,
        range: TimeRange,
        config: Option<crate::backtesting::BacktestConfig>,
        benchmark: &str,
    ) -> crate::backtesting::Result<crate::backtesting::BacktestResult> {
        let config = config.unwrap_or_default();
        config.validate()?;
        let bench_ticker = Ticker::new(benchmark)
            .await
            .map_err(|e| crate::backtesting::BacktestError::ChartError(e.to_string()))?;
        let (chart, bench_chart) = tokio::try_join!(
            self.chart(interval, range),
            bench_ticker.chart(interval, range)
        )
        .map_err(|e| crate::backtesting::BacktestError::ChartError(e.to_string()))?;
        let dividends = self.dividends(range).await.unwrap_or_default();
        crate::backtesting::BacktestEngine::new(config).run_with_benchmark(
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
    ) -> Result<crate::risk::RiskSummary> {
        let chart = self.chart(interval, range).await?;
        let bench_returns = if let Some(sym) = benchmark {
            let bt = Ticker::new(sym).await?;
            Some(crate::risk::candles_to_returns(
                &bt.chart(interval, range).await?.candles,
            ))
        } else {
            None
        };
        Ok(crate::risk::compute_risk_summary(
            &chart.candles,
            bench_returns.as_deref(),
        ))
    }

    async fn ensure_quote_summary(
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
        let yahoo = self.providers.first_yahoo()?;
        let resp = yahoo.request_with_crumb(&self.quote_summary_url()).await?;
        let json = resp.json::<serde_json::Value>().await?;
        let summary = QuoteSummaryResponse::from_json(json, &self.symbol)?;
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
