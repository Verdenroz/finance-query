//! Tickers implementation for batch operations on multiple symbols.
//!
//! Optimizes data fetching by using batch endpoints and concurrent requests.

use crate::client::{ClientConfig, YahooClient};
use crate::constants::{Interval, TimeRange};
use crate::error::{Result, YahooError};
use crate::models::chart::Chart;
use crate::models::chart::response::ChartResponse;
use crate::models::chart::result::ChartResult;
use crate::models::quote::Quote;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;

/// Cache key for chart data: (symbol, interval, range)
type ChartCacheKey = (String, Interval, TimeRange);

/// Chart cache type
type ChartCache = Arc<RwLock<HashMap<ChartCacheKey, ChartResult>>>;

/// Quote cache type
type QuoteCache = Arc<RwLock<HashMap<String, Quote>>>;

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

    /// Set the country (automatically sets correct lang and region)
    pub fn country(mut self, country: crate::constants::Country) -> Self {
        self.config.lang = country.lang().to_string();
        self.config.region = country.region().to_string();
        self
    }

    /// Set the language code (e.g., "en-US", "ja-JP", "de-DE")
    pub fn lang(mut self, lang: impl Into<String>) -> Self {
        self.config.lang = lang.into();
        self
    }

    /// Set the region code (e.g., "US", "JP", "DE")
    pub fn region(mut self, region: impl Into<String>) -> Self {
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
///     println!("{}: ${:.2}", symbol, quote.regular_market_price.unwrap_or(0.0));
/// }
///
/// // Fetch charts concurrently
/// use finance_query::{Interval, TimeRange};
/// let charts = tickers.charts(Interval::Daily, TimeRange::OneMonth).await?;
/// # Ok(())
/// # }
/// ```
pub struct Tickers {
    symbols: Vec<String>,
    client: Arc<YahooClient>,
    quote_cache: QuoteCache,
    chart_cache: ChartCache,
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

        for (symbol, result) in results {
            match result {
                Ok(json) => match ChartResponse::from_json(json) {
                    Ok(chart_response) => {
                        if let Some(mut chart_results) = chart_response.chart.result {
                            if let Some(chart_result) = chart_results.pop() {
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

    /// Clear all caches
    pub async fn clear_cache(&self) {
        self.quote_cache.write().await.clear();
        self.chart_cache.write().await.clear();
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
}
