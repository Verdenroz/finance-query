//! Ticker implementation for accessing symbol-specific data from Yahoo Finance.
//!
//! Provides async interface for fetching quotes, charts, financials, and news.

use super::macros;
use crate::client::{ClientConfig, YahooClient};
use crate::constants::{Interval, TimeRange};
use crate::error::Result;
use crate::models::chart::events::ChartEvents;
use crate::models::chart::response::ChartResponse;
use crate::models::chart::{CapitalGain, Chart, Dividend, Split};
use crate::models::financials::FinancialStatement;
use crate::models::options::Options;
use crate::models::quote::{
    AssetProfile, CalendarEvents, DefaultKeyStatistics, Earnings, EarningsHistory, EarningsTrend,
    EquityPerformance, FinancialData, FundOwnership, FundPerformance, FundProfile, IndexTrend,
    IndustryTrend, InsiderHolders, InsiderTransactions, InstitutionOwnership,
    MajorHoldersBreakdown, Module, NetSharePurchaseActivity, Price, Quote, QuoteSummaryResponse,
    QuoteTypeData, RecommendationTrend, SecFilings, SectorTrend, SummaryDetail, SummaryProfile,
    TopHoldings, UpgradeDowngradeHistory,
};
use crate::models::recommendation::Recommendation;
use crate::models::recommendation::response::RecommendationResponse;
use crate::utils::{CacheEntry, EVICTION_THRESHOLD, filter_by_range};
use std::collections::HashMap;
use std::sync::{Arc, OnceLock};
use std::time::Duration;
use tokio::sync::RwLock;

// Type aliases to keep struct definitions readable.
type Cache<T> = Arc<RwLock<Option<CacheEntry<T>>>>;
type MapCache<K, V> = Arc<RwLock<HashMap<K, CacheEntry<V>>>>;

/// Opaque handle to a shared Yahoo Finance client session.
///
/// Allows multiple [`Ticker`] and [`Tickers`](crate::Tickers) instances to share
/// a single authenticated session, avoiding redundant authentication handshakes.
///
/// Obtain a handle from an existing `Ticker` via [`Ticker::client_handle()`],
/// then pass it to other builders via `.client()`.
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
/// // Share the same session — no additional auth
/// let msft = Ticker::builder("MSFT").client(handle.clone()).build().await?;
/// let googl = Ticker::builder("GOOGL").client(handle).build().await?;
/// # Ok(())
/// # }
/// ```
#[derive(Clone)]
pub struct ClientHandle(pub(crate) Arc<YahooClient>);

/// Builder for Ticker
///
/// Provides a fluent API for constructing Ticker instances.
pub struct TickerBuilder {
    symbol: Arc<str>,
    config: ClientConfig,
    shared_client: Option<ClientHandle>,
    cache_ttl: Option<Duration>,
    include_logo: bool,
}

impl TickerBuilder {
    fn new(symbol: impl Into<String>) -> Self {
        Self {
            symbol: symbol.into().into(),
            config: ClientConfig::default(),
            shared_client: None,
            cache_ttl: None,
            include_logo: false,
        }
    }

    /// Set the region (automatically sets correct lang and region)
    ///
    /// This is the recommended way to configure regional settings as it ensures
    /// lang and region are correctly paired.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use finance_query::{Ticker, Region};
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let ticker = Ticker::builder("2330.TW")
    ///     .region(Region::Taiwan)
    ///     .build()
    ///     .await?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn region(mut self, region: crate::constants::Region) -> Self {
        self.config.lang = region.lang().to_string();
        self.config.region = region.region().to_string();
        self
    }

    /// Set the language code (e.g., "en-US", "ja-JP", "de-DE")
    ///
    /// For standard regions, prefer using `.region()` instead to ensure
    /// correct lang/region pairing.
    pub fn lang(mut self, lang: impl Into<String>) -> Self {
        self.config.lang = lang.into();
        self
    }

    /// Set the region code (e.g., "US", "JP", "DE")
    ///
    /// For standard regions, prefer using `.region()` instead to ensure
    /// correct lang/region pairing.
    pub fn region_code(mut self, region: impl Into<String>) -> Self {
        self.config.region = region.into();
        self
    }

    /// Set the HTTP request timeout
    pub fn timeout(mut self, timeout: Duration) -> Self {
        self.config.timeout = timeout;
        self
    }

    /// Set the proxy URL
    pub fn proxy(mut self, proxy: impl Into<String>) -> Self {
        self.config.proxy = Some(proxy.into());
        self
    }

    /// Set a complete ClientConfig (overrides any previously set individual config fields)
    pub fn config(mut self, config: ClientConfig) -> Self {
        self.config = config;
        self
    }

    /// Share an existing authenticated session instead of creating a new one.
    ///
    /// This avoids redundant authentication when you need multiple `Ticker`
    /// instances or want to share a session between `Ticker` and [`crate::Tickers`].
    ///
    /// Obtain a [`ClientHandle`] from any existing `Ticker` via
    /// [`Ticker::client_handle()`].
    ///
    /// When set, the builder's `config`, `timeout`, `proxy`, `lang`, and `region`
    /// settings are ignored (the shared session's configuration is used instead).
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
    pub fn client(mut self, handle: ClientHandle) -> Self {
        self.shared_client = Some(handle);
        self
    }

    /// Enable response caching with a time-to-live.
    ///
    /// By default caching is **disabled** — every call fetches fresh data.
    /// When enabled, responses are reused until the TTL expires, then
    /// automatically re-fetched. Expired entries in map-based caches
    /// (chart, options, financials) are evicted on the next write to
    /// limit memory growth.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// use finance_query::Ticker;
    /// use std::time::Duration;
    ///
    /// let ticker = Ticker::builder("AAPL")
    ///     .cache(Duration::from_secs(30))
    ///     .build()
    ///     .await?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn cache(mut self, ttl: Duration) -> Self {
        self.cache_ttl = Some(ttl);
        self
    }

    /// Include company logo URLs in quote responses.
    ///
    /// When enabled, `quote()` will fetch logo URLs in parallel with the
    /// quote summary, adding a small extra request.
    pub fn logo(mut self) -> Self {
        self.include_logo = true;
        self
    }

    /// Build the Ticker instance
    pub async fn build(self) -> Result<Ticker> {
        let client = match self.shared_client {
            Some(handle) => handle.0,
            None => Arc::new(YahooClient::new(self.config).await?),
        };

        Ok(Ticker {
            symbol: self.symbol,
            client,
            cache_ttl: self.cache_ttl,
            include_logo: self.include_logo,
            quote_summary: Default::default(),
            quote_summary_fetch: Arc::new(tokio::sync::Mutex::new(())),
            chart_cache: Default::default(),
            events_cache: Default::default(),
            recommendations_cache: Default::default(),
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

/// Ticker for fetching symbol-specific data.
///
/// Provides access to quotes, charts, financials, news, and other data for a specific symbol.
/// Uses smart lazy loading - quote data is fetched once and cached.
///
/// # Example
///
/// ```no_run
/// use finance_query::Ticker;
///
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let ticker = Ticker::new("AAPL").await?;
///
/// // Get quote data
/// let quote = ticker.quote().await?;
/// println!("Price: {:?}", quote.regular_market_price);
///
/// // Get chart data
/// use finance_query::{Interval, TimeRange};
/// let chart = ticker.chart(Interval::OneDay, TimeRange::OneMonth).await?;
/// println!("Candles: {}", chart.candles.len());
/// # Ok(())
/// # }
/// ```
#[derive(Clone)]
pub struct Ticker {
    symbol: Arc<str>,
    client: Arc<YahooClient>,
    cache_ttl: Option<Duration>,
    include_logo: bool,
    quote_summary: Cache<QuoteSummaryResponse>,
    quote_summary_fetch: Arc<tokio::sync::Mutex<()>>,
    chart_cache: MapCache<(Interval, TimeRange), Chart>,
    events_cache: Cache<ChartEvents>,
    recommendations_cache: Cache<RecommendationResponse>,
    news_cache: Cache<Vec<crate::models::news::News>>,
    options_cache: MapCache<Option<i64>, Options>,
    financials_cache: MapCache<
        (crate::constants::StatementType, crate::constants::Frequency),
        FinancialStatement,
    >,
    #[cfg(feature = "indicators")]
    indicators_cache: MapCache<(Interval, TimeRange), crate::indicators::IndicatorsSummary>,
    edgar_submissions_cache: Cache<crate::models::edgar::EdgarSubmissions>,
    edgar_facts_cache: Cache<crate::models::edgar::CompanyFacts>,
}

impl Ticker {
    /// Creates a new ticker with default configuration
    ///
    /// # Arguments
    ///
    /// * `symbol` - Stock symbol (e.g., "AAPL", "MSFT")
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use finance_query::Ticker;
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let ticker = Ticker::new("AAPL").await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn new(symbol: impl Into<String>) -> Result<Self> {
        Self::builder(symbol).build().await
    }

    /// Creates a new builder for Ticker
    ///
    /// Use this for custom configuration (language, region, timeout, proxy).
    ///
    /// # Arguments
    ///
    /// * `symbol` - Stock symbol (e.g., "AAPL", "MSFT")
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use finance_query::Ticker;
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// // Simple case with defaults (same as new())
    /// let ticker = Ticker::builder("AAPL").build().await?;
    ///
    /// // With custom configuration
    /// let ticker = Ticker::builder("AAPL")
    ///     .lang("ja-JP")
    ///     .region_code("JP")
    ///     .build()
    ///     .await?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn builder(symbol: impl Into<String>) -> TickerBuilder {
        TickerBuilder::new(symbol)
    }

    /// Returns the ticker symbol
    pub fn symbol(&self) -> &str {
        &self.symbol
    }

    /// Returns a shareable handle to this ticker's authenticated session.
    ///
    /// Pass the handle to other [`Ticker`] or [`Tickers`](crate::Tickers) builders
    /// via `.client()` to reuse the same session without re-authenticating.
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
    /// let msft = Ticker::builder("MSFT").client(handle).build().await?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn client_handle(&self) -> ClientHandle {
        ClientHandle(Arc::clone(&self.client))
    }

    /// Returns `true` if a cache entry exists and has not exceeded the TTL.
    ///
    /// Returns `false` when caching is disabled (`cache_ttl` is `None`).
    #[inline]
    fn is_cache_fresh<T>(&self, entry: Option<&CacheEntry<T>>) -> bool {
        CacheEntry::is_fresh_with_ttl(entry, self.cache_ttl)
    }

    /// Insert into a map cache, amortizing stale-entry eviction.
    ///
    /// Only sweeps stale entries when the map exceeds [`EVICTION_THRESHOLD`],
    /// avoiding O(n) scans on every write.
    #[inline]
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

    /// Helper to construct Recommendation from RecommendationResponse with limit
    fn build_recommendation_with_limit(
        &self,
        response: &RecommendationResponse,
        limit: u32,
    ) -> Recommendation {
        Recommendation {
            symbol: self.symbol.to_string(),
            recommendations: response
                .finance
                .result
                .iter()
                .flat_map(|r| &r.recommended_symbols)
                .take(limit as usize)
                .cloned()
                .collect(),
        }
    }

    /// Builds the quote summary URL with all modules.
    ///
    /// The module list is computed once and reused across all Ticker instances.
    fn build_quote_summary_url(&self) -> String {
        static MODULES_PARAM: OnceLock<String> = OnceLock::new();
        let modules = MODULES_PARAM.get_or_init(|| {
            Module::all()
                .iter()
                .map(|m| m.as_str())
                .collect::<Vec<_>>()
                .join(",")
        });
        let url = crate::endpoints::urls::api::quote_summary(&self.symbol);
        format!("{}?modules={}", url, modules)
    }

    /// Ensures quote summary is loaded and returns a read guard.
    ///
    /// Fast path: read lock only.
    /// Slow path: serialized fetch (mutex), HTTP I/O with no lock held, brief write lock update.
    async fn ensure_and_read_quote_summary(
        &self,
    ) -> Result<tokio::sync::RwLockReadGuard<'_, Option<CacheEntry<QuoteSummaryResponse>>>> {
        // Fast path: cache hit
        {
            let cache = self.quote_summary.read().await;
            if self.is_cache_fresh(cache.as_ref()) {
                return Ok(cache);
            }
        }

        // Slow path: serialize fetch operations to prevent duplicate requests
        let _fetch_guard = self.quote_summary_fetch.lock().await;

        // Double-check: another task may have fetched while we waited on mutex
        {
            let cache = self.quote_summary.read().await;
            if self.is_cache_fresh(cache.as_ref()) {
                return Ok(cache);
            }
        }

        // HTTP I/O with NO lock held — critical for concurrent readers
        let url = self.build_quote_summary_url();
        let http_response = self.client.request_with_crumb(&url).await?;
        let json = http_response.json::<serde_json::Value>().await?;
        let response = QuoteSummaryResponse::from_json(json, &self.symbol)?;

        // Brief write lock to update cache
        {
            let mut cache = self.quote_summary.write().await;
            *cache = Some(CacheEntry::new(response));
        }

        // Fetch mutex released automatically, return read guard
        Ok(self.quote_summary.read().await)
    }
}

// Generate quote summary accessor methods using macro to eliminate duplication.
macros::define_quote_accessors! {
    /// Get price information
    price -> Price, price,

    /// Get summary detail
    summary_detail -> SummaryDetail, summary_detail,

    /// Get financial data
    financial_data -> FinancialData, financial_data,

    /// Get key statistics
    key_stats -> DefaultKeyStatistics, default_key_statistics,

    /// Get asset profile
    asset_profile -> AssetProfile, asset_profile,

    /// Get calendar events
    calendar_events -> CalendarEvents, calendar_events,

    /// Get earnings
    earnings -> Earnings, earnings,

    /// Get earnings trend
    earnings_trend -> EarningsTrend, earnings_trend,

    /// Get earnings history
    earnings_history -> EarningsHistory, earnings_history,

    /// Get recommendation trend
    recommendation_trend -> RecommendationTrend, recommendation_trend,

    /// Get insider holders
    insider_holders -> InsiderHolders, insider_holders,

    /// Get insider transactions
    insider_transactions -> InsiderTransactions, insider_transactions,

    /// Get institution ownership
    institution_ownership -> InstitutionOwnership, institution_ownership,

    /// Get fund ownership
    fund_ownership -> FundOwnership, fund_ownership,

    /// Get major holders breakdown
    major_holders -> MajorHoldersBreakdown, major_holders_breakdown,

    /// Get net share purchase activity
    share_purchase_activity -> NetSharePurchaseActivity, net_share_purchase_activity,

    /// Get quote type
    quote_type -> QuoteTypeData, quote_type,

    /// Get summary profile
    summary_profile -> SummaryProfile, summary_profile,

    /// Get SEC filings (limited Yahoo Finance data)
    ///
    /// **DEPRECATED:** This method returns limited SEC filing metadata from Yahoo Finance.
    /// For comprehensive filing data directly from SEC EDGAR, use `edgar_submissions()` instead.
    ///
    /// To use EDGAR methods:
    /// ```no_run
    /// # use finance_query::{Ticker, edgar};
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// edgar::init("user@example.com")?;
    /// let ticker = Ticker::new("AAPL").await?;
    /// let submissions = ticker.edgar_submissions().await?;  // Comprehensive EDGAR data
    /// # Ok(())
    /// # }
    /// ```
    #[deprecated(
        since = "2.2.0",
        note = "Use `edgar_submissions()` for comprehensive SEC EDGAR data instead of limited Yahoo Finance metadata"
    )]
    sec_filings -> SecFilings, sec_filings,

    /// Get upgrade/downgrade history
    grading_history -> UpgradeDowngradeHistory, upgrade_downgrade_history,

    /// Get fund performance data (returns, trailing returns, risk statistics)
    ///
    /// Primarily relevant for ETFs and mutual funds. Returns `None` for equities.
    fund_performance -> FundPerformance, fund_performance,

    /// Get fund profile (category, family, fees, legal type)
    ///
    /// Primarily relevant for ETFs and mutual funds. Returns `None` for equities.
    fund_profile -> FundProfile, fund_profile,

    /// Get top holdings for ETFs and mutual funds
    ///
    /// Includes top stock/bond holdings with weights and sector weightings.
    /// Returns `None` for equities.
    top_holdings -> TopHoldings, top_holdings,

    /// Get index trend data (P/E estimates and growth rates)
    ///
    /// Contains trend data for the symbol's associated index.
    index_trend -> IndexTrend, index_trend,

    /// Get industry trend data
    ///
    /// Contains P/E and growth estimates for the symbol's industry.
    industry_trend -> IndustryTrend, industry_trend,

    /// Get sector trend data
    ///
    /// Contains P/E and growth estimates for the symbol's sector.
    sector_trend -> SectorTrend, sector_trend,

    /// Get equity performance vs benchmark
    ///
    /// Performance comparison across multiple time periods.
    equity_performance -> EquityPerformance, equity_performance,
}

impl Ticker {
    /// Get full quote data, optionally including logo URLs.
    ///
    /// Use [`TickerBuilder::logo()`](TickerBuilder::logo) to enable logo fetching
    /// for this ticker instance.
    ///
    /// When logos are enabled, fetches both quote summary and logo URL in parallel
    /// using tokio::join! for minimal latency impact (~0-100ms overhead).
    pub async fn quote(&self) -> Result<Quote> {
        let not_found = || crate::error::FinanceError::SymbolNotFound {
            symbol: Some(self.symbol.to_string()),
            context: "Quote summary not loaded".to_string(),
        };

        if self.include_logo {
            // Ensure quote summary is loaded in background while we fetch logos
            let (cache_result, logo_result) = tokio::join!(
                self.ensure_and_read_quote_summary(),
                self.client.get_logo_url(&self.symbol)
            );
            let cache = cache_result?;
            let entry = cache.as_ref().ok_or_else(not_found)?;
            let (logo_url, company_logo_url) = logo_result;
            Ok(Quote::from_response(
                &entry.value,
                logo_url,
                company_logo_url,
            ))
        } else {
            let cache = self.ensure_and_read_quote_summary().await?;
            let entry = cache.as_ref().ok_or_else(not_found)?;
            Ok(Quote::from_response(&entry.value, None, None))
        }
    }

    /// Get historical chart data
    pub async fn chart(&self, interval: Interval, range: TimeRange) -> Result<Chart> {
        // Fast path: return cached Chart directly (no re-parsing)
        {
            let cache = self.chart_cache.read().await;
            if let Some(entry) = cache.get(&(interval, range))
                && self.is_cache_fresh(Some(entry))
            {
                return Ok(entry.value.clone());
            }
        }

        // Fetch from Yahoo
        let json = self.client.get_chart(&self.symbol, interval, range).await?;
        let chart_result = Self::parse_chart_result(json, &self.symbol)?;

        // Always update events when we have fresh data from Yahoo
        if let Some(events) = &chart_result.events {
            let mut events_cache = self.events_cache.write().await;
            *events_cache = Some(CacheEntry::new(events.clone()));
        }

        // Materialize Chart from raw result — this is the only place to_candles() runs
        let chart = Chart {
            symbol: self.symbol.to_string(),
            meta: chart_result.meta.clone(),
            candles: chart_result.to_candles(),
            interval: Some(interval),
            range: Some(range),
        };

        // Only clone when caching is enabled to avoid unnecessary allocations
        if self.cache_ttl.is_some() {
            let ret = chart.clone();
            let mut cache = self.chart_cache.write().await;
            self.cache_insert(&mut cache, (interval, range), chart);
            Ok(ret)
        } else {
            Ok(chart)
        }
    }

    /// Parse a ChartResult from raw JSON, returning a descriptive error on failure.
    fn parse_chart_result(
        json: serde_json::Value,
        symbol: &str,
    ) -> Result<crate::models::chart::result::ChartResult> {
        let response = ChartResponse::from_json(json).map_err(|e| {
            crate::error::FinanceError::ResponseStructureError {
                field: "chart".to_string(),
                context: e.to_string(),
            }
        })?;

        let results =
            response
                .chart
                .result
                .ok_or_else(|| crate::error::FinanceError::SymbolNotFound {
                    symbol: Some(symbol.to_string()),
                    context: "Chart data not found".to_string(),
                })?;

        results
            .into_iter()
            .next()
            .ok_or_else(|| crate::error::FinanceError::SymbolNotFound {
                symbol: Some(symbol.to_string()),
                context: "Chart data empty".to_string(),
            })
    }

    /// Get historical chart data for a custom date range.
    ///
    /// Unlike [`chart()`](Self::chart) which uses predefined time ranges,
    /// this method accepts absolute start/end timestamps for precise date control.
    ///
    /// Results are **not cached** since custom ranges have unbounded key space.
    ///
    /// # Arguments
    ///
    /// * `interval` - Time interval between data points
    /// * `start` - Start date as Unix timestamp (seconds since epoch)
    /// * `end` - End date as Unix timestamp (seconds since epoch)
    ///
    /// # Example
    ///
    /// ```no_run
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// use finance_query::{Ticker, Interval};
    /// use chrono::NaiveDate;
    ///
    /// let ticker = Ticker::new("AAPL").await?;
    ///
    /// // Q3 2024
    /// let start = NaiveDate::from_ymd_opt(2024, 7, 1).unwrap()
    ///     .and_hms_opt(0, 0, 0).unwrap().and_utc().timestamp();
    /// let end = NaiveDate::from_ymd_opt(2024, 9, 30).unwrap()
    ///     .and_hms_opt(23, 59, 59).unwrap().and_utc().timestamp();
    ///
    /// let chart = ticker.chart_range(Interval::OneDay, start, end).await?;
    /// println!("Q3 2024 candles: {}", chart.candles.len());
    /// # Ok(())
    /// # }
    /// ```
    pub async fn chart_range(&self, interval: Interval, start: i64, end: i64) -> Result<Chart> {
        let json = self
            .client
            .get_chart_range(&self.symbol, interval, start, end)
            .await?;
        let chart_result = Self::parse_chart_result(json, &self.symbol)?;

        // Always update events when we have fresh data from Yahoo
        if let Some(events) = &chart_result.events {
            let mut events_cache = self.events_cache.write().await;
            *events_cache = Some(CacheEntry::new(events.clone()));
        }

        Ok(Chart {
            symbol: self.symbol.to_string(),
            meta: chart_result.meta.clone(),
            candles: chart_result.to_candles(),
            interval: Some(interval),
            range: None,
        })
    }

    /// Ensures events data is loaded (fetches events only if not cached)
    async fn ensure_events_loaded(&self) -> Result<()> {
        // Quick read check
        {
            let cache = self.events_cache.read().await;
            if self.is_cache_fresh(cache.as_ref()) {
                return Ok(());
            }
        }

        // Fetch events using max range with 1d interval to get all historical events
        // Using 1d interval minimizes candle count compared to shorter intervals
        let json = crate::endpoints::chart::fetch(
            &self.client,
            &self.symbol,
            Interval::OneDay,
            TimeRange::Max,
        )
        .await?;
        let chart_result = Self::parse_chart_result(json, &self.symbol)?;

        // Write to events cache unconditionally for temporary storage during this method
        // Note: when cache_ttl is None, is_cache_fresh() returns false, so this will
        // be refetched on the next call to dividends()/splits()/capital_gains().
        // Cache empty ChartEvents when Yahoo returns no events to prevent infinite refetch loops
        let mut events_cache = self.events_cache.write().await;
        *events_cache = Some(CacheEntry::new(chart_result.events.unwrap_or_default()));

        Ok(())
    }

    /// Get dividend history
    ///
    /// Returns historical dividend payments sorted by date.
    /// Events are lazily loaded (fetched once, then filtered by range).
    ///
    /// # Arguments
    ///
    /// * `range` - Time range to filter dividends
    ///
    /// # Example
    ///
    /// ```no_run
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// use finance_query::{Ticker, TimeRange};
    ///
    /// let ticker = Ticker::new("AAPL").await?;
    ///
    /// // Get all dividends
    /// let all = ticker.dividends(TimeRange::Max).await?;
    ///
    /// // Get last year's dividends
    /// let recent = ticker.dividends(TimeRange::OneYear).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn dividends(&self, range: TimeRange) -> Result<Vec<Dividend>> {
        self.ensure_events_loaded().await?;

        let cache = self.events_cache.read().await;
        let all = cache
            .as_ref()
            .map(|e| e.value.to_dividends())
            .unwrap_or_default();

        Ok(filter_by_range(all, range))
    }

    /// Compute dividend analytics for the requested time range.
    ///
    /// Calculates statistics on the dividend history: total paid, payment count,
    /// average payment, and Compound Annual Growth Rate (CAGR).
    ///
    /// **CAGR note:** requires at least two payments spanning at least one calendar year.
    ///
    /// # Arguments
    ///
    /// * `range` - Time range to analyse
    ///
    /// # Example
    ///
    /// ```no_run
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// use finance_query::{Ticker, TimeRange};
    ///
    /// let ticker = Ticker::new("AAPL").await?;
    /// let analytics = ticker.dividend_analytics(TimeRange::FiveYears).await?;
    ///
    /// println!("Total paid: ${:.2}", analytics.total_paid);
    /// println!("Payments:   {}", analytics.payment_count);
    /// if let Some(cagr) = analytics.cagr {
    ///     println!("CAGR:       {:.1}%", cagr * 100.0);
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub async fn dividend_analytics(
        &self,
        range: TimeRange,
    ) -> Result<crate::models::chart::DividendAnalytics> {
        let dividends = self.dividends(range).await?;
        Ok(crate::models::chart::DividendAnalytics::from_dividends(
            &dividends,
        ))
    }

    /// Get stock split history
    ///
    /// Returns historical stock splits sorted by date.
    /// Events are lazily loaded (fetched once, then filtered by range).
    ///
    /// # Arguments
    ///
    /// * `range` - Time range to filter splits
    ///
    /// # Example
    ///
    /// ```no_run
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// use finance_query::{Ticker, TimeRange};
    ///
    /// let ticker = Ticker::new("NVDA").await?;
    ///
    /// // Get all splits
    /// let all = ticker.splits(TimeRange::Max).await?;
    ///
    /// // Get last 5 years
    /// let recent = ticker.splits(TimeRange::FiveYears).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn splits(&self, range: TimeRange) -> Result<Vec<Split>> {
        self.ensure_events_loaded().await?;

        let cache = self.events_cache.read().await;
        let all = cache
            .as_ref()
            .map(|e| e.value.to_splits())
            .unwrap_or_default();

        Ok(filter_by_range(all, range))
    }

    /// Get capital gains distribution history
    ///
    /// Returns historical capital gain distributions sorted by date.
    /// This is primarily relevant for mutual funds and ETFs.
    /// Events are lazily loaded (fetched once, then filtered by range).
    ///
    /// # Arguments
    ///
    /// * `range` - Time range to filter capital gains
    ///
    /// # Example
    ///
    /// ```no_run
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// use finance_query::{Ticker, TimeRange};
    ///
    /// let ticker = Ticker::new("VFIAX").await?;
    ///
    /// // Get all capital gains
    /// let all = ticker.capital_gains(TimeRange::Max).await?;
    ///
    /// // Get last 2 years
    /// let recent = ticker.capital_gains(TimeRange::TwoYears).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn capital_gains(&self, range: TimeRange) -> Result<Vec<CapitalGain>> {
        self.ensure_events_loaded().await?;

        let cache = self.events_cache.read().await;
        let all = cache
            .as_ref()
            .map(|e| e.value.to_capital_gains())
            .unwrap_or_default();

        Ok(filter_by_range(all, range))
    }

    /// Calculate all technical indicators from chart data
    ///
    /// # Arguments
    ///
    /// * `interval` - The time interval for each candle
    /// * `range` - The time range to fetch data for
    ///
    /// # Returns
    ///
    /// Returns `IndicatorsSummary` containing all calculated indicators.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use finance_query::{Ticker, Interval, TimeRange};
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let ticker = Ticker::new("AAPL").await?;
    /// let indicators = ticker.indicators(Interval::OneDay, TimeRange::OneYear).await?;
    ///
    /// println!("RSI(14): {:?}", indicators.rsi_14);
    /// println!("MACD: {:?}", indicators.macd);
    /// # Ok(())
    /// # }
    /// ```
    #[cfg(feature = "indicators")]
    pub async fn indicators(
        &self,
        interval: Interval,
        range: TimeRange,
    ) -> Result<crate::indicators::IndicatorsSummary> {
        // Check cache first (read lock)
        {
            let cache = self.indicators_cache.read().await;
            if let Some(entry) = cache.get(&(interval, range))
                && self.is_cache_fresh(Some(entry))
            {
                return Ok(entry.value.clone());
            }
        }

        // Fetch chart data (this is also cached!)
        let chart = self.chart(interval, range).await?;

        // Calculate indicators from candles
        let indicators = crate::indicators::summary::calculate_indicators(&chart.candles);

        // Only clone when caching is enabled
        if self.cache_ttl.is_some() {
            let mut cache = self.indicators_cache.write().await;
            self.cache_insert(&mut cache, (interval, range), indicators.clone());
            Ok(indicators)
        } else {
            Ok(indicators)
        }
    }

    /// Calculate a specific technical indicator over a time range.
    ///
    /// Returns the full time series for the requested indicator, not just the latest value.
    /// This is useful when you need historical indicator values for analysis or charting.
    ///
    /// # Arguments
    ///
    /// * `indicator` - The indicator to calculate (from `crate::indicators::Indicator`)
    /// * `interval` - Time interval for candles (1d, 1h, etc.)
    /// * `range` - Time range for historical data
    ///
    /// # Returns
    ///
    /// An `IndicatorResult` containing the full time series. Access the data using match:
    /// - `IndicatorResult::Series(values)` - for simple indicators (SMA, EMA, RSI, ATR, OBV, VWAP, WMA)
    /// - `IndicatorResult::Macd(data)` - for MACD (macd_line, signal_line, histogram)
    /// - `IndicatorResult::Bollinger(data)` - for Bollinger Bands (upper, middle, lower)
    ///
    /// # Example
    ///
    /// ```no_run
    /// use finance_query::{Ticker, Interval, TimeRange};
    /// use finance_query::indicators::{Indicator, IndicatorResult};
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let ticker = Ticker::new("AAPL").await?;
    ///
    /// // Calculate 14-period RSI
    /// let result = ticker.indicator(
    ///     Indicator::Rsi(14),
    ///     Interval::OneDay,
    ///     TimeRange::ThreeMonths
    /// ).await?;
    ///
    /// match result {
    ///     IndicatorResult::Series(values) => {
    ///         println!("Latest RSI: {:?}", values.last());
    ///     }
    ///     _ => {}
    /// }
    ///
    /// // Calculate MACD
    /// let macd_result = ticker.indicator(
    ///     Indicator::Macd { fast: 12, slow: 26, signal: 9 },
    ///     Interval::OneDay,
    ///     TimeRange::SixMonths
    /// ).await?;
    ///
    /// # Ok(())
    /// # }
    /// ```
    #[cfg(feature = "indicators")]
    pub async fn indicator(
        &self,
        indicator: crate::indicators::Indicator,
        interval: Interval,
        range: TimeRange,
    ) -> Result<crate::indicators::IndicatorResult> {
        use crate::indicators::{Indicator, IndicatorResult};

        // Fetch chart data
        let chart = self.chart(interval, range).await?;

        // Calculate the requested indicator
        // Note: Price vectors are extracted lazily within each arm to avoid waste
        let result = match indicator {
            Indicator::Sma(period) => IndicatorResult::Series(chart.sma(period)),
            Indicator::Ema(period) => IndicatorResult::Series(chart.ema(period)),
            Indicator::Rsi(period) => IndicatorResult::Series(chart.rsi(period)?),
            Indicator::Macd { fast, slow, signal } => {
                IndicatorResult::Macd(chart.macd(fast, slow, signal)?)
            }
            Indicator::Bollinger { period, std_dev } => {
                IndicatorResult::Bollinger(chart.bollinger_bands(period, std_dev)?)
            }
            Indicator::Atr(period) => IndicatorResult::Series(chart.atr(period)?),
            Indicator::Obv => {
                let closes = chart.close_prices();
                let volumes = chart.volumes();
                IndicatorResult::Series(crate::indicators::obv(&closes, &volumes)?)
            }
            Indicator::Vwap => {
                let highs = chart.high_prices();
                let lows = chart.low_prices();
                let closes = chart.close_prices();
                let volumes = chart.volumes();
                IndicatorResult::Series(crate::indicators::vwap(&highs, &lows, &closes, &volumes)?)
            }
            Indicator::Wma(period) => {
                let closes = chart.close_prices();
                IndicatorResult::Series(crate::indicators::wma(&closes, period)?)
            }
            Indicator::Dema(period) => {
                let closes = chart.close_prices();
                IndicatorResult::Series(crate::indicators::dema(&closes, period)?)
            }
            Indicator::Tema(period) => {
                let closes = chart.close_prices();
                IndicatorResult::Series(crate::indicators::tema(&closes, period)?)
            }
            Indicator::Hma(period) => {
                let closes = chart.close_prices();
                IndicatorResult::Series(crate::indicators::hma(&closes, period)?)
            }
            Indicator::Vwma(period) => {
                let closes = chart.close_prices();
                let volumes = chart.volumes();
                IndicatorResult::Series(crate::indicators::vwma(&closes, &volumes, period)?)
            }
            Indicator::Alma {
                period,
                offset,
                sigma,
            } => {
                let closes = chart.close_prices();
                IndicatorResult::Series(crate::indicators::alma(&closes, period, offset, sigma)?)
            }
            Indicator::McginleyDynamic(period) => {
                let closes = chart.close_prices();
                IndicatorResult::Series(crate::indicators::mcginley_dynamic(&closes, period)?)
            }
            Indicator::Stochastic {
                k_period,
                k_slow,
                d_period,
            } => {
                let highs = chart.high_prices();
                let lows = chart.low_prices();
                let closes = chart.close_prices();
                IndicatorResult::Stochastic(crate::indicators::stochastic(
                    &highs, &lows, &closes, k_period, k_slow, d_period,
                )?)
            }
            Indicator::StochasticRsi {
                rsi_period,
                stoch_period,
                k_period,
                d_period,
            } => {
                let closes = chart.close_prices();
                IndicatorResult::Stochastic(crate::indicators::stochastic_rsi(
                    &closes,
                    rsi_period,
                    stoch_period,
                    k_period,
                    d_period,
                )?)
            }
            Indicator::Cci(period) => {
                let highs = chart.high_prices();
                let lows = chart.low_prices();
                let closes = chart.close_prices();
                IndicatorResult::Series(crate::indicators::cci(&highs, &lows, &closes, period)?)
            }
            Indicator::WilliamsR(period) => {
                let highs = chart.high_prices();
                let lows = chart.low_prices();
                let closes = chart.close_prices();
                IndicatorResult::Series(crate::indicators::williams_r(
                    &highs, &lows, &closes, period,
                )?)
            }
            Indicator::Roc(period) => {
                let closes = chart.close_prices();
                IndicatorResult::Series(crate::indicators::roc(&closes, period)?)
            }
            Indicator::Momentum(period) => {
                let closes = chart.close_prices();
                IndicatorResult::Series(crate::indicators::momentum(&closes, period)?)
            }
            Indicator::Cmo(period) => {
                let closes = chart.close_prices();
                IndicatorResult::Series(crate::indicators::cmo(&closes, period)?)
            }
            Indicator::AwesomeOscillator { fast, slow } => {
                let highs = chart.high_prices();
                let lows = chart.low_prices();
                IndicatorResult::Series(crate::indicators::awesome_oscillator(
                    &highs, &lows, fast, slow,
                )?)
            }
            Indicator::CoppockCurve {
                wma_period,
                long_roc,
                short_roc,
            } => {
                let closes = chart.close_prices();
                IndicatorResult::Series(crate::indicators::coppock_curve(
                    &closes, long_roc, short_roc, wma_period,
                )?)
            }
            Indicator::Adx(period) => {
                let highs = chart.high_prices();
                let lows = chart.low_prices();
                let closes = chart.close_prices();
                IndicatorResult::Series(crate::indicators::adx(&highs, &lows, &closes, period)?)
            }
            Indicator::Aroon(period) => {
                let highs = chart.high_prices();
                let lows = chart.low_prices();
                IndicatorResult::Aroon(crate::indicators::aroon(&highs, &lows, period)?)
            }
            Indicator::Supertrend { period, multiplier } => {
                let highs = chart.high_prices();
                let lows = chart.low_prices();
                let closes = chart.close_prices();
                IndicatorResult::SuperTrend(crate::indicators::supertrend(
                    &highs, &lows, &closes, period, multiplier,
                )?)
            }
            Indicator::Ichimoku {
                conversion,
                base,
                lagging,
                displacement,
            } => {
                let highs = chart.high_prices();
                let lows = chart.low_prices();
                let closes = chart.close_prices();
                IndicatorResult::Ichimoku(crate::indicators::ichimoku(
                    &highs,
                    &lows,
                    &closes,
                    conversion,
                    base,
                    lagging,
                    displacement,
                )?)
            }
            Indicator::ParabolicSar { step, max } => {
                let highs = chart.high_prices();
                let lows = chart.low_prices();
                let closes = chart.close_prices();
                IndicatorResult::Series(crate::indicators::parabolic_sar(
                    &highs, &lows, &closes, step, max,
                )?)
            }
            Indicator::BullBearPower(period) => {
                let highs = chart.high_prices();
                let lows = chart.low_prices();
                let closes = chart.close_prices();
                IndicatorResult::BullBearPower(crate::indicators::bull_bear_power(
                    &highs, &lows, &closes, period,
                )?)
            }
            Indicator::ElderRay(period) => {
                let highs = chart.high_prices();
                let lows = chart.low_prices();
                let closes = chart.close_prices();
                IndicatorResult::ElderRay(crate::indicators::elder_ray(
                    &highs, &lows, &closes, period,
                )?)
            }
            Indicator::KeltnerChannels {
                period,
                multiplier,
                atr_period,
            } => {
                let highs = chart.high_prices();
                let lows = chart.low_prices();
                let closes = chart.close_prices();
                IndicatorResult::Keltner(crate::indicators::keltner_channels(
                    &highs, &lows, &closes, period, atr_period, multiplier,
                )?)
            }
            Indicator::DonchianChannels(period) => {
                let highs = chart.high_prices();
                let lows = chart.low_prices();
                IndicatorResult::Donchian(crate::indicators::donchian_channels(
                    &highs, &lows, period,
                )?)
            }
            Indicator::TrueRange => {
                let highs = chart.high_prices();
                let lows = chart.low_prices();
                let closes = chart.close_prices();
                IndicatorResult::Series(crate::indicators::true_range(&highs, &lows, &closes)?)
            }
            Indicator::ChoppinessIndex(period) => {
                let highs = chart.high_prices();
                let lows = chart.low_prices();
                let closes = chart.close_prices();
                IndicatorResult::Series(crate::indicators::choppiness_index(
                    &highs, &lows, &closes, period,
                )?)
            }
            Indicator::Mfi(period) => {
                let highs = chart.high_prices();
                let lows = chart.low_prices();
                let closes = chart.close_prices();
                let volumes = chart.volumes();
                IndicatorResult::Series(crate::indicators::mfi(
                    &highs, &lows, &closes, &volumes, period,
                )?)
            }
            Indicator::Cmf(period) => {
                let highs = chart.high_prices();
                let lows = chart.low_prices();
                let closes = chart.close_prices();
                let volumes = chart.volumes();
                IndicatorResult::Series(crate::indicators::cmf(
                    &highs, &lows, &closes, &volumes, period,
                )?)
            }
            Indicator::ChaikinOscillator => {
                let highs = chart.high_prices();
                let lows = chart.low_prices();
                let closes = chart.close_prices();
                let volumes = chart.volumes();
                IndicatorResult::Series(crate::indicators::chaikin_oscillator(
                    &highs, &lows, &closes, &volumes,
                )?)
            }
            Indicator::AccumulationDistribution => {
                let highs = chart.high_prices();
                let lows = chart.low_prices();
                let closes = chart.close_prices();
                let volumes = chart.volumes();
                IndicatorResult::Series(crate::indicators::accumulation_distribution(
                    &highs, &lows, &closes, &volumes,
                )?)
            }
            Indicator::BalanceOfPower(period) => {
                let opens = chart.open_prices();
                let highs = chart.high_prices();
                let lows = chart.low_prices();
                let closes = chart.close_prices();
                IndicatorResult::Series(crate::indicators::balance_of_power(
                    &opens, &highs, &lows, &closes, period,
                )?)
            }
        };

        Ok(result)
    }

    /// Get analyst recommendations
    pub async fn recommendations(&self, limit: u32) -> Result<Recommendation> {
        // Check cache (always fetches max from server, truncated to limit on return)
        {
            let cache = self.recommendations_cache.read().await;
            if let Some(entry) = cache.as_ref()
                && self.is_cache_fresh(Some(entry))
            {
                return Ok(self.build_recommendation_with_limit(&entry.value, limit));
            }
        }

        // Always fetch server maximum (no limit restriction to maximize cache utility)
        let json = self.client.get_recommendations(&self.symbol, 15).await?;
        let response = RecommendationResponse::from_json(json).map_err(|e| {
            crate::error::FinanceError::ResponseStructureError {
                field: "finance".to_string(),
                context: e.to_string(),
            }
        })?;

        // Cache full response, return truncated result
        if self.cache_ttl.is_some() {
            let mut cache = self.recommendations_cache.write().await;
            *cache = Some(CacheEntry::new(response));
            let entry = cache.as_ref().unwrap();
            return Ok(self.build_recommendation_with_limit(&entry.value, limit));
        }

        Ok(self.build_recommendation_with_limit(&response, limit))
    }

    /// Get financial statements
    ///
    /// # Arguments
    ///
    /// * `statement_type` - Type of statement (Income, Balance, CashFlow)
    /// * `frequency` - Annual or Quarterly
    ///
    /// # Example
    ///
    /// ```no_run
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// use finance_query::{Ticker, Frequency, StatementType};
    ///
    /// let ticker = Ticker::new("AAPL").await?;
    /// let income = ticker.financials(StatementType::Income, Frequency::Annual).await?;
    /// println!("Revenue: {:?}", income.statement.get("TotalRevenue"));
    /// # Ok(())
    /// # }
    /// ```
    pub async fn financials(
        &self,
        statement_type: crate::constants::StatementType,
        frequency: crate::constants::Frequency,
    ) -> Result<FinancialStatement> {
        let cache_key = (statement_type, frequency);

        // Check cache
        {
            let cache = self.financials_cache.read().await;
            if let Some(entry) = cache.get(&cache_key)
                && self.is_cache_fresh(Some(entry))
            {
                return Ok(entry.value.clone());
            }
        }

        // Fetch financials
        let financials = self
            .client
            .get_financials(&self.symbol, statement_type, frequency)
            .await?;

        // Only clone when caching is enabled
        if self.cache_ttl.is_some() {
            let mut cache = self.financials_cache.write().await;
            self.cache_insert(&mut cache, cache_key, financials.clone());
            Ok(financials)
        } else {
            Ok(financials)
        }
    }

    /// Get news articles for this symbol
    ///
    /// # Example
    ///
    /// ```no_run
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// use finance_query::Ticker;
    ///
    /// let ticker = Ticker::new("AAPL").await?;
    /// let news = ticker.news().await?;
    /// for article in news {
    ///     println!("{}: {}", article.source, article.title);
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub async fn news(&self) -> Result<Vec<crate::models::news::News>> {
        // Check cache
        {
            let cache = self.news_cache.read().await;
            if self.is_cache_fresh(cache.as_ref()) {
                return Ok(cache.as_ref().unwrap().value.clone());
            }
        }

        // Fetch news
        let news = crate::scrapers::stockanalysis::scrape_symbol_news(&self.symbol).await?;

        // Only clone when caching is enabled
        if self.cache_ttl.is_some() {
            let mut cache = self.news_cache.write().await;
            *cache = Some(CacheEntry::new(news.clone()));
            Ok(news)
        } else {
            Ok(news)
        }
    }

    /// Get options chain
    pub async fn options(&self, date: Option<i64>) -> Result<Options> {
        // Check cache
        {
            let cache = self.options_cache.read().await;
            if let Some(entry) = cache.get(&date)
                && self.is_cache_fresh(Some(entry))
            {
                return Ok(entry.value.clone());
            }
        }

        // Fetch options
        let json = self.client.get_options(&self.symbol, date).await?;
        let options: Options = serde_json::from_value(json).map_err(|e| {
            crate::error::FinanceError::ResponseStructureError {
                field: "options".to_string(),
                context: e.to_string(),
            }
        })?;

        // Only clone when caching is enabled
        if self.cache_ttl.is_some() {
            let mut cache = self.options_cache.write().await;
            self.cache_insert(&mut cache, date, options.clone());
            Ok(options)
        } else {
            Ok(options)
        }
    }

    /// Run a backtest with the given strategy and configuration.
    ///
    /// # Arguments
    ///
    /// * `strategy` - Trading strategy implementing the Strategy trait
    /// * `interval` - Candle interval (1d, 1h, etc.)
    /// * `range` - Time range for historical data
    /// * `config` - Backtest configuration (optional, uses defaults if None)
    ///
    /// # Example
    ///
    /// ```no_run
    /// use finance_query::{Ticker, Interval, TimeRange};
    /// use finance_query::backtesting::{SmaCrossover, BacktestConfig};
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let ticker = Ticker::new("AAPL").await?;
    ///
    /// // Simple backtest with defaults
    /// let strategy = SmaCrossover::new(10, 20);
    /// let result = ticker.backtest(
    ///     strategy,
    ///     Interval::OneDay,
    ///     TimeRange::OneYear,
    ///     None,
    /// ).await?;
    ///
    /// println!("{}", result.summary());
    /// println!("Total trades: {}", result.trades.len());
    ///
    /// // With custom config
    /// let config = BacktestConfig::builder()
    ///     .initial_capital(50_000.0)
    ///     .commission_pct(0.001)
    ///     .stop_loss_pct(0.05)
    ///     .allow_short(true)
    ///     .build()?;
    ///
    /// let result = ticker.backtest(
    ///     SmaCrossover::new(5, 20),
    ///     Interval::OneDay,
    ///     TimeRange::TwoYears,
    ///     Some(config),
    /// ).await?;
    /// # Ok(())
    /// # }
    /// ```
    #[cfg(feature = "backtesting")]
    pub async fn backtest<S: crate::backtesting::Strategy>(
        &self,
        strategy: S,
        interval: Interval,
        range: TimeRange,
        config: Option<crate::backtesting::BacktestConfig>,
    ) -> crate::backtesting::Result<crate::backtesting::BacktestResult> {
        use crate::backtesting::BacktestEngine;

        let config = config.unwrap_or_default();
        config.validate()?;

        // Fetch chart data — also populates the events cache used by dividends()
        let chart = self
            .chart(interval, range)
            .await
            .map_err(|e| crate::backtesting::BacktestError::ChartError(e.to_string()))?;

        // Fetch dividends from the events cache (no extra network request after chart())
        let dividends = self.dividends(range).await.unwrap_or_default();

        // Run backtest engine with dividend data
        let engine = BacktestEngine::new(config);
        engine.run_with_dividends(&self.symbol, &chart.candles, strategy, &dividends)
    }

    /// Run a backtest and compare performance against a benchmark symbol.
    ///
    /// Fetches both the symbol chart and the benchmark chart concurrently, then
    /// runs the backtest and populates [`BacktestResult::benchmark`] with
    /// comparison metrics (alpha, beta, information ratio, buy-and-hold return).
    ///
    /// Requires the **`backtesting`** feature flag.
    ///
    /// # Arguments
    ///
    /// * `strategy` - The strategy to backtest
    /// * `interval` - Candle interval
    /// * `range` - Historical range
    /// * `config` - Optional backtest configuration (uses defaults if `None`)
    /// * `benchmark` - Symbol to use as benchmark (e.g. `"SPY"`)
    #[cfg(feature = "backtesting")]
    pub async fn backtest_with_benchmark<S: crate::backtesting::Strategy>(
        &self,
        strategy: S,
        interval: Interval,
        range: TimeRange,
        config: Option<crate::backtesting::BacktestConfig>,
        benchmark: &str,
    ) -> crate::backtesting::Result<crate::backtesting::BacktestResult> {
        use crate::backtesting::BacktestEngine;

        let config = config.unwrap_or_default();
        config.validate()?;

        // Fetch the symbol chart and benchmark chart concurrently
        let benchmark_ticker = crate::Ticker::new(benchmark)
            .await
            .map_err(|e| crate::backtesting::BacktestError::ChartError(e.to_string()))?;

        let (chart, bench_chart) = tokio::try_join!(
            self.chart(interval, range),
            benchmark_ticker.chart(interval, range),
        )
        .map_err(|e| crate::backtesting::BacktestError::ChartError(e.to_string()))?;

        // Fetch dividends from events cache (no extra network request after chart())
        let dividends = self.dividends(range).await.unwrap_or_default();

        let engine = BacktestEngine::new(config);
        engine.run_with_benchmark(
            &self.symbol,
            &chart.candles,
            strategy,
            &dividends,
            benchmark,
            &bench_chart.candles,
        )
    }

    // ========================================================================
    // Risk Analytics
    // ========================================================================

    /// Compute a risk summary for this symbol.
    ///
    /// Requires the **`risk`** feature flag.
    ///
    /// Calculates Value at Risk, Sharpe/Sortino/Calmar ratios, and maximum drawdown
    /// from close-to-close returns derived from the requested chart data.
    ///
    /// # Arguments
    ///
    /// * `interval` - Candle interval (use `Interval::OneDay` for daily risk metrics)
    /// * `range` - Historical range to analyse
    /// * `benchmark` - Optional symbol to use as the benchmark for beta calculation
    ///
    /// # Example
    ///
    /// ```no_run
    /// use finance_query::{Ticker, Interval, TimeRange};
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let ticker = Ticker::new("AAPL").await?;
    ///
    /// // Risk vs no benchmark
    /// let summary = ticker.risk(Interval::OneDay, TimeRange::OneYear, None).await?;
    /// println!("VaR 95%:      {:.2}%", summary.var_95 * 100.0);
    /// println!("Max drawdown: {:.2}%", summary.max_drawdown * 100.0);
    ///
    /// // Risk with S&P 500 as benchmark
    /// let summary = ticker.risk(Interval::OneDay, TimeRange::OneYear, Some("^GSPC")).await?;
    /// println!("Beta: {:?}", summary.beta);
    /// # Ok(())
    /// # }
    /// ```
    #[cfg(feature = "risk")]
    pub async fn risk(
        &self,
        interval: Interval,
        range: TimeRange,
        benchmark: Option<&str>,
    ) -> Result<crate::risk::RiskSummary> {
        let chart = self.chart(interval, range).await?;

        let benchmark_returns = if let Some(sym) = benchmark {
            let bench_ticker = Ticker::new(sym).await?;
            let bench_chart = bench_ticker.chart(interval, range).await?;
            Some(crate::risk::candles_to_returns(&bench_chart.candles))
        } else {
            None
        };

        Ok(crate::risk::compute_risk_summary(
            &chart.candles,
            benchmark_returns.as_deref(),
        ))
    }

    // ========================================================================
    // SEC EDGAR
    // ========================================================================

    /// Get SEC EDGAR filing history for this symbol.
    ///
    /// Returns company metadata and recent filings. Results are cached for
    /// the lifetime of this `Ticker` instance.
    ///
    /// Requires EDGAR to be initialized via `edgar::init(email)`.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use finance_query::{Ticker, edgar};
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// edgar::init("user@example.com")?;
    /// let ticker = Ticker::new("AAPL").await?;
    ///
    /// let submissions = ticker.edgar_submissions().await?;
    /// println!("Company: {:?}", submissions.name);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn edgar_submissions(&self) -> Result<crate::models::edgar::EdgarSubmissions> {
        // Check cache
        {
            let cache = self.edgar_submissions_cache.read().await;
            if self.is_cache_fresh(cache.as_ref()) {
                return Ok(cache.as_ref().unwrap().value.clone());
            }
        }

        // Fetch using singleton
        let cik = crate::edgar::resolve_cik(&self.symbol).await?;
        let submissions = crate::edgar::submissions(cik).await?;

        // Only clone when caching is enabled
        if self.cache_ttl.is_some() {
            let mut cache = self.edgar_submissions_cache.write().await;
            *cache = Some(CacheEntry::new(submissions.clone()));
            Ok(submissions)
        } else {
            Ok(submissions)
        }
    }

    /// Get SEC EDGAR company facts (structured XBRL financial data) for this symbol.
    ///
    /// Returns all extracted XBRL facts organized by taxonomy. Results are cached
    /// for the lifetime of this `Ticker` instance.
    ///
    /// Requires EDGAR to be initialized via `edgar::init(email)`.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use finance_query::{Ticker, edgar};
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// edgar::init("user@example.com")?;
    /// let ticker = Ticker::new("AAPL").await?;
    ///
    /// let facts = ticker.edgar_company_facts().await?;
    /// if let Some(revenue) = facts.get_us_gaap_fact("Revenue") {
    ///     println!("Revenue data points: {:?}", revenue.units.keys().collect::<Vec<_>>());
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub async fn edgar_company_facts(&self) -> Result<crate::models::edgar::CompanyFacts> {
        // Check cache
        {
            let cache = self.edgar_facts_cache.read().await;
            if self.is_cache_fresh(cache.as_ref()) {
                return Ok(cache.as_ref().unwrap().value.clone());
            }
        }

        // Fetch using singleton
        let cik = crate::edgar::resolve_cik(&self.symbol).await?;
        let facts = crate::edgar::company_facts(cik).await?;

        // Only clone when caching is enabled
        if self.cache_ttl.is_some() {
            let mut cache = self.edgar_facts_cache.write().await;
            *cache = Some(CacheEntry::new(facts.clone()));
            Ok(facts)
        } else {
            Ok(facts)
        }
    }

    // ========================================================================
    // Cache Management
    // ========================================================================

    /// Clear all cached data, forcing fresh fetches on next access.
    ///
    /// Use this when you need up-to-date data from a long-lived `Ticker` instance.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// use finance_query::Ticker;
    ///
    /// let ticker = Ticker::new("AAPL").await?;
    /// let quote = ticker.quote().await?; // fetches from API
    ///
    /// // ... some time later ...
    /// ticker.clear_cache().await;
    /// let fresh_quote = ticker.quote().await?; // fetches again
    /// # Ok(())
    /// # }
    /// ```
    pub async fn clear_cache(&self) {
        // Acquire all independent write locks in parallel
        tokio::join!(
            async {
                *self.quote_summary.write().await = None;
            },
            async {
                self.chart_cache.write().await.clear();
            },
            async {
                *self.events_cache.write().await = None;
            },
            async {
                *self.recommendations_cache.write().await = None;
            },
            async {
                *self.news_cache.write().await = None;
            },
            async {
                self.options_cache.write().await.clear();
            },
            async {
                self.financials_cache.write().await.clear();
            },
            async {
                *self.edgar_submissions_cache.write().await = None;
            },
            async {
                *self.edgar_facts_cache.write().await = None;
            },
            async {
                #[cfg(feature = "indicators")]
                self.indicators_cache.write().await.clear();
            },
        );
    }

    /// Clear only the cached quote summary data.
    ///
    /// The next call to any quote accessor (e.g., `price()`, `financial_data()`)
    /// will re-fetch all quote modules from the API.
    pub async fn clear_quote_cache(&self) {
        *self.quote_summary.write().await = None;
    }

    /// Clear only the cached chart and events data.
    ///
    /// The next call to `chart()`, `dividends()`, `splits()`, or `capital_gains()`
    /// will re-fetch from the API.
    pub async fn clear_chart_cache(&self) {
        tokio::join!(
            async {
                self.chart_cache.write().await.clear();
            },
            async {
                *self.events_cache.write().await = None;
            },
            async {
                #[cfg(feature = "indicators")]
                self.indicators_cache.write().await.clear();
            }
        );
    }
}
