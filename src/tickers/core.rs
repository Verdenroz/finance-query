//! Tickers implementation for batch operations on multiple symbols.
//!
//! Optimizes data fetching by using batch endpoints and concurrent requests.

use super::macros::{batch_fetch_cached, define_batch_response};
use crate::client::{ClientConfig, YahooClient};
use crate::constants::{Frequency, Interval, StatementType, TimeRange};
use crate::error::{FinanceError, Result};
use crate::models::chart::events::ChartEvents;
use crate::models::chart::response::ChartResponse;
use crate::models::chart::{CapitalGain, Chart, Dividend, Split};
use crate::models::financials::FinancialStatement;
use crate::models::news::News;
use crate::models::options::Options;
use crate::models::quote::Quote;
use crate::models::recommendation::Recommendation;
use crate::models::spark::Spark;
use crate::models::spark::response::SparkResponse;
use crate::utils::{CacheEntry, EVICTION_THRESHOLD, filter_by_range};
use futures::stream::{self, StreamExt};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;

// Type aliases — MapCache wraps values in CacheEntry for TTL support.
type MapCache<K, V> = Arc<RwLock<HashMap<K, CacheEntry<V>>>>;
type ChartCacheKey = (Arc<str>, Interval, TimeRange);
type QuoteCache = MapCache<Arc<str>, Quote>;
type ChartCache = MapCache<ChartCacheKey, Chart>;
type EventsCache = MapCache<Arc<str>, ChartEvents>;
type FinancialsCache = MapCache<(Arc<str>, StatementType, Frequency), FinancialStatement>;
type NewsCache = MapCache<Arc<str>, Vec<News>>;
type RecommendationsCache = MapCache<(Arc<str>, u32), Recommendation>;
type OptionsCache = MapCache<(Arc<str>, Option<i64>), Options>;
type SparkCacheKey = (Arc<str>, Interval, TimeRange);
type SparkCache = MapCache<SparkCacheKey, Spark>;
#[cfg(feature = "indicators")]
type IndicatorsCache =
    MapCache<(Arc<str>, Interval, TimeRange), crate::indicators::IndicatorsSummary>;

// Fetch guards for request deduplication — prevent concurrent duplicate fetches
type FetchGuard = Arc<tokio::sync::Mutex<()>>;
type FetchGuardMap<K> = Arc<RwLock<HashMap<K, FetchGuard>>>;

// Generate all batch response types
define_batch_response! {
    /// Response containing quotes for multiple symbols.
    BatchQuotesResponse => quotes: Quote
}

define_batch_response! {
    /// Response containing charts for multiple symbols.
    BatchChartsResponse => charts: Chart
}

define_batch_response! {
    /// Response containing spark data for multiple symbols.
    ///
    /// Spark data is optimized for sparkline rendering with only close prices.
    /// Unlike charts, spark data is fetched in a single batch request.
    BatchSparksResponse => sparks: Spark
}

define_batch_response! {
    /// Response containing dividends for multiple symbols.
    BatchDividendsResponse => dividends: Vec<Dividend>
}

define_batch_response! {
    /// Response containing splits for multiple symbols.
    BatchSplitsResponse => splits: Vec<Split>
}

define_batch_response! {
    /// Response containing capital gains for multiple symbols.
    BatchCapitalGainsResponse => capital_gains: Vec<CapitalGain>
}

define_batch_response! {
    /// Response containing financial statements for multiple symbols.
    BatchFinancialsResponse => financials: FinancialStatement
}

define_batch_response! {
    /// Response containing news articles for multiple symbols.
    BatchNewsResponse => news: Vec<News>
}

define_batch_response! {
    /// Response containing recommendations for multiple symbols.
    BatchRecommendationsResponse => recommendations: Recommendation
}

define_batch_response! {
    /// Response containing options chains for multiple symbols.
    BatchOptionsResponse => options: Options
}

#[cfg(feature = "indicators")]
define_batch_response! {
    /// Response containing technical indicators for multiple symbols.
    BatchIndicatorsResponse => indicators: crate::indicators::IndicatorsSummary
}

/// Default maximum concurrent requests for batch operations.
const DEFAULT_MAX_CONCURRENCY: usize = 10;

/// Builder for Tickers
pub struct TickersBuilder {
    symbols: Vec<Arc<str>>,
    config: ClientConfig,
    shared_client: Option<crate::ticker::ClientHandle>,
    max_concurrency: usize,
    cache_ttl: Option<Duration>,
    include_logo: bool,
}

impl TickersBuilder {
    fn new<S, I>(symbols: I) -> Self
    where
        S: Into<String>,
        I: IntoIterator<Item = S>,
    {
        Self {
            symbols: symbols.into_iter().map(|s| s.into().into()).collect(),
            config: ClientConfig::default(),
            shared_client: None,
            max_concurrency: DEFAULT_MAX_CONCURRENCY,
            cache_ttl: None,
            include_logo: false,
        }
    }

    /// Set the region (automatically sets correct lang and region code)
    pub fn region(mut self, region: crate::constants::Region) -> Self {
        self.config.lang = region.lang().to_string();
        self.config.region = region.region().to_string();
        self
    }

    /// Set the language code (e.g., "en-US", "ja-JP", "de-DE")
    pub fn lang(mut self, lang: impl Into<String>) -> Self {
        self.config.lang = lang.into();
        self
    }

    /// Set the region code (e.g., "US", "JP", "DE")
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

    /// Set a complete ClientConfig
    pub fn config(mut self, config: ClientConfig) -> Self {
        self.config = config;
        self
    }

    /// Set the maximum number of concurrent requests for batch operations.
    ///
    /// Controls how many HTTP requests run in parallel when methods like
    /// `charts()`, `financials()`, or `news()` fetch data for each symbol.
    /// Default is 10.
    ///
    /// Lower values reduce the risk of rate limiting from Yahoo Finance.
    /// Higher values increase throughput for large symbol lists.
    pub fn max_concurrency(mut self, n: usize) -> Self {
        self.max_concurrency = n.max(1);
        self
    }

    /// Share an existing authenticated session instead of creating a new one.
    ///
    /// This avoids redundant authentication when you have multiple `Tickers`
    /// instances or want to share a session with individual [`Ticker`] instances.
    ///
    /// Obtain a [`ClientHandle`](crate::ClientHandle) from any existing
    /// [`Ticker`](crate::Ticker) via [`Ticker::client_handle()`](crate::Ticker::client_handle).
    ///
    /// When set, the builder's `config`, `timeout`, `proxy`, `lang`, and `region`
    /// settings are ignored (the shared session's configuration is used instead).
    ///
    /// # Example
    ///
    /// ```no_run
    /// use finance_query::{Ticker, Tickers};
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let aapl = Ticker::new("AAPL").await?;
    /// let handle = aapl.client_handle();
    ///
    /// let tickers = Tickers::builder(["MSFT", "GOOGL"])
    ///     .client(handle)
    ///     .build()
    ///     .await?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn client(mut self, handle: crate::ticker::ClientHandle) -> Self {
        self.shared_client = Some(handle);
        self
    }

    /// Enable response caching with a time-to-live.
    ///
    /// By default caching is **disabled** — every call fetches fresh data.
    /// When enabled, responses are reused until the TTL expires. Stale
    /// entries are evicted on the next write.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// use finance_query::Tickers;
    /// use std::time::Duration;
    ///
    /// let tickers = Tickers::builder(["AAPL", "MSFT"])
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
    /// When enabled, `quotes()` will fetch logo URLs in parallel with the
    /// quote batch request, adding a small extra request.
    pub fn logo(mut self) -> Self {
        self.include_logo = true;
        self
    }

    /// Build the Tickers instance
    pub async fn build(self) -> Result<Tickers> {
        let client = match self.shared_client {
            Some(handle) => handle.0,
            None => Arc::new(YahooClient::new(self.config).await?),
        };

        Ok(Tickers {
            symbols: self.symbols,
            client,
            max_concurrency: self.max_concurrency,
            cache_ttl: self.cache_ttl,
            include_logo: self.include_logo,
            quote_cache: Default::default(),
            chart_cache: Default::default(),
            events_cache: Default::default(),
            financials_cache: Default::default(),
            news_cache: Default::default(),
            recommendations_cache: Default::default(),
            options_cache: Default::default(),
            spark_cache: Default::default(),
            #[cfg(feature = "indicators")]
            indicators_cache: Default::default(),

            // Initialize fetch guards for request deduplication
            quotes_fetch: Arc::new(tokio::sync::Mutex::new(())),
            charts_fetch: Default::default(),
            financials_fetch: Default::default(),
            news_fetch: Arc::new(tokio::sync::Mutex::new(())),
            recommendations_fetch: Default::default(),
            options_fetch: Default::default(),
            spark_fetch: Default::default(),
            #[cfg(feature = "indicators")]
            indicators_fetch: Default::default(),
        })
    }
}

/// Multi-symbol ticker for efficient batch operations.
///
/// `Tickers` optimizes data fetching for multiple symbols by:
/// - Using batch endpoints where available (e.g., /v7/finance/quote)
/// - Fetching concurrently when batch endpoints don't exist
/// - Sharing a single authenticated client across all symbols
/// - Caching results per symbol
///
/// # Example
///
/// ```no_run
/// use finance_query::Tickers;
///
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// // Create tickers for multiple symbols
/// let tickers = Tickers::new(["AAPL", "MSFT", "GOOGL"]).await?;
///
/// // Batch fetch all quotes (single API call)
/// let quotes = tickers.quotes().await?;
/// for (symbol, quote) in &quotes.quotes {
///     let price = quote.regular_market_price.as_ref().and_then(|v| v.raw).unwrap_or(0.0);
///     println!("{}: ${:.2}", symbol, price);
/// }
///
/// // Fetch charts concurrently
/// use finance_query::{Interval, TimeRange};
/// let charts = tickers.charts(Interval::OneDay, TimeRange::OneMonth).await?;
/// # Ok(())
/// # }
/// ```
pub struct Tickers {
    symbols: Vec<Arc<str>>,
    client: Arc<YahooClient>,
    max_concurrency: usize,
    cache_ttl: Option<Duration>,
    include_logo: bool,
    quote_cache: QuoteCache,
    chart_cache: ChartCache,
    events_cache: EventsCache,
    financials_cache: FinancialsCache,
    news_cache: NewsCache,
    recommendations_cache: RecommendationsCache,
    options_cache: OptionsCache,
    spark_cache: SparkCache,
    #[cfg(feature = "indicators")]
    indicators_cache: IndicatorsCache,

    // Fetch guards prevent duplicate concurrent requests
    quotes_fetch: FetchGuard,
    charts_fetch: FetchGuardMap<(Interval, TimeRange)>,
    financials_fetch: FetchGuardMap<(StatementType, Frequency)>,
    news_fetch: FetchGuard,
    recommendations_fetch: FetchGuardMap<u32>,
    options_fetch: FetchGuardMap<Option<i64>>,
    spark_fetch: FetchGuardMap<(Interval, TimeRange)>,
    #[cfg(feature = "indicators")]
    indicators_fetch: FetchGuardMap<(Interval, TimeRange)>,
}

impl std::fmt::Debug for Tickers {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Tickers")
            .field("symbols", &self.symbols)
            .field("max_concurrency", &self.max_concurrency)
            .field("cache_ttl", &self.cache_ttl)
            .finish_non_exhaustive()
    }
}

impl Tickers {
    /// Creates new tickers with default configuration
    ///
    /// # Arguments
    ///
    /// * `symbols` - Iterable of stock symbols (e.g., `["AAPL", "MSFT"]`)
    ///
    /// # Example
    ///
    /// ```no_run
    /// use finance_query::Tickers;
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let tickers = Tickers::new(["AAPL", "MSFT", "GOOGL"]).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn new<S, I>(symbols: I) -> Result<Self>
    where
        S: Into<String>,
        I: IntoIterator<Item = S>,
    {
        Self::builder(symbols).build().await
    }

    /// Creates a new builder for Tickers
    pub fn builder<S, I>(symbols: I) -> TickersBuilder
    where
        S: Into<String>,
        I: IntoIterator<Item = S>,
    {
        TickersBuilder::new(symbols)
    }

    /// Returns the symbols this tickers instance manages
    pub fn symbols(&self) -> Vec<&str> {
        self.symbols.iter().map(|s| &**s).collect()
    }

    /// Number of symbols
    pub fn len(&self) -> usize {
        self.symbols.len()
    }

    /// Check if empty
    pub fn is_empty(&self) -> bool {
        self.symbols.is_empty()
    }

    /// Returns a shareable handle to this instance's authenticated session.
    ///
    /// Pass the handle to [`Ticker`](crate::Ticker) or other `Tickers` builders
    /// via `.client()` to reuse the same session without re-authenticating.
    pub fn client_handle(&self) -> crate::ticker::ClientHandle {
        crate::ticker::ClientHandle(Arc::clone(&self.client))
    }

    /// Returns `true` if a cache entry exists and has not exceeded the TTL.
    #[inline]
    fn is_cache_fresh<T>(&self, entry: Option<&CacheEntry<T>>) -> bool {
        CacheEntry::is_fresh_with_ttl(entry, self.cache_ttl)
    }

    /// Returns `true` if all keys are present and fresh in a map cache.
    fn all_cached<K: Eq + std::hash::Hash, V>(
        &self,
        map: &HashMap<K, CacheEntry<V>>,
        keys: impl Iterator<Item = K>,
    ) -> bool {
        let Some(ttl) = self.cache_ttl else {
            return false;
        };
        keys.into_iter()
            .all(|k| map.get(&k).map(|e| e.is_fresh(ttl)).unwrap_or(false))
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

    /// Batch fetch quotes for all symbols.
    ///
    /// Uses /v7/finance/quote endpoint - fetches all symbols in a single API call.
    /// When logos are enabled, makes a parallel call for logo URLs.
    ///
    /// Use [`TickersBuilder::logo()`](TickersBuilder::logo) to enable logo fetching
    /// for this tickers instance.
    pub async fn quotes(&self) -> Result<BatchQuotesResponse> {
        // Fast path: check if all symbols are cached
        {
            let cache = self.quote_cache.read().await;
            if self.all_cached(&cache, self.symbols.iter().cloned()) {
                let mut response = BatchQuotesResponse::with_capacity(self.symbols.len());
                for symbol in &self.symbols {
                    if let Some(entry) = cache.get(symbol) {
                        response
                            .quotes
                            .insert(symbol.to_string(), entry.value.clone());
                    }
                }
                return Ok(response);
            }
        }

        // Slow path: acquire fetch guard to prevent duplicate concurrent requests
        let _fetch_guard = self.quotes_fetch.lock().await;

        // Double-check: another task may have fetched while we waited
        {
            let cache = self.quote_cache.read().await;
            if self.all_cached(&cache, self.symbols.iter().cloned()) {
                let mut response = BatchQuotesResponse::with_capacity(self.symbols.len());
                for symbol in &self.symbols {
                    if let Some(entry) = cache.get(symbol) {
                        response
                            .quotes
                            .insert(symbol.to_string(), entry.value.clone());
                    }
                }
                return Ok(response);
            }
        }

        // Fetch batch quotes (no lock held during HTTP I/O)
        let symbols_ref: Vec<&str> = self.symbols.iter().map(|s| &**s).collect();

        // Yahoo requires separate calls for quotes vs logos
        // When include_logo=true, fetch both in parallel
        let (json, logos) = if self.include_logo {
            let quote_future = crate::endpoints::quotes::fetch_with_fields(
                &self.client,
                &symbols_ref,
                None,  // all fields
                true,  // formatted
                false, // no logo params for main call
            );
            let logo_future = crate::endpoints::quotes::fetch_with_fields(
                &self.client,
                &symbols_ref,
                Some(&["logoUrl", "companyLogoUrl"]), // only logo fields
                true,
                true, // include logo params
            );
            let (quote_result, logo_result) = tokio::join!(quote_future, logo_future);
            (quote_result?, logo_result.ok())
        } else {
            let json = crate::endpoints::quotes::fetch_with_fields(
                &self.client,
                &symbols_ref,
                None,
                true,
                false,
            )
            .await?;
            (json, None)
        };

        // Build logo lookup map if we have logos
        let logo_map: std::collections::HashMap<String, (Option<String>, Option<String>)> = logos
            .and_then(|l| l.get("quoteResponse")?.get("result")?.as_array().cloned())
            .map(|results| {
                results
                    .iter()
                    .filter_map(|r| {
                        let symbol = r.get("symbol")?.as_str()?.to_string();
                        let logo_url = r
                            .get("logoUrl")
                            .and_then(|v| v.as_str())
                            .map(|s| s.to_string());
                        let company_logo_url = r
                            .get("companyLogoUrl")
                            .and_then(|v| v.as_str())
                            .map(|s| s.to_string());
                        Some((symbol, (logo_url, company_logo_url)))
                    })
                    .collect()
            })
            .unwrap_or_default();

        // Parse response
        let mut response = BatchQuotesResponse::with_capacity(self.symbols.len());

        if let Some(quote_response) = json.get("quoteResponse") {
            if let Some(results) = quote_response.get("result").and_then(|r| r.as_array()) {
                // Parse all quotes first (no lock held)
                let mut parsed_quotes: Vec<(String, Quote)> = Vec::new();

                for result in results {
                    if let Some(symbol) = result.get("symbol").and_then(|s| s.as_str()) {
                        match Quote::from_batch_response(result) {
                            Ok(mut quote) => {
                                // Merge logo URLs if we have them
                                if let Some((logo_url, company_logo_url)) = logo_map.get(symbol) {
                                    if quote.logo_url.is_none() {
                                        quote.logo_url = logo_url.clone();
                                    }
                                    if quote.company_logo_url.is_none() {
                                        quote.company_logo_url = company_logo_url.clone();
                                    }
                                }
                                parsed_quotes.push((symbol.to_string(), quote));
                            }
                            Err(e) => {
                                response.errors.insert(symbol.to_string(), e.to_string());
                            }
                        }
                    }
                }

                // Now acquire write lock briefly for batch cache insertion
                if self.cache_ttl.is_some() {
                    let mut cache = self.quote_cache.write().await;
                    for (symbol, quote) in &parsed_quotes {
                        self.cache_insert(&mut cache, symbol.as_str().into(), quote.clone());
                    }
                }

                // Populate response (no lock needed)
                for (symbol, quote) in parsed_quotes {
                    response.quotes.insert(symbol, quote);
                }
            }

            // Track missing symbols
            for symbol in &self.symbols {
                let symbol_str = &**symbol;
                if !response.quotes.contains_key(symbol_str)
                    && !response.errors.contains_key(symbol_str)
                {
                    response.errors.insert(
                        symbol.to_string(),
                        "Symbol not found in response".to_string(),
                    );
                }
            }
        }

        Ok(response)
    }

    /// Get a specific quote by symbol (from cache or fetch all)
    pub async fn quote(&self, symbol: &str) -> Result<Quote> {
        {
            let cache = self.quote_cache.read().await;
            if let Some(entry) = cache.get(symbol)
                && self.is_cache_fresh(Some(entry))
            {
                return Ok(entry.value.clone());
            }
        }

        let response = self.quotes().await?;

        response
            .quotes
            .get(symbol)
            .cloned()
            .ok_or_else(|| FinanceError::SymbolNotFound {
                symbol: Some(symbol.to_string()),
                context: response
                    .errors
                    .get(symbol)
                    .cloned()
                    .unwrap_or_else(|| "Symbol not found".to_string()),
            })
    }

    /// Helper to get or create a fetch guard for a given key.
    ///
    /// Returns the guard from the map, never a locally-created copy that
    /// could diverge under contention.
    async fn get_fetch_guard<K: Clone + Eq + std::hash::Hash>(
        guard_map: &FetchGuardMap<K>,
        key: K,
    ) -> FetchGuard {
        {
            let guards = guard_map.read().await;
            if let Some(guard) = guards.get(&key) {
                return Arc::clone(guard);
            }
        }

        let mut guards = guard_map.write().await;
        Arc::clone(
            guards
                .entry(key)
                .or_insert_with(|| Arc::new(tokio::sync::Mutex::new(()))),
        )
    }

    /// Batch fetch charts for all symbols concurrently
    ///
    /// Chart data cannot be batched in a single request, so this fetches
    /// all charts concurrently using tokio for maximum performance.
    pub async fn charts(
        &self,
        interval: Interval,
        range: TimeRange,
    ) -> Result<BatchChartsResponse> {
        // Fast path: check if all symbols are cached
        {
            let cache = self.chart_cache.read().await;
            if self.all_cached(
                &cache,
                self.symbols.iter().map(|s| (s.clone(), interval, range)),
            ) {
                let mut response = BatchChartsResponse::with_capacity(self.symbols.len());
                for symbol in &self.symbols {
                    if let Some(entry) = cache.get(&(symbol.clone(), interval, range)) {
                        response
                            .charts
                            .insert(symbol.to_string(), entry.value.clone());
                    }
                }
                return Ok(response);
            }
        }

        // Slow path: acquire fetch guard to prevent duplicate concurrent requests
        let fetch_guard = Self::get_fetch_guard(&self.charts_fetch, (interval, range)).await;
        let _guard = fetch_guard.lock().await;

        // Double-check: another task may have fetched while we waited
        {
            let cache = self.chart_cache.read().await;
            if self.all_cached(
                &cache,
                self.symbols.iter().map(|s| (s.clone(), interval, range)),
            ) {
                let mut response = BatchChartsResponse::with_capacity(self.symbols.len());
                for symbol in &self.symbols {
                    if let Some(entry) = cache.get(&(symbol.clone(), interval, range)) {
                        response
                            .charts
                            .insert(symbol.to_string(), entry.value.clone());
                    }
                }
                return Ok(response);
            }
        }

        // Fetch all charts concurrently (no lock held during HTTP I/O)
        let futures: Vec<_> = self
            .symbols
            .iter()
            .map(|symbol| {
                let client = Arc::clone(&self.client);
                let symbol = Arc::clone(symbol);
                async move {
                    let result = client.get_chart(&symbol, interval, range).await;
                    (symbol, result)
                }
            })
            .collect();

        let results: Vec<_> = stream::iter(futures)
            .buffer_unordered(self.max_concurrency)
            .collect()
            .await;

        let mut response = BatchChartsResponse::with_capacity(self.symbols.len());

        // Parse all charts first (no locks held)
        let mut parsed_charts: Vec<(Arc<str>, Chart)> = Vec::new();
        let mut parsed_events: Vec<(Arc<str>, ChartEvents)> = Vec::new();

        for (symbol, result) in results {
            match result {
                Ok(json) => match ChartResponse::from_json(json) {
                    Ok(chart_response) => {
                        if let Some(mut chart_results) = chart_response.chart.result {
                            if let Some(chart_result) = chart_results.pop() {
                                // Collect events for later caching
                                if let Some(events) = chart_result.events.clone() {
                                    parsed_events.push((Arc::clone(&symbol), events));
                                }

                                let chart = Chart {
                                    symbol: symbol.to_string(),
                                    meta: chart_result.meta.clone(),
                                    candles: chart_result.to_candles(),
                                    interval: Some(interval),
                                    range: Some(range),
                                };
                                parsed_charts.push((symbol, chart));
                            } else {
                                response
                                    .errors
                                    .insert(symbol.to_string(), "Empty chart response".to_string());
                            }
                        } else {
                            response.errors.insert(
                                symbol.to_string(),
                                "No chart data in response".to_string(),
                            );
                        }
                    }
                    Err(e) => {
                        response.errors.insert(symbol.to_string(), e.to_string());
                    }
                },
                Err(e) => {
                    response.errors.insert(symbol.to_string(), e.to_string());
                }
            }
        }

        // Move into cache, then clone for response — avoids double-clone
        if self.cache_ttl.is_some() {
            let mut cache = self.chart_cache.write().await;

            // Cache all charts (consuming parsed_charts) and collect keys
            let cache_keys: Vec<_> = parsed_charts
                .into_iter()
                .map(|(symbol, chart)| {
                    self.cache_insert(&mut cache, (symbol.clone(), interval, range), chart);
                    symbol
                })
                .collect();

            // Clone from cache into response (convert Arc<str> → String)
            for symbol in cache_keys {
                if let Some(cached) = cache.get(&(symbol.clone(), interval, range)) {
                    response
                        .charts
                        .insert(symbol.to_string(), cached.value.clone());
                }
            }
        } else {
            // No caching: directly populate response (convert Arc<str> → String)
            for (symbol, chart) in parsed_charts {
                response.charts.insert(symbol.to_string(), chart);
            }
        }

        // Always store events — they are derived data, not TTL-bounded cache
        if !parsed_events.is_empty() {
            let mut events_cache = self.events_cache.write().await;
            for (symbol, events) in parsed_events {
                events_cache
                    .entry(symbol)
                    .or_insert_with(|| CacheEntry::new(events));
            }
        }

        Ok(response)
    }

    /// Get a specific chart by symbol
    pub async fn chart(&self, symbol: &str, interval: Interval, range: TimeRange) -> Result<Chart> {
        {
            let cache = self.chart_cache.read().await;
            let key: Arc<str> = symbol.into();
            if let Some(entry) = cache.get(&(key, interval, range))
                && self.is_cache_fresh(Some(entry))
            {
                return Ok(entry.value.clone());
            }
        }

        let response = self.charts(interval, range).await?;

        response
            .charts
            .get(symbol)
            .cloned()
            .ok_or_else(|| FinanceError::SymbolNotFound {
                symbol: Some(symbol.to_string()),
                context: response
                    .errors
                    .get(symbol)
                    .cloned()
                    .unwrap_or_else(|| "Symbol not found".to_string()),
            })
    }

    /// Batch fetch chart data for a custom date range for all symbols concurrently.
    ///
    /// Unlike [`charts()`](Self::charts) which uses predefined time ranges,
    /// this method accepts absolute start/end timestamps. Results are **not cached**
    /// since custom ranges have unbounded key space.
    ///
    /// # Arguments
    ///
    /// * `interval` - Time interval between data points
    /// * `start` - Start date as Unix timestamp (seconds since epoch)
    /// * `end` - End date as Unix timestamp (seconds since epoch)
    pub async fn charts_range(
        &self,
        interval: Interval,
        start: i64,
        end: i64,
    ) -> Result<BatchChartsResponse> {
        let futures: Vec<_> = self
            .symbols
            .iter()
            .map(|symbol| {
                let client = Arc::clone(&self.client);
                let symbol = Arc::clone(symbol);
                async move {
                    let result = client.get_chart_range(&symbol, interval, start, end).await;
                    (symbol, result)
                }
            })
            .collect();

        let results: Vec<_> = stream::iter(futures)
            .buffer_unordered(self.max_concurrency)
            .collect()
            .await;

        let mut response = BatchChartsResponse::with_capacity(self.symbols.len());
        let mut parsed_events: Vec<(Arc<str>, ChartEvents)> = Vec::new();

        for (symbol, result) in results {
            match result {
                Ok(json) => match ChartResponse::from_json(json) {
                    Ok(chart_response) => {
                        if let Some(mut chart_results) = chart_response.chart.result {
                            if let Some(chart_result) = chart_results.pop() {
                                // Collect events for later caching
                                if let Some(events) = chart_result.events.clone() {
                                    parsed_events.push((Arc::clone(&symbol), events));
                                }

                                let chart = Chart {
                                    symbol: symbol.to_string(),
                                    meta: chart_result.meta.clone(),
                                    candles: chart_result.to_candles(),
                                    interval: Some(interval),
                                    range: None,
                                };
                                response.charts.insert(symbol.to_string(), chart);
                            } else {
                                response
                                    .errors
                                    .insert(symbol.to_string(), "Empty chart response".to_string());
                            }
                        } else {
                            response.errors.insert(
                                symbol.to_string(),
                                "No chart data in response".to_string(),
                            );
                        }
                    }
                    Err(e) => {
                        response.errors.insert(symbol.to_string(), e.to_string());
                    }
                },
                Err(e) => {
                    response.errors.insert(symbol.to_string(), e.to_string());
                }
            }
        }

        // Always store events — they are derived data, not TTL-bounded cache
        if !parsed_events.is_empty() {
            let mut events_cache = self.events_cache.write().await;
            for (symbol, events) in parsed_events {
                events_cache
                    .entry(symbol)
                    .or_insert_with(|| CacheEntry::new(events));
            }
        }

        Ok(response)
    }

    /// Ensures events are loaded for all symbols using chart requests.
    ///
    /// Fetches events concurrently for symbols that don't have cached events.
    /// Uses `TimeRange::Max` to get full event history (Yahoo returns all
    /// dividends/splits/capital gains regardless of chart range).
    ///
    /// Events are always stored regardless of `cache_ttl` because they are
    /// derived data (not a TTL-bounded cache). When `cache_ttl` is `None`,
    /// events persist for the lifetime of the `Tickers` instance.
    async fn ensure_events_loaded(&self) -> Result<()> {
        // Check which symbols need event data (existence check, not TTL-based)
        let symbols_to_fetch: Vec<Arc<str>> = {
            let cache = self.events_cache.read().await;
            self.symbols
                .iter()
                .filter(|sym| !cache.contains_key(*sym))
                .cloned()
                .collect()
        };

        if symbols_to_fetch.is_empty() {
            return Ok(());
        }

        // Fetch events concurrently for all symbols that need it
        let futures: Vec<_> = symbols_to_fetch
            .iter()
            .map(|symbol| {
                let client = Arc::clone(&self.client);
                let symbol = Arc::clone(symbol);
                async move {
                    let result = crate::endpoints::chart::fetch(
                        &client,
                        &symbol,
                        Interval::OneDay,
                        TimeRange::Max,
                    )
                    .await;
                    (symbol, result)
                }
            })
            .collect();

        let results: Vec<_> = stream::iter(futures)
            .buffer_unordered(self.max_concurrency)
            .collect()
            .await;

        // Parse and cache events
        let mut parsed_events: Vec<(Arc<str>, ChartEvents)> = Vec::new();

        for (symbol, result) in results {
            if let Ok(json) = result
                && let Ok(chart_response) = ChartResponse::from_json(json)
                && let Some(mut chart_results) = chart_response.chart.result
                && let Some(chart_result) = chart_results.pop()
                && let Some(events) = chart_result.events
            {
                parsed_events.push((symbol, events));
            }
        }

        // Always store events — they are derived data, not TTL-bounded cache
        if !parsed_events.is_empty() {
            let mut events_cache = self.events_cache.write().await;
            for (symbol, events) in parsed_events {
                events_cache.insert(symbol, CacheEntry::new(events));
            }
        }

        Ok(())
    }

    /// Batch fetch spark data for all symbols in a single request.
    ///
    /// Spark data is optimized for sparkline rendering, returning only close prices.
    /// Unlike `charts()`, this fetches all symbols in ONE API call, making it
    /// much more efficient for displaying price trends on dashboards or watchlists.
    ///
    /// # Arguments
    ///
    /// * `interval` - Time interval between data points (e.g., `Interval::FiveMinutes`)
    /// * `range` - Time range to fetch (e.g., `TimeRange::OneDay`)
    ///
    /// # Example
    ///
    /// ```no_run
    /// use finance_query::{Tickers, Interval, TimeRange};
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let tickers = Tickers::new(["AAPL", "MSFT", "GOOGL"]).await?;
    /// let sparks = tickers.spark(Interval::FiveMinutes, TimeRange::OneDay).await?;
    ///
    /// for (symbol, spark) in &sparks.sparks {
    ///     if let Some(change) = spark.percent_change() {
    ///         println!("{}: {:.2}%", symbol, change);
    ///     }
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub async fn spark(&self, interval: Interval, range: TimeRange) -> Result<BatchSparksResponse> {
        // Fast path: check if all symbols are cached
        {
            let cache = self.spark_cache.read().await;
            if self.all_cached(
                &cache,
                self.symbols.iter().map(|s| (s.clone(), interval, range)),
            ) {
                let mut response = BatchSparksResponse::with_capacity(self.symbols.len());
                for symbol in &self.symbols {
                    if let Some(entry) = cache.get(&(symbol.clone(), interval, range)) {
                        response
                            .sparks
                            .insert(symbol.to_string(), entry.value.clone());
                    }
                }
                return Ok(response);
            }
        }

        // Slow path: acquire fetch guard
        let fetch_guard = Self::get_fetch_guard(&self.spark_fetch, (interval, range)).await;
        let _guard = fetch_guard.lock().await;

        // Double-check after guard
        {
            let cache = self.spark_cache.read().await;
            if self.all_cached(
                &cache,
                self.symbols.iter().map(|s| (s.clone(), interval, range)),
            ) {
                let mut response = BatchSparksResponse::with_capacity(self.symbols.len());
                for symbol in &self.symbols {
                    if let Some(entry) = cache.get(&(symbol.clone(), interval, range)) {
                        response
                            .sparks
                            .insert(symbol.to_string(), entry.value.clone());
                    }
                }
                return Ok(response);
            }
        }

        // Fetch (single batch API call, no lock held during I/O)
        let symbols_ref: Vec<&str> = self.symbols.iter().map(|s| &**s).collect();
        let json =
            crate::endpoints::spark::fetch(&self.client, &symbols_ref, interval, range).await?;

        let mut response = BatchSparksResponse::with_capacity(self.symbols.len());

        match SparkResponse::from_json(json) {
            Ok(spark_response) => {
                let mut parsed_sparks: Vec<(Arc<str>, Spark)> = Vec::new();

                if let Some(results) = spark_response.spark.result {
                    for result in &results {
                        if let Some(spark) = Spark::from_response(
                            result,
                            Some(interval.as_str().to_string()),
                            Some(range.as_str().to_string()),
                        ) {
                            let sym: Arc<str> = result.symbol.as_str().into();
                            parsed_sparks.push((sym, spark));
                        } else {
                            response.errors.insert(
                                result.symbol.to_string(),
                                "Failed to parse spark data".to_string(),
                            );
                        }
                    }
                }

                // Cache all parsed sparks
                if self.cache_ttl.is_some() {
                    let mut cache = self.spark_cache.write().await;
                    for (symbol, spark) in &parsed_sparks {
                        self.cache_insert(
                            &mut cache,
                            (symbol.clone(), interval, range),
                            spark.clone(),
                        );
                    }
                }

                // Build response
                for (symbol, spark) in parsed_sparks {
                    response.sparks.insert(symbol.to_string(), spark);
                }

                // Track missing symbols
                for symbol in &self.symbols {
                    let symbol_str = &**symbol;
                    if !response.sparks.contains_key(symbol_str)
                        && !response.errors.contains_key(symbol_str)
                    {
                        response.errors.insert(
                            symbol.to_string(),
                            "Symbol not found in response".to_string(),
                        );
                    }
                }
            }
            Err(e) => {
                for symbol in &self.symbols {
                    response.errors.insert(symbol.to_string(), e.to_string());
                }
            }
        }

        Ok(response)
    }

    /// Batch fetch dividends for all symbols
    ///
    /// Returns dividend history for all symbols, filtered by the specified time range.
    /// Dividends are cached per symbol after the first chart fetch.
    ///
    /// # Arguments
    ///
    /// * `range` - Time range to filter dividends
    ///
    /// # Example
    ///
    /// ```no_run
    /// use finance_query::{Tickers, TimeRange};
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let tickers = Tickers::new(["AAPL", "MSFT"]).await?;
    /// let dividends = tickers.dividends(TimeRange::OneYear).await?;
    ///
    /// for (symbol, divs) in &dividends.dividends {
    ///     println!("{}: {} dividends", symbol, divs.len());
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub async fn dividends(&self, range: TimeRange) -> Result<BatchDividendsResponse> {
        let mut response = BatchDividendsResponse::with_capacity(self.symbols.len());

        // Fetch events efficiently (1-day chart request per symbol)
        self.ensure_events_loaded().await?;

        let events_cache = self.events_cache.read().await;

        for symbol in &self.symbols {
            if let Some(entry) = events_cache.get(symbol) {
                let all_dividends = entry.value.to_dividends();
                let filtered = filter_by_range(all_dividends, range);
                response.dividends.insert(symbol.to_string(), filtered);
            } else {
                response
                    .errors
                    .insert(symbol.to_string(), "No events data available".to_string());
            }
        }

        Ok(response)
    }

    /// Batch fetch stock splits for all symbols
    ///
    /// Returns stock split history for all symbols, filtered by the specified time range.
    /// Splits are cached per symbol after the first chart fetch.
    ///
    /// # Arguments
    ///
    /// * `range` - Time range to filter splits
    ///
    /// # Example
    ///
    /// ```no_run
    /// use finance_query::{Tickers, TimeRange};
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let tickers = Tickers::new(["NVDA", "TSLA"]).await?;
    /// let splits = tickers.splits(TimeRange::FiveYears).await?;
    ///
    /// for (symbol, sp) in &splits.splits {
    ///     for split in sp {
    ///         println!("{}: {}", symbol, split.ratio);
    ///     }
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub async fn splits(&self, range: TimeRange) -> Result<BatchSplitsResponse> {
        let mut response = BatchSplitsResponse::with_capacity(self.symbols.len());

        // Fetch events efficiently (1-day chart request per symbol)
        self.ensure_events_loaded().await?;

        let events_cache = self.events_cache.read().await;

        for symbol in &self.symbols {
            if let Some(entry) = events_cache.get(symbol) {
                let all_splits = entry.value.to_splits();
                let filtered = filter_by_range(all_splits, range);
                response.splits.insert(symbol.to_string(), filtered);
            } else {
                response
                    .errors
                    .insert(symbol.to_string(), "No events data available".to_string());
            }
        }

        Ok(response)
    }

    /// Batch fetch capital gains for all symbols
    ///
    /// Returns capital gain distribution history for all symbols, filtered by the
    /// specified time range. This is primarily relevant for mutual funds and ETFs.
    /// Capital gains are cached per symbol after the first chart fetch.
    ///
    /// # Arguments
    ///
    /// * `range` - Time range to filter capital gains
    ///
    /// # Example
    ///
    /// ```no_run
    /// use finance_query::{Tickers, TimeRange};
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let tickers = Tickers::new(["VFIAX", "VTI"]).await?;
    /// let gains = tickers.capital_gains(TimeRange::TwoYears).await?;
    ///
    /// for (symbol, cg) in &gains.capital_gains {
    ///     println!("{}: {} distributions", symbol, cg.len());
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub async fn capital_gains(&self, range: TimeRange) -> Result<BatchCapitalGainsResponse> {
        let mut response = BatchCapitalGainsResponse::with_capacity(self.symbols.len());

        // Fetch events efficiently (1-day chart request per symbol)
        self.ensure_events_loaded().await?;

        let events_cache = self.events_cache.read().await;

        for symbol in &self.symbols {
            if let Some(entry) = events_cache.get(symbol) {
                let all_gains = entry.value.to_capital_gains();
                let filtered = filter_by_range(all_gains, range);
                response.capital_gains.insert(symbol.to_string(), filtered);
            } else {
                response
                    .errors
                    .insert(symbol.to_string(), "No events data available".to_string());
            }
        }

        Ok(response)
    }

    /// Batch fetch financial statements for all symbols
    ///
    /// Fetches the specified financial statement type for all symbols concurrently.
    /// Financial statements are cached per (symbol, statement_type, frequency) tuple.
    ///
    /// # Arguments
    ///
    /// * `statement_type` - Type of statement (Income, Balance, CashFlow)
    /// * `frequency` - Annual or Quarterly
    ///
    /// # Example
    ///
    /// ```no_run
    /// use finance_query::{Tickers, StatementType, Frequency};
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let tickers = Tickers::new(["AAPL", "MSFT", "GOOGL"]).await?;
    /// let financials = tickers.financials(StatementType::Income, Frequency::Annual).await?;
    ///
    /// for (symbol, stmt) in &financials.financials {
    ///     if let Some(revenue) = stmt.statement.get("TotalRevenue") {
    ///         println!("{}: {:?}", symbol, revenue);
    ///     }
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub async fn financials(
        &self,
        statement_type: StatementType,
        frequency: Frequency,
    ) -> Result<BatchFinancialsResponse> {
        batch_fetch_cached!(self;
            cache: financials_cache,
            guard: map(financials_fetch, (statement_type, frequency)),
            key: |s| (s.clone(), statement_type, frequency),
            response: BatchFinancialsResponse.financials,
            fetch: |client, symbol| client.get_financials(&symbol, statement_type, frequency).await,
        )
    }

    /// Batch fetch news articles for all symbols
    ///
    /// Fetches recent news articles for all symbols concurrently using scrapers.
    /// News articles are cached per symbol.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use finance_query::Tickers;
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let tickers = Tickers::new(["AAPL", "MSFT"]).await?;
    /// let news = tickers.news().await?;
    ///
    /// for (symbol, articles) in &news.news {
    ///     println!("{}: {} articles", symbol, articles.len());
    ///     for article in articles.iter().take(3) {
    ///         println!("  - {}", article.title);
    ///     }
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub async fn news(&self) -> Result<BatchNewsResponse> {
        batch_fetch_cached!(self;
            cache: news_cache,
            guard: simple(news_fetch),
            key: |s| s.clone(),
            response: BatchNewsResponse.news,
            fetch: |_client, symbol| crate::scrapers::stockanalysis::scrape_symbol_news(&symbol).await,
        )
    }

    /// Batch fetch recommendations for all symbols
    ///
    /// Fetches analyst recommendations and similar stocks for all symbols concurrently.
    /// Recommendations are cached per (symbol, limit) tuple — different limits
    /// produce different API responses and are cached independently.
    ///
    /// # Arguments
    ///
    /// * `limit` - Maximum number of similar stocks to return per symbol
    ///
    /// # Example
    ///
    /// ```no_run
    /// use finance_query::Tickers;
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let tickers = Tickers::new(["AAPL", "MSFT"]).await?;
    /// let recommendations = tickers.recommendations(10).await?;
    ///
    /// for (symbol, rec) in &recommendations.recommendations {
    ///     println!("{}: {} recommendations", symbol, rec.count());
    ///     for similar in &rec.recommendations {
    ///         println!("  - {}: score {}", similar.symbol, similar.score);
    ///     }
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub async fn recommendations(&self, limit: u32) -> Result<BatchRecommendationsResponse> {
        batch_fetch_cached!(self;
            cache: recommendations_cache,
            guard: map(recommendations_fetch, limit),
            key: |s| (s.clone(), limit),
            response: BatchRecommendationsResponse.recommendations,
            fetch: |client, symbol| {
                let json = client.get_recommendations(&symbol, limit).await?;
                let rec_response =
                    crate::models::recommendation::response::RecommendationResponse::from_json(json)?;
                Ok(Recommendation {
                    symbol: symbol.to_string(),
                    recommendations: rec_response
                        .finance
                        .result
                        .iter()
                        .flat_map(|r| &r.recommended_symbols)
                        .cloned()
                        .collect(),
                })
            },
        )
    }

    /// Batch fetch options chains for all symbols
    ///
    /// Fetches options chains for the specified expiration date for all symbols concurrently.
    /// Options are cached per (symbol, date) tuple.
    ///
    /// # Arguments
    ///
    /// * `date` - Optional expiration date (Unix timestamp). If None, fetches nearest expiration.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use finance_query::Tickers;
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let tickers = Tickers::new(["AAPL", "MSFT"]).await?;
    /// let options = tickers.options(None).await?;
    ///
    /// for (symbol, opts) in &options.options {
    ///     println!("{}: {} expirations", symbol, opts.expiration_dates().len());
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub async fn options(&self, date: Option<i64>) -> Result<BatchOptionsResponse> {
        batch_fetch_cached!(self;
            cache: options_cache,
            guard: map(options_fetch, date),
            key: |s| (s.clone(), date),
            response: BatchOptionsResponse.options,
            fetch: |client, symbol| {
                let json = client.get_options(&symbol, date).await?;
                Ok(serde_json::from_value::<Options>(json)?)
            },
        )
    }

    /// Batch calculate all technical indicators for all symbols
    ///
    /// Calculates complete indicator summaries for all symbols from their chart data.
    /// Indicators are cached per (symbol, interval, range) tuple.
    ///
    /// # Arguments
    ///
    /// * `interval` - The time interval for each candle
    /// * `range` - The time range to fetch data for
    ///
    /// # Example
    ///
    /// ```no_run
    /// use finance_query::{Tickers, Interval, TimeRange};
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let tickers = Tickers::new(["AAPL", "MSFT"]).await?;
    /// let indicators = tickers.indicators(Interval::OneDay, TimeRange::ThreeMonths).await?;
    ///
    /// for (symbol, ind) in &indicators.indicators {
    ///     println!("{}: RSI(14) = {:?}, SMA(20) = {:?}", symbol, ind.rsi_14, ind.sma_20);
    /// }
    /// # Ok(())
    /// # }
    /// ```
    #[cfg(feature = "indicators")]
    pub async fn indicators(
        &self,
        interval: Interval,
        range: TimeRange,
    ) -> Result<BatchIndicatorsResponse> {
        let cache_key_for = |symbol: &Arc<str>| (symbol.clone(), interval, range);

        // Fast path: check if all symbols are cached
        {
            let cache = self.indicators_cache.read().await;
            if self.all_cached(&cache, self.symbols.iter().map(&cache_key_for)) {
                let mut response = BatchIndicatorsResponse::with_capacity(self.symbols.len());
                for symbol in &self.symbols {
                    if let Some(entry) = cache.get(&cache_key_for(symbol)) {
                        response
                            .indicators
                            .insert(symbol.to_string(), entry.value.clone());
                    }
                }
                return Ok(response);
            }
        }

        // Slow path: acquire fetch guard to prevent duplicate concurrent calculations
        let fetch_guard = Self::get_fetch_guard(&self.indicators_fetch, (interval, range)).await;
        let _guard = fetch_guard.lock().await;

        // Double-check: another task may have computed while we waited
        {
            let cache = self.indicators_cache.read().await;
            if self.all_cached(&cache, self.symbols.iter().map(&cache_key_for)) {
                let mut response = BatchIndicatorsResponse::with_capacity(self.symbols.len());
                for symbol in &self.symbols {
                    if let Some(entry) = cache.get(&cache_key_for(symbol)) {
                        response
                            .indicators
                            .insert(symbol.to_string(), entry.value.clone());
                    }
                }
                return Ok(response);
            }
        }

        // Fetch charts first (which may already be cached, has its own deduplication)
        let charts_response = self.charts(interval, range).await?;

        let mut response = BatchIndicatorsResponse::with_capacity(self.symbols.len());

        // Calculate all indicators first (no lock held)
        let mut calculated_indicators: Vec<(String, crate::indicators::IndicatorsSummary)> =
            Vec::new();

        for (symbol, chart) in &charts_response.charts {
            let indicators = crate::indicators::summary::calculate_indicators(&chart.candles);
            calculated_indicators.push((symbol.to_string(), indicators));
        }

        // Now acquire write lock briefly for batch cache insertion
        if self.cache_ttl.is_some() {
            let mut cache = self.indicators_cache.write().await;
            for (symbol, indicators) in &calculated_indicators {
                let key: Arc<str> = symbol.as_str().into();
                self.cache_insert(&mut cache, cache_key_for(&key), indicators.clone());
            }
        }

        // Populate response (no lock needed)
        for (symbol, indicators) in calculated_indicators {
            response.indicators.insert(symbol, indicators);
        }

        // Add errors from chart fetch
        for (symbol, error) in &charts_response.errors {
            response.errors.insert(symbol.to_string(), error.clone());
        }

        Ok(response)
    }

    // ========================================================================
    // Dynamic Symbol Management
    // ========================================================================

    /// Add symbols to the watch list
    ///
    /// Adds new symbols to track without affecting existing cached data.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use finance_query::Tickers;
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let mut tickers = Tickers::new(["AAPL"]).await?;
    /// tickers.add_symbols(&["MSFT", "GOOGL"]);
    /// assert_eq!(tickers.len(), 3);
    /// # Ok(())
    /// # }
    /// ```
    pub fn add_symbols(&mut self, symbols: &[impl AsRef<str>]) {
        // Use HashSet for O(n+m) deduplication instead of O(n*m) linear search
        use std::collections::HashSet;

        let existing: HashSet<&str> = self.symbols.iter().map(|s| &**s).collect();
        let to_add: Vec<Arc<str>> = symbols
            .iter()
            .map(|s| s.as_ref())
            .filter(|s| !existing.contains(s))
            .map(|s| s.into())
            .collect();

        self.symbols.extend(to_add);
    }

    /// Remove symbols from the watch list
    ///
    /// Removes symbols and clears their cached data to free memory.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use finance_query::Tickers;
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let mut tickers = Tickers::new(["AAPL", "MSFT", "GOOGL"]).await?;
    /// tickers.remove_symbols(&["MSFT"]);
    /// assert_eq!(tickers.len(), 2);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn remove_symbols(&mut self, symbols: &[impl AsRef<str>]) {
        use std::collections::HashSet;
        let to_remove: HashSet<&str> = symbols.iter().map(|s| s.as_ref()).collect();

        // Remove from symbol list — O(1) lookup per element
        self.symbols.retain(|s| !to_remove.contains(&**s));

        // Acquire all independent write locks in parallel
        let (
            mut quote_cache,
            mut chart_cache,
            mut events_cache,
            mut financials_cache,
            mut news_cache,
            mut recommendations_cache,
            mut options_cache,
            mut spark_cache,
        ) = tokio::join!(
            self.quote_cache.write(),
            self.chart_cache.write(),
            self.events_cache.write(),
            self.financials_cache.write(),
            self.news_cache.write(),
            self.recommendations_cache.write(),
            self.options_cache.write(),
            self.spark_cache.write(),
        );

        // Simple key caches — O(1) per removal
        for symbol in &to_remove {
            let key: Arc<str> = (*symbol).into();
            quote_cache.remove(&key);
            events_cache.remove(&key);
            news_cache.remove(&key);
        }

        // Composite key caches — O(n) retain but O(1) contains check
        chart_cache.retain(|(sym, _, _), _| !to_remove.contains(&**sym));
        financials_cache.retain(|(sym, _, _), _| !to_remove.contains(&**sym));
        recommendations_cache.retain(|(sym, _), _| !to_remove.contains(&**sym));
        options_cache.retain(|(sym, _), _| !to_remove.contains(&**sym));
        spark_cache.retain(|(sym, _, _), _| !to_remove.contains(&**sym));

        // Drop all guards before cfg-gated lock
        drop((
            quote_cache,
            chart_cache,
            events_cache,
            financials_cache,
            news_cache,
            recommendations_cache,
            options_cache,
            spark_cache,
        ));

        #[cfg(feature = "indicators")]
        self.indicators_cache
            .write()
            .await
            .retain(|(sym, _, _), _| !to_remove.contains(&**sym));
    }

    /// Clear all cached data and fetch guards, forcing fresh fetches on next access.
    ///
    /// Use this when you need up-to-date data from a long-lived `Tickers` instance.
    /// Also clears fetch guard maps to prevent unbounded growth.
    pub async fn clear_cache(&self) {
        tokio::join!(
            // Data caches
            async { self.quote_cache.write().await.clear() },
            async { self.chart_cache.write().await.clear() },
            async { self.events_cache.write().await.clear() },
            async { self.financials_cache.write().await.clear() },
            async { self.news_cache.write().await.clear() },
            async { self.recommendations_cache.write().await.clear() },
            async { self.options_cache.write().await.clear() },
            async { self.spark_cache.write().await.clear() },
            async {
                #[cfg(feature = "indicators")]
                self.indicators_cache.write().await.clear();
            },
            // Fetch guard maps (prevent unbounded growth)
            async { self.charts_fetch.write().await.clear() },
            async { self.financials_fetch.write().await.clear() },
            async { self.recommendations_fetch.write().await.clear() },
            async { self.options_fetch.write().await.clear() },
            async { self.spark_fetch.write().await.clear() },
            async {
                #[cfg(feature = "indicators")]
                self.indicators_fetch.write().await.clear();
            },
        );
    }

    /// Clear only the cached quote data.
    ///
    /// The next call to `quotes()` or `quote()` will re-fetch from the API.
    pub async fn clear_quote_cache(&self) {
        self.quote_cache.write().await.clear();
    }

    /// Clear only the cached chart, spark, and events data.
    ///
    /// The next call to `charts()`, `spark()`, `dividends()`, `splits()`,
    /// or `capital_gains()` will re-fetch from the API.
    pub async fn clear_chart_cache(&self) {
        tokio::join!(
            async { self.chart_cache.write().await.clear() },
            async { self.events_cache.write().await.clear() },
            async { self.spark_cache.write().await.clear() },
            async {
                #[cfg(feature = "indicators")]
                self.indicators_cache.write().await.clear();
            },
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    #[ignore] // Requires network access
    async fn test_tickers_quotes() {
        let tickers = Tickers::new(["AAPL", "MSFT", "GOOGL"]).await.unwrap();
        let result = tickers.quotes().await.unwrap();

        assert!(result.success_count() > 0);
    }

    #[tokio::test]
    #[ignore] // Requires network access
    async fn test_tickers_charts() {
        let tickers = Tickers::new(["AAPL", "MSFT"]).await.unwrap();
        let result = tickers
            .charts(Interval::OneDay, TimeRange::FiveDays)
            .await
            .unwrap();

        assert!(result.success_count() > 0);
    }

    #[tokio::test]
    #[ignore = "requires network access"]
    async fn test_tickers_spark() {
        let tickers = Tickers::new(["AAPL", "MSFT", "GOOGL"]).await.unwrap();
        let result = tickers
            .spark(Interval::FiveMinutes, TimeRange::OneDay)
            .await
            .unwrap();

        assert!(result.success_count() > 0);

        // Verify spark data structure
        if let Some(spark) = result.sparks.get("AAPL") {
            assert!(!spark.closes.is_empty());
            assert_eq!(spark.symbol, "AAPL");
            // Verify helper methods work
            assert!(spark.percent_change().is_some());
        }
    }

    #[tokio::test]
    #[ignore = "requires network access"]
    async fn test_tickers_dividends() {
        let tickers = Tickers::new(["AAPL", "MSFT"]).await.unwrap();
        let result = tickers.dividends(TimeRange::OneYear).await.unwrap();

        assert!(result.success_count() > 0);

        // Verify dividend data structure
        if let Some(dividends) = result.dividends.get("AAPL")
            && !dividends.is_empty()
        {
            let div = &dividends[0];
            assert!(div.timestamp > 0);
            assert!(div.amount > 0.0);
        }
    }

    #[tokio::test]
    #[ignore = "requires network access"]
    async fn test_tickers_splits() {
        let tickers = Tickers::new(["NVDA", "TSLA"]).await.unwrap();
        let result = tickers.splits(TimeRange::FiveYears).await.unwrap();

        // Note: Not all symbols have splits, so we just check for successful response
        assert!(result.success_count() > 0);

        // If there are splits, verify structure
        for splits in result.splits.values() {
            for split in splits {
                assert!(split.timestamp > 0);
                assert!(split.numerator > 0.0);
                assert!(split.denominator > 0.0);
                assert!(!split.ratio.is_empty());
            }
        }
    }

    #[tokio::test]
    #[ignore = "requires network access"]
    async fn test_tickers_capital_gains() {
        let tickers = Tickers::new(["VFIAX", "VTI"]).await.unwrap();
        let result = tickers.capital_gains(TimeRange::TwoYears).await.unwrap();

        // Note: Not all symbols have capital gains distributions
        assert!(result.success_count() > 0);

        // If there are capital gains, verify structure
        for gains in result.capital_gains.values() {
            for gain in gains {
                assert!(gain.timestamp > 0);
                assert!(gain.amount >= 0.0);
            }
        }
    }

    #[tokio::test]
    #[ignore = "requires network access"]
    async fn test_tickers_financials() {
        let tickers = Tickers::new(["AAPL", "MSFT"]).await.unwrap();
        let result = tickers
            .financials(StatementType::Income, Frequency::Annual)
            .await
            .unwrap();

        assert!(result.success_count() > 0);

        // Verify financial statement structure
        for (symbol, stmt) in &result.financials {
            assert_eq!(stmt.symbol, *symbol);
            assert_eq!(stmt.statement_type, "income");
            assert_eq!(stmt.frequency, "annual");
            assert!(!stmt.statement.is_empty());

            // Common income statement fields
            if let Some(revenue) = stmt.statement.get("TotalRevenue") {
                assert!(!revenue.is_empty());
            }
        }
    }

    #[tokio::test]
    #[ignore = "requires network access"]
    async fn test_tickers_news() {
        let tickers = Tickers::new(["AAPL", "TSLA"]).await.unwrap();
        let result = tickers.news().await.unwrap();

        assert!(result.success_count() > 0);

        // Verify news structure
        for articles in result.news.values() {
            if !articles.is_empty() {
                let article = &articles[0];
                assert!(!article.title.is_empty());
                assert!(!article.link.is_empty());
                assert!(!article.source.is_empty());
            }
        }
    }

    #[tokio::test]
    #[ignore = "requires network access"]
    async fn test_tickers_recommendations() {
        let tickers = Tickers::new(["AAPL", "MSFT"]).await.unwrap();
        let result = tickers.recommendations(5).await.unwrap();

        assert!(result.success_count() > 0);

        // Verify recommendations structure
        for (symbol, rec) in &result.recommendations {
            assert_eq!(rec.symbol, *symbol);
            assert!(rec.count() > 0);
            for similar in &rec.recommendations {
                assert!(!similar.symbol.is_empty());
            }
        }
    }

    #[tokio::test]
    #[ignore = "requires network access"]
    async fn test_tickers_options() {
        let tickers = Tickers::new(["AAPL", "MSFT"]).await.unwrap();
        let result = tickers.options(None).await.unwrap();

        assert!(result.success_count() > 0);

        // Verify options structure
        for opts in result.options.values() {
            assert!(!opts.expiration_dates().is_empty());
        }
    }

    #[tokio::test]
    #[ignore = "requires network access"]
    #[cfg(feature = "indicators")]
    async fn test_tickers_indicators() {
        let tickers = Tickers::new(["AAPL", "MSFT"]).await.unwrap();
        let result = tickers
            .indicators(Interval::OneDay, TimeRange::ThreeMonths)
            .await
            .unwrap();

        assert!(result.success_count() > 0);

        // Verify indicators structure
        for ind in result.indicators.values() {
            // Check that at least some indicators are present
            assert!(ind.rsi_14.is_some() || ind.sma_20.is_some());
        }
    }

    #[tokio::test]
    async fn test_tickers_add_symbols() {
        let mut tickers = Tickers::new(["AAPL"]).await.unwrap();
        assert_eq!(tickers.len(), 1);
        assert_eq!(tickers.symbols(), &["AAPL"]);

        tickers.add_symbols(&["MSFT", "GOOGL"]);
        assert_eq!(tickers.len(), 3);
        assert!(tickers.symbols().contains(&"AAPL"));
        assert!(tickers.symbols().contains(&"MSFT"));
        assert!(tickers.symbols().contains(&"GOOGL"));

        // Adding duplicate shouldn't increase count
        tickers.add_symbols(&["AAPL"]);
        assert_eq!(tickers.len(), 3);
    }

    #[tokio::test]
    #[ignore = "requires network access"]
    async fn test_tickers_remove_symbols() {
        let mut tickers = Tickers::new(["AAPL", "MSFT", "GOOGL"]).await.unwrap();
        assert_eq!(tickers.len(), 3);

        // Fetch some data to populate caches
        let _ = tickers.quotes().await;

        // Remove one symbol
        tickers.remove_symbols(&["MSFT"]).await;
        assert_eq!(tickers.len(), 2);
        assert!(tickers.symbols().contains(&"AAPL"));
        assert!(!tickers.symbols().contains(&"MSFT"));
        assert!(tickers.symbols().contains(&"GOOGL"));

        // Verify cache was cleared
        let quotes = tickers.quotes().await.unwrap();
        assert!(!quotes.quotes.contains_key("MSFT"));
        assert_eq!(quotes.quotes.len(), 2);
    }
}
