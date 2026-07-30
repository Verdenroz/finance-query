//! Tickers implementation for batch operations on multiple symbols.
//!
//! Optimizes data fetching by using batch endpoints and concurrent requests.

use super::macros::define_batch_response;
use crate::adapters::yahoo::client::ClientConfig;
use crate::constants::{Frequency, Interval, Region, StatementType, TimeRange};
use crate::error::Result;
#[cfg(any(feature = "backtesting", feature = "indicators"))]
use crate::indicators;
use crate::models::chart::events::ChartEvents;
use crate::models::chart::spark::Spark;
use crate::models::chart::{CapitalGain, Chart, Dividend, Split};
use crate::models::corporate::news::News;
use crate::models::corporate::recommendation::Recommendation;
use crate::models::fundamentals::FinancialStatement;
use crate::models::options::Options;
use crate::models::quote::Quote;
use crate::providers::yahoo::YahooProvider;
use crate::providers::{Fetch, Provider, ProviderAdapter, ProviderSet, Routes, build_providers};
use crate::ticker::ClientHandle;
use crate::utils::{CacheEntry, EVICTION_THRESHOLD};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;

#[cfg(any(feature = "backtesting", feature = "indicators"))]
mod analysis;
mod charts;
mod corporate;
mod fundamentals;
mod membership;
mod quotes;

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
type IndicatorsCache = MapCache<(Arc<str>, Interval, TimeRange), indicators::IndicatorsSummary>;

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
    BatchIndicatorsResponse => indicators: indicators::IndicatorsSummary
}

/// Default maximum concurrent requests for batch operations.
const DEFAULT_MAX_CONCURRENCY: usize = 10;

/// Builder for Tickers
pub struct TickersBuilder {
    symbols: Vec<Arc<str>>,
    config: ClientConfig,
    shared_client: Option<ClientHandle>,
    injected_providers: Option<Arc<ProviderSet>>,
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
            injected_providers: None,
            max_concurrency: DEFAULT_MAX_CONCURRENCY,
            cache_ttl: None,
            include_logo: false,
        }
    }

    /// Set the region (automatically sets correct lang and region code)
    pub fn region(mut self, region: Region) -> Self {
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

    #[allow(dead_code)]
    pub(crate) fn config(mut self, config: ClientConfig) -> Self {
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

    /// Pre-inject a shared provider set (used by [`Providers::tickers`]).
    pub(crate) fn with_provider_set(mut self, set: Arc<ProviderSet>) -> Self {
        self.injected_providers = Some(set);
        self
    }

    /// Share an existing authenticated session instead of creating a new one.
    ///
    /// Avoids redundant auth handshakes when combining `Tickers` with other
    /// `Ticker` instances. Obtain a handle from any existing `Ticker` or
    /// `Tickers` via `.client_handle()`.
    pub fn client(mut self, handle: ClientHandle) -> Self {
        self.shared_client = Some(handle);
        self
    }

    /// Build the Tickers instance
    pub async fn build(self) -> Result<Tickers> {
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

        Ok(Tickers {
            symbols: self.symbols,
            providers,
            max_concurrency: self.max_concurrency,
            cache_ttl: self.cache_ttl,
            include_logo: self.include_logo,
            #[cfg(feature = "translation")]
            translate_lang,
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
    providers: Arc<ProviderSet>,
    max_concurrency: usize,
    cache_ttl: Option<Duration>,
    include_logo: bool,
    #[cfg(feature = "translation")]
    translate_lang: Option<crate::translation::Lang>,
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

    /// Returns a handle to the underlying Yahoo Finance session.
    ///
    /// Pass to [`Ticker::builder`](crate::Ticker::builder) or other
    /// [`Tickers::builder`] calls via `.client(handle)` to share the
    /// authenticated session without a new auth handshake.
    ///
    /// # Panics
    ///
    /// Panics if these tickers were created via [`Providers`](crate::Providers) with
    /// no Yahoo provider configured. For session sharing across multiple tickers,
    /// prefer [`Providers::tickers`](crate::Providers::tickers) instead.
    pub fn client_handle(&self) -> ClientHandle {
        ClientHandle(
            self.providers
                .first_yahoo()
                .expect("Tickers always uses a Yahoo session"),
        )
    }

    /// Returns `true` if a cache entry exists and has not exceeded the TTL.
    #[inline]
    fn is_cache_fresh<T>(&self, entry: Option<&CacheEntry<T>>) -> bool {
        CacheEntry::is_fresh_with_ttl(entry, self.cache_ttl)
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
}
