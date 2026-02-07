//! Tickers implementation for batch operations on multiple symbols.
//!
//! Optimizes data fetching by using batch endpoints and concurrent requests.

use crate::client::{ClientConfig, YahooClient};
use crate::constants::{Frequency, Interval, StatementType, TimeRange};
use crate::error::{Result, YahooError};
use crate::models::chart::events::ChartEvents;
use crate::models::chart::response::ChartResponse;
use crate::models::chart::result::ChartResult;
use crate::models::chart::{CapitalGain, Chart, Dividend, Split};
use crate::models::financials::FinancialStatement;
use crate::models::news::News;
use crate::models::options::Options;
use crate::models::quote::Quote;
use crate::models::recommendation::Recommendation;
use crate::models::spark::Spark;
use crate::models::spark::response::SparkResponse;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tokio::sync::RwLock;

/// Cache key for chart data: (symbol, interval, range)
type ChartCacheKey = (String, Interval, TimeRange);

/// Chart cache type
type ChartCache = Arc<RwLock<HashMap<ChartCacheKey, ChartResult>>>;

/// Quote cache type
type QuoteCache = Arc<RwLock<HashMap<String, Quote>>>;

/// Events cache type (events are cached per symbol, fetched once with chart data)
type EventsCache = Arc<RwLock<HashMap<String, ChartEvents>>>;

/// Financials cache type (cached per (symbol, statement_type, frequency))
type FinancialsCache = Arc<RwLock<HashMap<(String, StatementType, Frequency), FinancialStatement>>>;

/// News cache type (cached per symbol)
type NewsCache = Arc<RwLock<HashMap<String, Vec<News>>>>;

/// Recommendations cache type (cached per symbol)
type RecommendationsCache = Arc<RwLock<HashMap<String, Recommendation>>>;

/// Options cache type (cached per (symbol, date))
type OptionsCache = Arc<RwLock<HashMap<(String, Option<i64>), Options>>>;

/// Indicators cache type (cached per (symbol, interval, range))
#[cfg(feature = "indicators")]
type IndicatorsCache =
    Arc<RwLock<HashMap<(String, Interval, TimeRange), crate::indicators::IndicatorsSummary>>>;

/// Response containing quotes for multiple symbols.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct BatchQuotesResponse {
    /// Successfully fetched quotes, keyed by symbol
    pub quotes: HashMap<String, Quote>,
    /// Symbols that failed to fetch, with error messages
    pub errors: HashMap<String, String>,
}

impl BatchQuotesResponse {
    pub(crate) fn new() -> Self {
        Self {
            quotes: HashMap::new(),
            errors: HashMap::new(),
        }
    }

    /// Number of successfully fetched quotes
    pub fn success_count(&self) -> usize {
        self.quotes.len()
    }

    /// Number of failed symbols
    pub fn error_count(&self) -> usize {
        self.errors.len()
    }

    /// Check if all symbols were successful
    pub fn all_successful(&self) -> bool {
        self.errors.is_empty()
    }
}

/// Response containing charts for multiple symbols.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct BatchChartsResponse {
    /// Successfully fetched charts, keyed by symbol
    pub charts: HashMap<String, Chart>,
    /// Symbols that failed to fetch, with error messages
    pub errors: HashMap<String, String>,
}

impl BatchChartsResponse {
    pub(crate) fn new() -> Self {
        Self {
            charts: HashMap::new(),
            errors: HashMap::new(),
        }
    }

    /// Number of successfully fetched charts
    pub fn success_count(&self) -> usize {
        self.charts.len()
    }

    /// Number of failed symbols
    pub fn error_count(&self) -> usize {
        self.errors.len()
    }

    /// Check if all symbols were successful
    pub fn all_successful(&self) -> bool {
        self.errors.is_empty()
    }
}

/// Response containing spark data for multiple symbols.
///
/// Spark data is optimized for sparkline rendering with only close prices.
/// Unlike charts, spark data is fetched in a single batch request.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct BatchSparksResponse {
    /// Successfully fetched sparks, keyed by symbol
    pub sparks: HashMap<String, Spark>,
    /// Symbols that failed to fetch, with error messages
    pub errors: HashMap<String, String>,
}

impl BatchSparksResponse {
    pub(crate) fn new() -> Self {
        Self {
            sparks: HashMap::new(),
            errors: HashMap::new(),
        }
    }

    /// Number of successfully fetched sparks
    pub fn success_count(&self) -> usize {
        self.sparks.len()
    }

    /// Number of failed symbols
    pub fn error_count(&self) -> usize {
        self.errors.len()
    }

    /// Check if all symbols were successful
    pub fn all_successful(&self) -> bool {
        self.errors.is_empty()
    }
}

/// Response containing dividends for multiple symbols.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct BatchDividendsResponse {
    /// Successfully fetched dividends, keyed by symbol
    pub dividends: HashMap<String, Vec<Dividend>>,
    /// Symbols that failed to fetch, with error messages
    pub errors: HashMap<String, String>,
}

impl BatchDividendsResponse {
    pub(crate) fn new() -> Self {
        Self {
            dividends: HashMap::new(),
            errors: HashMap::new(),
        }
    }

    /// Number of successfully fetched dividend lists
    pub fn success_count(&self) -> usize {
        self.dividends.len()
    }

    /// Number of failed symbols
    pub fn error_count(&self) -> usize {
        self.errors.len()
    }

    /// Check if all symbols were successful
    pub fn all_successful(&self) -> bool {
        self.errors.is_empty()
    }
}

/// Response containing splits for multiple symbols.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct BatchSplitsResponse {
    /// Successfully fetched splits, keyed by symbol
    pub splits: HashMap<String, Vec<Split>>,
    /// Symbols that failed to fetch, with error messages
    pub errors: HashMap<String, String>,
}

impl BatchSplitsResponse {
    pub(crate) fn new() -> Self {
        Self {
            splits: HashMap::new(),
            errors: HashMap::new(),
        }
    }

    /// Number of successfully fetched split lists
    pub fn success_count(&self) -> usize {
        self.splits.len()
    }

    /// Number of failed symbols
    pub fn error_count(&self) -> usize {
        self.errors.len()
    }

    /// Check if all symbols were successful
    pub fn all_successful(&self) -> bool {
        self.errors.is_empty()
    }
}

/// Response containing capital gains for multiple symbols.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct BatchCapitalGainsResponse {
    /// Successfully fetched capital gains, keyed by symbol
    pub capital_gains: HashMap<String, Vec<CapitalGain>>,
    /// Symbols that failed to fetch, with error messages
    pub errors: HashMap<String, String>,
}

impl BatchCapitalGainsResponse {
    pub(crate) fn new() -> Self {
        Self {
            capital_gains: HashMap::new(),
            errors: HashMap::new(),
        }
    }

    /// Number of successfully fetched capital gain lists
    pub fn success_count(&self) -> usize {
        self.capital_gains.len()
    }

    /// Number of failed symbols
    pub fn error_count(&self) -> usize {
        self.errors.len()
    }

    /// Check if all symbols were successful
    pub fn all_successful(&self) -> bool {
        self.errors.is_empty()
    }
}

/// Response containing financial statements for multiple symbols.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct BatchFinancialsResponse {
    /// Successfully fetched financial statements, keyed by symbol
    pub financials: HashMap<String, FinancialStatement>,
    /// Symbols that failed to fetch, with error messages
    pub errors: HashMap<String, String>,
}

impl BatchFinancialsResponse {
    pub(crate) fn new() -> Self {
        Self {
            financials: HashMap::new(),
            errors: HashMap::new(),
        }
    }

    /// Number of successfully fetched financial statements
    pub fn success_count(&self) -> usize {
        self.financials.len()
    }

    /// Number of failed symbols
    pub fn error_count(&self) -> usize {
        self.errors.len()
    }

    /// Check if all symbols were successful
    pub fn all_successful(&self) -> bool {
        self.errors.is_empty()
    }
}

/// Response containing news articles for multiple symbols.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct BatchNewsResponse {
    /// Successfully fetched news, keyed by symbol
    pub news: HashMap<String, Vec<News>>,
    /// Symbols that failed to fetch, with error messages
    pub errors: HashMap<String, String>,
}

impl BatchNewsResponse {
    pub(crate) fn new() -> Self {
        Self {
            news: HashMap::new(),
            errors: HashMap::new(),
        }
    }

    /// Number of successfully fetched news lists
    pub fn success_count(&self) -> usize {
        self.news.len()
    }

    /// Number of failed symbols
    pub fn error_count(&self) -> usize {
        self.errors.len()
    }

    /// Check if all symbols were successful
    pub fn all_successful(&self) -> bool {
        self.errors.is_empty()
    }
}

/// Response containing recommendations for multiple symbols.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct BatchRecommendationsResponse {
    /// Successfully fetched recommendations, keyed by symbol
    pub recommendations: HashMap<String, Recommendation>,
    /// Symbols that failed to fetch, with error messages
    pub errors: HashMap<String, String>,
}

impl BatchRecommendationsResponse {
    pub(crate) fn new() -> Self {
        Self {
            recommendations: HashMap::new(),
            errors: HashMap::new(),
        }
    }

    /// Number of successfully fetched recommendations
    pub fn success_count(&self) -> usize {
        self.recommendations.len()
    }

    /// Number of failed symbols
    pub fn error_count(&self) -> usize {
        self.errors.len()
    }

    /// Check if all symbols were successful
    pub fn all_successful(&self) -> bool {
        self.errors.is_empty()
    }
}

/// Response containing options chains for multiple symbols.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct BatchOptionsResponse {
    /// Successfully fetched options, keyed by symbol
    pub options: HashMap<String, Options>,
    /// Symbols that failed to fetch, with error messages
    pub errors: HashMap<String, String>,
}

impl BatchOptionsResponse {
    pub(crate) fn new() -> Self {
        Self {
            options: HashMap::new(),
            errors: HashMap::new(),
        }
    }

    /// Number of successfully fetched options
    pub fn success_count(&self) -> usize {
        self.options.len()
    }

    /// Number of failed symbols
    pub fn error_count(&self) -> usize {
        self.errors.len()
    }

    /// Check if all symbols were successful
    pub fn all_successful(&self) -> bool {
        self.errors.is_empty()
    }
}

/// Response containing technical indicators for multiple symbols.
#[cfg(feature = "indicators")]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct BatchIndicatorsResponse {
    /// Successfully fetched indicators, keyed by symbol
    pub indicators: HashMap<String, crate::indicators::IndicatorsSummary>,
    /// Symbols that failed to fetch, with error messages
    pub errors: HashMap<String, String>,
}

#[cfg(feature = "indicators")]
impl BatchIndicatorsResponse {
    pub(crate) fn new() -> Self {
        Self {
            indicators: HashMap::new(),
            errors: HashMap::new(),
        }
    }

    /// Number of successfully fetched indicator summaries
    pub fn success_count(&self) -> usize {
        self.indicators.len()
    }

    /// Number of failed symbols
    pub fn error_count(&self) -> usize {
        self.errors.len()
    }

    /// Check if all symbols were successful
    pub fn all_successful(&self) -> bool {
        self.errors.is_empty()
    }
}

/// Builder for Tickers
pub struct TickersBuilder {
    symbols: Vec<String>,
    config: ClientConfig,
}

impl TickersBuilder {
    fn new<S, I>(symbols: I) -> Self
    where
        S: Into<String>,
        I: IntoIterator<Item = S>,
    {
        Self {
            symbols: symbols.into_iter().map(|s| s.into()).collect(),
            config: ClientConfig::default(),
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

    /// Build the Tickers instance
    pub async fn build(self) -> Result<Tickers> {
        let client = Arc::new(YahooClient::new(self.config).await?);

        Ok(Tickers {
            symbols: self.symbols,
            client,
            quote_cache: Arc::new(RwLock::new(HashMap::new())),
            chart_cache: Arc::new(RwLock::new(HashMap::new())),
            events_cache: Arc::new(RwLock::new(HashMap::new())),
            financials_cache: Arc::new(RwLock::new(HashMap::new())),
            news_cache: Arc::new(RwLock::new(HashMap::new())),
            recommendations_cache: Arc::new(RwLock::new(HashMap::new())),
            options_cache: Arc::new(RwLock::new(HashMap::new())),
            #[cfg(feature = "indicators")]
            indicators_cache: Arc::new(RwLock::new(HashMap::new())),
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
/// let quotes = tickers.quotes(false).await?;
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
    symbols: Vec<String>,
    client: Arc<YahooClient>,
    quote_cache: QuoteCache,
    chart_cache: ChartCache,
    events_cache: EventsCache,
    financials_cache: FinancialsCache,
    news_cache: NewsCache,
    recommendations_cache: RecommendationsCache,
    options_cache: OptionsCache,
    #[cfg(feature = "indicators")]
    indicators_cache: IndicatorsCache,
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
    pub fn symbols(&self) -> &[String] {
        &self.symbols
    }

    /// Number of symbols
    pub fn len(&self) -> usize {
        self.symbols.len()
    }

    /// Check if empty
    pub fn is_empty(&self) -> bool {
        self.symbols.is_empty()
    }

    /// Batch fetch quotes for all symbols
    ///
    /// Uses /v7/finance/quote endpoint - fetches all symbols in a single API call.
    /// When `include_logo` is true, makes a parallel call for logo URLs.
    ///
    /// # Arguments
    ///
    /// * `include_logo` - Whether to fetch company logo URLs
    pub async fn quotes(&self, include_logo: bool) -> Result<BatchQuotesResponse> {
        // Check cache
        {
            let cache = self.quote_cache.read().await;
            if self.symbols.iter().all(|s| cache.contains_key(s)) {
                let mut response = BatchQuotesResponse::new();
                for symbol in &self.symbols {
                    if let Some(quote) = cache.get(symbol) {
                        response.quotes.insert(symbol.clone(), quote.clone());
                    }
                }
                return Ok(response);
            }
        }

        // Fetch batch quotes
        let symbols_ref: Vec<&str> = self.symbols.iter().map(|s| s.as_str()).collect();

        // Yahoo requires separate calls for quotes vs logos
        // When include_logo=true, fetch both in parallel
        let (json, logos) = if include_logo {
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
        let mut response = BatchQuotesResponse::new();

        if let Some(quote_response) = json.get("quoteResponse") {
            if let Some(results) = quote_response.get("result").and_then(|r| r.as_array()) {
                let mut cache = self.quote_cache.write().await;

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
                                cache.insert(symbol.to_string(), quote.clone());
                                response.quotes.insert(symbol.to_string(), quote);
                            }
                            Err(e) => {
                                response.errors.insert(symbol.to_string(), e.to_string());
                            }
                        }
                    }
                }
            }

            // Track missing symbols
            for symbol in &self.symbols {
                if !response.quotes.contains_key(symbol) && !response.errors.contains_key(symbol) {
                    response
                        .errors
                        .insert(symbol.clone(), "Symbol not found in response".to_string());
                }
            }
        }

        Ok(response)
    }

    /// Get a specific quote by symbol (from cache or fetch all)
    pub async fn quote(&self, symbol: &str, include_logo: bool) -> Result<Quote> {
        {
            let cache = self.quote_cache.read().await;
            if let Some(quote) = cache.get(symbol) {
                return Ok(quote.clone());
            }
        }

        let response = self.quotes(include_logo).await?;

        response
            .quotes
            .get(symbol)
            .cloned()
            .ok_or_else(|| YahooError::SymbolNotFound {
                symbol: Some(symbol.to_string()),
                context: response
                    .errors
                    .get(symbol)
                    .cloned()
                    .unwrap_or_else(|| "Symbol not found".to_string()),
            })
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
        // Check cache
        {
            let cache = self.chart_cache.read().await;
            if self
                .symbols
                .iter()
                .all(|s| cache.contains_key(&(s.clone(), interval, range)))
            {
                let mut response = BatchChartsResponse::new();
                for symbol in &self.symbols {
                    if let Some(result) = cache.get(&(symbol.clone(), interval, range)) {
                        response.charts.insert(
                            symbol.clone(),
                            Chart {
                                symbol: symbol.clone(),
                                meta: result.meta.clone(),
                                candles: result.to_candles(),
                                interval: Some(interval.as_str().to_string()),
                                range: Some(range.as_str().to_string()),
                            },
                        );
                    }
                }
                return Ok(response);
            }
        }

        // Fetch all charts concurrently
        let futures: Vec<_> = self
            .symbols
            .iter()
            .map(|symbol| {
                let client = Arc::clone(&self.client);
                let symbol = symbol.clone();
                async move {
                    let result = client.get_chart(&symbol, interval, range).await;
                    (symbol, result)
                }
            })
            .collect();

        let results = futures::future::join_all(futures).await;

        let mut response = BatchChartsResponse::new();
        let mut cache = self.chart_cache.write().await;
        let mut events_cache = self.events_cache.write().await;

        for (symbol, result) in results {
            match result {
                Ok(json) => match ChartResponse::from_json(json) {
                    Ok(chart_response) => {
                        if let Some(mut chart_results) = chart_response.chart.result {
                            if let Some(chart_result) = chart_results.pop() {
                                // Cache events if present and not already cached
                                if let Some(events) = &chart_result.events {
                                    events_cache
                                        .entry(symbol.clone())
                                        .or_insert_with(|| events.clone());
                                }

                                let chart = Chart {
                                    symbol: symbol.clone(),
                                    meta: chart_result.meta.clone(),
                                    candles: chart_result.to_candles(),
                                    interval: Some(interval.as_str().to_string()),
                                    range: Some(range.as_str().to_string()),
                                };
                                cache.insert((symbol.clone(), interval, range), chart_result);
                                response.charts.insert(symbol, chart);
                            } else {
                                response
                                    .errors
                                    .insert(symbol, "Empty chart response".to_string());
                            }
                        } else {
                            response
                                .errors
                                .insert(symbol, "No chart data in response".to_string());
                        }
                    }
                    Err(e) => {
                        response.errors.insert(symbol, e.to_string());
                    }
                },
                Err(e) => {
                    response.errors.insert(symbol, e.to_string());
                }
            }
        }

        Ok(response)
    }

    /// Get a specific chart by symbol
    pub async fn chart(&self, symbol: &str, interval: Interval, range: TimeRange) -> Result<Chart> {
        {
            let cache = self.chart_cache.read().await;
            if let Some(result) = cache.get(&(symbol.to_string(), interval, range)) {
                return Ok(Chart {
                    symbol: symbol.to_string(),
                    meta: result.meta.clone(),
                    candles: result.to_candles(),
                    interval: Some(interval.as_str().to_string()),
                    range: Some(range.as_str().to_string()),
                });
            }
        }

        let response = self.charts(interval, range).await?;

        response
            .charts
            .get(symbol)
            .cloned()
            .ok_or_else(|| YahooError::SymbolNotFound {
                symbol: Some(symbol.to_string()),
                context: response
                    .errors
                    .get(symbol)
                    .cloned()
                    .unwrap_or_else(|| "Symbol not found".to_string()),
            })
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
        let symbols_ref: Vec<&str> = self.symbols.iter().map(|s| s.as_str()).collect();

        let json =
            crate::endpoints::spark::fetch(&self.client, &symbols_ref, interval, range).await?;

        let mut response = BatchSparksResponse::new();

        match SparkResponse::from_json(json) {
            Ok(spark_response) => {
                if let Some(results) = spark_response.spark.result {
                    for result in &results {
                        if let Some(spark) = Spark::from_response(
                            result,
                            Some(interval.as_str().to_string()),
                            Some(range.as_str().to_string()),
                        ) {
                            response.sparks.insert(result.symbol.clone(), spark);
                        } else {
                            response.errors.insert(
                                result.symbol.clone(),
                                "Failed to parse spark data".to_string(),
                            );
                        }
                    }
                }

                // Track missing symbols
                for symbol in &self.symbols {
                    if !response.sparks.contains_key(symbol)
                        && !response.errors.contains_key(symbol)
                    {
                        response
                            .errors
                            .insert(symbol.clone(), "Symbol not found in response".to_string());
                    }
                }
            }
            Err(e) => {
                // If parsing failed entirely, mark all symbols as errored
                for symbol in &self.symbols {
                    response.errors.insert(symbol.clone(), e.to_string());
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
        let mut response = BatchDividendsResponse::new();

        // Ensure charts are fetched for all symbols (to populate events cache)
        let _ = self.charts(Interval::OneDay, TimeRange::Max).await;

        let events_cache = self.events_cache.read().await;

        for symbol in &self.symbols {
            if let Some(events) = events_cache.get(symbol) {
                let all_dividends = events.to_dividends();
                let filtered = filter_by_range(all_dividends, range);
                response.dividends.insert(symbol.clone(), filtered);
            } else {
                response
                    .errors
                    .insert(symbol.clone(), "No events data available".to_string());
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
        let mut response = BatchSplitsResponse::new();

        // Ensure charts are fetched for all symbols (to populate events cache)
        let _ = self.charts(Interval::OneDay, TimeRange::Max).await;

        let events_cache = self.events_cache.read().await;

        for symbol in &self.symbols {
            if let Some(events) = events_cache.get(symbol) {
                let all_splits = events.to_splits();
                let filtered = filter_by_range(all_splits, range);
                response.splits.insert(symbol.clone(), filtered);
            } else {
                response
                    .errors
                    .insert(symbol.clone(), "No events data available".to_string());
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
        let mut response = BatchCapitalGainsResponse::new();

        // Ensure charts are fetched for all symbols (to populate events cache)
        let _ = self.charts(Interval::OneDay, TimeRange::Max).await;

        let events_cache = self.events_cache.read().await;

        for symbol in &self.symbols {
            if let Some(events) = events_cache.get(symbol) {
                let all_gains = events.to_capital_gains();
                let filtered = filter_by_range(all_gains, range);
                response.capital_gains.insert(symbol.clone(), filtered);
            } else {
                response
                    .errors
                    .insert(symbol.clone(), "No events data available".to_string());
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
        let cache_key_for = |symbol: &String| (symbol.clone(), statement_type, frequency);

        // Check if all symbols are cached
        {
            let cache = self.financials_cache.read().await;
            if self
                .symbols
                .iter()
                .all(|s| cache.contains_key(&cache_key_for(s)))
            {
                let mut response = BatchFinancialsResponse::new();
                for symbol in &self.symbols {
                    if let Some(stmt) = cache.get(&cache_key_for(symbol)) {
                        response.financials.insert(symbol.clone(), stmt.clone());
                    }
                }
                return Ok(response);
            }
        }

        // Fetch all financials concurrently
        let futures: Vec<_> = self
            .symbols
            .iter()
            .map(|symbol| {
                let client = Arc::clone(&self.client);
                let symbol = symbol.clone();
                async move {
                    let result = client
                        .get_financials(&symbol, statement_type, frequency)
                        .await;
                    (symbol, result)
                }
            })
            .collect();

        let results = futures::future::join_all(futures).await;

        let mut response = BatchFinancialsResponse::new();
        let mut cache = self.financials_cache.write().await;

        for (symbol, result) in results {
            match result {
                Ok(stmt) => {
                    cache.insert(cache_key_for(&symbol), stmt.clone());
                    response.financials.insert(symbol, stmt);
                }
                Err(e) => {
                    response.errors.insert(symbol, e.to_string());
                }
            }
        }

        Ok(response)
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
        // Check if all symbols are cached
        {
            let cache = self.news_cache.read().await;
            if self.symbols.iter().all(|s| cache.contains_key(s)) {
                let mut response = BatchNewsResponse::new();
                for symbol in &self.symbols {
                    if let Some(articles) = cache.get(symbol) {
                        response.news.insert(symbol.clone(), articles.clone());
                    }
                }
                return Ok(response);
            }
        }

        // Fetch news for all symbols concurrently using scraper
        let futures: Vec<_> = self
            .symbols
            .iter()
            .map(|symbol| {
                let symbol = symbol.clone();
                async move {
                    let result = crate::scrapers::stockanalysis::scrape_symbol_news(&symbol).await;
                    (symbol, result)
                }
            })
            .collect();

        let results = futures::future::join_all(futures).await;

        let mut response = BatchNewsResponse::new();
        let mut cache = self.news_cache.write().await;

        for (symbol, result) in results {
            match result {
                Ok(articles) => {
                    cache.insert(symbol.clone(), articles.clone());
                    response.news.insert(symbol, articles);
                }
                Err(e) => {
                    response.errors.insert(symbol, e.to_string());
                }
            }
        }

        Ok(response)
    }

    /// Batch fetch recommendations for all symbols
    ///
    /// Fetches analyst recommendations and similar stocks for all symbols concurrently.
    /// Recommendations are cached per symbol.
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
        // Check if all symbols are cached
        {
            let cache = self.recommendations_cache.read().await;
            if self.symbols.iter().all(|s| cache.contains_key(s)) {
                let mut response = BatchRecommendationsResponse::new();
                for symbol in &self.symbols {
                    if let Some(rec) = cache.get(symbol) {
                        response.recommendations.insert(symbol.clone(), rec.clone());
                    }
                }
                return Ok(response);
            }
        }

        // Fetch recommendations for all symbols concurrently
        let futures: Vec<_> = self
            .symbols
            .iter()
            .map(|symbol| {
                let client = Arc::clone(&self.client);
                let symbol = symbol.clone();
                async move {
                    let result = client.get_recommendations(&symbol, limit).await;
                    (symbol, result)
                }
            })
            .collect();

        let results = futures::future::join_all(futures).await;

        let mut response = BatchRecommendationsResponse::new();
        let mut cache = self.recommendations_cache.write().await;

        for (symbol, result) in results {
            match result {
                Ok(json) => {
                    match crate::models::recommendation::response::RecommendationResponse::from_json(
                        json,
                    ) {
                        Ok(rec_response) => {
                            let rec = Recommendation {
                                symbol: symbol.clone(),
                                recommendations: rec_response
                                    .finance
                                    .result
                                    .iter()
                                    .flat_map(|r| &r.recommended_symbols)
                                    .cloned()
                                    .collect(),
                            };
                            cache.insert(symbol.clone(), rec.clone());
                            response.recommendations.insert(symbol, rec);
                        }
                        Err(e) => {
                            response.errors.insert(symbol, e.to_string());
                        }
                    }
                }
                Err(e) => {
                    response.errors.insert(symbol, e.to_string());
                }
            }
        }

        Ok(response)
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
        let cache_key_for = |symbol: &String| (symbol.clone(), date);

        // Check if all symbols are cached
        {
            let cache = self.options_cache.read().await;
            if self
                .symbols
                .iter()
                .all(|s| cache.contains_key(&cache_key_for(s)))
            {
                let mut response = BatchOptionsResponse::new();
                for symbol in &self.symbols {
                    if let Some(opts) = cache.get(&cache_key_for(symbol)) {
                        response.options.insert(symbol.clone(), opts.clone());
                    }
                }
                return Ok(response);
            }
        }

        // Fetch options for all symbols concurrently
        let futures: Vec<_> = self
            .symbols
            .iter()
            .map(|symbol| {
                let client = Arc::clone(&self.client);
                let symbol = symbol.clone();
                async move {
                    let result = client.get_options(&symbol, date).await;
                    (symbol, result)
                }
            })
            .collect();

        let results = futures::future::join_all(futures).await;

        let mut response = BatchOptionsResponse::new();
        let mut cache = self.options_cache.write().await;

        for (symbol, result) in results {
            match result {
                Ok(json) => match serde_json::from_value::<Options>(json) {
                    Ok(opts) => {
                        cache.insert(cache_key_for(&symbol), opts.clone());
                        response.options.insert(symbol, opts);
                    }
                    Err(e) => {
                        response.errors.insert(symbol, e.to_string());
                    }
                },
                Err(e) => {
                    response.errors.insert(symbol, e.to_string());
                }
            }
        }

        Ok(response)
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
        let cache_key_for = |symbol: &String| (symbol.clone(), interval, range);

        // Check if all symbols are cached
        {
            let cache = self.indicators_cache.read().await;
            if self
                .symbols
                .iter()
                .all(|s| cache.contains_key(&cache_key_for(s)))
            {
                let mut response = BatchIndicatorsResponse::new();
                for symbol in &self.symbols {
                    if let Some(ind) = cache.get(&cache_key_for(symbol)) {
                        response.indicators.insert(symbol.clone(), ind.clone());
                    }
                }
                return Ok(response);
            }
        }

        // Fetch charts first (which may already be cached)
        let charts_response = self.charts(interval, range).await?;

        let mut response = BatchIndicatorsResponse::new();
        let mut cache = self.indicators_cache.write().await;

        for (symbol, chart) in &charts_response.charts {
            let indicators = crate::indicators::summary::calculate_indicators(&chart.candles);
            cache.insert(cache_key_for(symbol), indicators.clone());
            response.indicators.insert(symbol.clone(), indicators);
        }

        // Add errors from chart fetch
        for (symbol, error) in &charts_response.errors {
            response.errors.insert(symbol.clone(), error.clone());
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
        for symbol in symbols {
            let symbol_str = symbol.as_ref().to_string();
            if !self.symbols.contains(&symbol_str) {
                self.symbols.push(symbol_str);
            }
        }
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
        let symbols_to_remove: Vec<String> =
            symbols.iter().map(|s| s.as_ref().to_string()).collect();

        // Remove from symbol list
        self.symbols.retain(|s| !symbols_to_remove.contains(s));

        // Clear caches for removed symbols
        {
            let mut quote_cache = self.quote_cache.write().await;
            for symbol in &symbols_to_remove {
                quote_cache.remove(symbol);
            }
        }

        {
            let mut chart_cache = self.chart_cache.write().await;
            chart_cache.retain(|(sym, _, _), _| !symbols_to_remove.contains(sym));
        }

        {
            let mut events_cache = self.events_cache.write().await;
            for symbol in &symbols_to_remove {
                events_cache.remove(symbol);
            }
        }

        {
            let mut financials_cache = self.financials_cache.write().await;
            financials_cache.retain(|(sym, _, _), _| !symbols_to_remove.contains(sym));
        }

        {
            let mut news_cache = self.news_cache.write().await;
            for symbol in &symbols_to_remove {
                news_cache.remove(symbol);
            }
        }

        {
            let mut recommendations_cache = self.recommendations_cache.write().await;
            for symbol in &symbols_to_remove {
                recommendations_cache.remove(symbol);
            }
        }

        {
            let mut options_cache = self.options_cache.write().await;
            options_cache.retain(|(sym, _), _| !symbols_to_remove.contains(sym));
        }

        #[cfg(feature = "indicators")]
        {
            let mut indicators_cache = self.indicators_cache.write().await;
            indicators_cache.retain(|(sym, _, _), _| !symbols_to_remove.contains(sym));
        }
    }

    /// Clear all caches
    pub async fn clear_cache(&self) {
        self.quote_cache.write().await.clear();
        self.chart_cache.write().await.clear();
        self.events_cache.write().await.clear();
        self.financials_cache.write().await.clear();
        self.news_cache.write().await.clear();
        self.recommendations_cache.write().await.clear();
        self.options_cache.write().await.clear();
        #[cfg(feature = "indicators")]
        self.indicators_cache.write().await.clear();
    }
}

/// Trait for types with a timestamp field
trait HasTimestamp {
    fn timestamp(&self) -> i64;
}

impl HasTimestamp for Dividend {
    fn timestamp(&self) -> i64 {
        self.timestamp
    }
}

impl HasTimestamp for Split {
    fn timestamp(&self) -> i64 {
        self.timestamp
    }
}

impl HasTimestamp for CapitalGain {
    fn timestamp(&self) -> i64 {
        self.timestamp
    }
}

/// Calculate cutoff timestamp for a given time range
fn range_to_cutoff(range: TimeRange) -> i64 {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64;

    const DAY: i64 = 86400;

    match range {
        TimeRange::OneDay => now - DAY,
        TimeRange::FiveDays => now - 5 * DAY,
        TimeRange::OneMonth => now - 30 * DAY,
        TimeRange::ThreeMonths => now - 90 * DAY,
        TimeRange::SixMonths => now - 180 * DAY,
        TimeRange::OneYear => now - 365 * DAY,
        TimeRange::TwoYears => now - 2 * 365 * DAY,
        TimeRange::FiveYears => now - 5 * 365 * DAY,
        TimeRange::TenYears => now - 10 * 365 * DAY,
        TimeRange::YearToDate => {
            let days_in_year = (now % (365 * DAY)) / DAY;
            now - days_in_year * DAY
        }
        TimeRange::Max => 0,
    }
}

/// Filter a list of timestamped items by time range
fn filter_by_range<T: HasTimestamp>(items: Vec<T>, range: TimeRange) -> Vec<T> {
    match range {
        TimeRange::Max => items,
        range => {
            let cutoff = range_to_cutoff(range);
            items
                .into_iter()
                .filter(|item| item.timestamp() >= cutoff)
                .collect()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    #[ignore] // Requires network access
    async fn test_tickers_quotes() {
        let tickers = Tickers::new(["AAPL", "MSFT", "GOOGL"]).await.unwrap();
        let result = tickers.quotes(false).await.unwrap();

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
        assert!(tickers.symbols().contains(&"AAPL".to_string()));
        assert!(tickers.symbols().contains(&"MSFT".to_string()));
        assert!(tickers.symbols().contains(&"GOOGL".to_string()));

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
        let _ = tickers.quotes(false).await;

        // Remove one symbol
        tickers.remove_symbols(&["MSFT"]).await;
        assert_eq!(tickers.len(), 2);
        assert!(tickers.symbols().contains(&"AAPL".to_string()));
        assert!(!tickers.symbols().contains(&"MSFT".to_string()));
        assert!(tickers.symbols().contains(&"GOOGL".to_string()));

        // Verify cache was cleared
        let quotes = tickers.quotes(false).await.unwrap();
        assert!(!quotes.quotes.contains_key("MSFT"));
        assert_eq!(quotes.quotes.len(), 2);
    }
}
