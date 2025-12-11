//! Ticker implementations for accessing symbol-specific data from Yahoo Finance.
//!
//! Provides both async and sync interfaces for fetching quotes, charts, financials, and news.

use super::macros;
use crate::client::{BlockingYahooClient, ClientConfig, YahooClient};
use crate::constants::{Interval, TimeRange};
use crate::error::Result;
use crate::models::chart::{Chart, ChartResponse, ChartResult};
use crate::models::news::NewsResponse;
use crate::models::options::OptionsResponse;
use crate::models::quote::{
    AssetProfile, CalendarEvents, DefaultKeyStatistics, Earnings, EarningsHistory, EarningsTrend,
    FinancialData, FundOwnership, InsiderHolders, InsiderTransactions, InstitutionOwnership,
    MajorHoldersBreakdown, Module, NetSharePurchaseActivity, Price, Quote, QuoteSummaryResponse,
    QuoteTypeData, RecommendationTrend, SecFilings, SummaryDetail, SummaryProfile,
    UpgradeDowngradeHistory,
};
use crate::models::recommendation::{Recommendation, RecommendationResponse};
use crate::models::timeseries::TimeseriesResponse;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

// Core ticker helpers
struct TickerCoreData {
    symbol: String,
}

impl TickerCoreData {
    fn new(symbol: impl Into<String>) -> Self {
        Self {
            symbol: symbol.into(),
        }
    }

    /// Builds the quote summary URL with all modules
    fn build_quote_summary_url(&self) -> String {
        let url = crate::constants::endpoints::quote_summary(&self.symbol);
        let quote_modules = Module::all();
        let module_str = quote_modules
            .iter()
            .map(|m| m.as_str())
            .collect::<Vec<_>>()
            .join(",");
        format!("{}?modules={}", url, module_str)
    }

    /// Parses quote summary response from JSON
    fn parse_quote_summary(&self, json: serde_json::Value) -> Result<QuoteSummaryResponse> {
        QuoteSummaryResponse::from_json(json, &self.symbol)
    }
}

/// Builder for AsyncTicker
///
/// Provides a fluent API for constructing AsyncTicker instances.
pub struct AsyncTickerBuilder {
    symbol: String,
    config: ClientConfig,
}

impl AsyncTickerBuilder {
    fn new(symbol: impl Into<String>) -> Self {
        Self {
            symbol: symbol.into(),
            config: ClientConfig::default(),
        }
    }

    /// Set the country (automatically sets correct lang and region)
    ///
    /// This is the recommended way to configure regional settings as it ensures
    /// lang and region codes are correctly paired.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use finance_query::{AsyncTicker, Country};
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let ticker = AsyncTicker::builder("7203.T")
    ///     .country(Country::Taiwan)
    ///     .build()
    ///     .await?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn country(mut self, country: crate::constants::Country) -> Self {
        self.config.lang = country.lang().to_string();
        self.config.region = country.region().to_string();
        self
    }

    /// Set the language code (e.g., "en-US", "ja-JP", "de-DE")
    ///
    /// For standard countries, prefer using `.country()` instead to ensure
    /// correct lang/region pairing.
    pub fn lang(mut self, lang: impl Into<String>) -> Self {
        self.config.lang = lang.into();
        self
    }

    /// Set the region code (e.g., "US", "JP", "DE")
    ///
    /// For standard countries, prefer using `.country()` instead to ensure
    /// correct lang/region pairing.
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

    /// Set a complete ClientConfig (overrides any previously set individual config fields)
    pub fn config(mut self, config: ClientConfig) -> Self {
        self.config = config;
        self
    }

    /// Build the AsyncTicker instance
    pub async fn build(self) -> Result<AsyncTicker> {
        let client = Arc::new(YahooClient::new(self.config).await?);

        Ok(AsyncTicker {
            core: TickerCoreData::new(self.symbol),
            client,
            quote_summary: Arc::new(tokio::sync::RwLock::new(None)),
            chart_cache: Arc::new(tokio::sync::RwLock::new(HashMap::new())),
            recommendations_cache: Arc::new(tokio::sync::RwLock::new(None)),
        })
    }
}

/// Asynchronous ticker for fetching symbol-specific data.
pub struct AsyncTicker {
    core: TickerCoreData,
    client: Arc<YahooClient>,
    quote_summary: Arc<tokio::sync::RwLock<Option<QuoteSummaryResponse>>>,
    chart_cache: Arc<tokio::sync::RwLock<HashMap<(Interval, TimeRange), ChartResult>>>,
    recommendations_cache: Arc<tokio::sync::RwLock<Option<RecommendationResponse>>>,
}

impl AsyncTicker {
    /// Creates a new async ticker with default configuration
    ///
    /// # Arguments
    ///
    /// * `symbol` - Stock symbol (e.g., "AAPL", "MSFT")
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use finance_query::AsyncTicker;
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let ticker = AsyncTicker::new("AAPL").await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn new(symbol: impl Into<String>) -> Result<Self> {
        Self::builder(symbol).build().await
    }

    /// Creates a new builder for AsyncTicker
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
    /// use finance_query::AsyncTicker;
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// // Simple case with defaults (same as new())
    /// let ticker = AsyncTicker::builder("AAPL").build().await?;
    ///
    /// // With custom configuration
    /// let ticker = AsyncTicker::builder("AAPL")
    ///     .lang("ja-JP")
    ///     .region("JP")
    ///     .build()
    ///     .await?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn builder(symbol: impl Into<String>) -> AsyncTickerBuilder {
        AsyncTickerBuilder::new(symbol)
    }

    /// Returns the ticker symbol
    pub fn symbol(&self) -> &str {
        &self.core.symbol
    }

    /// Ensures quote summary is loaded
    async fn ensure_quote_summary_loaded(&self) -> Result<()> {
        // Quick read check
        {
            let cache = self.quote_summary.read().await;
            if cache.is_some() {
                return Ok(());
            }
        }

        // Acquire write lock
        let mut cache = self.quote_summary.write().await;

        // Double-check (another task may have loaded while we waited)
        if cache.is_some() {
            return Ok(());
        }

        // Fetch using core's URL builder
        let url = self.core.build_quote_summary_url();
        let http_response = self.client.request_with_crumb(&url).await?;
        let json = http_response.json::<serde_json::Value>().await?;

        // Parse using core's parser
        let response = self.core.parse_quote_summary(json)?;
        *cache = Some(response);

        Ok(())
    }
}

// Generate quote summary accessor methods using macro to eliminate duplication.
// This replaces 20 hand-written methods (140 lines) with a concise declaration.
macros::define_quote_accessors! {
    /// Get price information
    price -> Price, "price",

    /// Get summary detail
    summary_detail -> SummaryDetail, "summaryDetail",

    /// Get financial data
    financial_data -> FinancialData, "financialData",

    /// Get key statistics
    key_stats -> DefaultKeyStatistics, "defaultKeyStatistics",

    /// Get asset profile
    asset_profile -> AssetProfile, "assetProfile",

    /// Get calendar events
    calendar_events -> CalendarEvents, "calendarEvents",

    /// Get earnings
    earnings -> Earnings, "earnings",

    /// Get earnings trend
    earnings_trend -> EarningsTrend, "earningsTrend",

    /// Get earnings history
    earnings_history -> EarningsHistory, "earningsHistory",

    /// Get recommendation trend
    recommendation_trend -> RecommendationTrend, "recommendationTrend",

    /// Get insider holders
    insider_holders -> InsiderHolders, "insiderHolders",

    /// Get insider transactions
    insider_transactions -> InsiderTransactions, "insiderTransactions",

    /// Get institution ownership
    institution_ownership -> InstitutionOwnership, "institutionOwnership",

    /// Get fund ownership
    fund_ownership -> FundOwnership, "fundOwnership",

    /// Get major holders breakdown
    major_holders -> MajorHoldersBreakdown, "majorHoldersBreakdown",

    /// Get net share purchase activity
    share_purchase_activity -> NetSharePurchaseActivity, "netSharePurchaseActivity",

    /// Get quote type
    quote_type -> QuoteTypeData, "quoteType",

    /// Get summary profile
    summary_profile -> SummaryProfile, "summaryProfile",

    /// Get SEC filings
    sec_filings -> SecFilings, "secFilings",

    /// Get upgrade/downgrade history
    grading_history -> UpgradeDowngradeHistory, "upgradeDowngradeHistory",
}

impl AsyncTicker {
    /// Get full quote
    pub async fn quote(&self) -> Result<Quote> {
        // Ensure quote summary is loaded (fetches all modules)
        self.ensure_quote_summary_loaded().await?;

        // Get the cached quote summary
        let cache = self.quote_summary.read().await;
        let response = cache
            .as_ref()
            .ok_or_else(|| crate::error::YahooError::SymbolNotFound {
                symbol: Some(self.core.symbol.clone()),
                context: "Quote summary not loaded".to_string(),
            })?;

        // Convert QuoteSummaryResponse to Quote
        Ok(Quote::from_response(response))
    }

    /// Get historical chart data
    pub async fn chart(&self, interval: Interval, range: TimeRange) -> Result<Chart> {
        // Check cache first
        {
            let cache = self.chart_cache.read().await;
            if let Some(cached) = cache.get(&(interval, range)) {
                return Ok(Chart {
                    symbol: self.core.symbol.clone(),
                    meta: cached.meta.clone(),
                    candles: cached.to_candles(),
                    interval: Some(interval.as_str().to_string()),
                    range: Some(range.as_str().to_string()),
                });
            }
        }

        // Fetch using client delegation
        let json = self
            .client
            .get_chart(&self.core.symbol, interval, range)
            .await?;
        let response = ChartResponse::from_json(json).map_err(|e| {
            crate::error::YahooError::ResponseStructureError {
                field: "chart".to_string(),
                context: e.to_string(),
            }
        })?;

        let mut results =
            response
                .chart
                .result
                .ok_or_else(|| crate::error::YahooError::SymbolNotFound {
                    symbol: Some(self.core.symbol.clone()),
                    context: "Chart data not found".to_string(),
                })?;

        let result = results
            .pop()
            .ok_or_else(|| crate::error::YahooError::SymbolNotFound {
                symbol: Some(self.core.symbol.clone()),
                context: "Chart data empty".to_string(),
            })?;

        let candles = result.to_candles();
        let meta = result.meta.clone();

        // Cache the result
        {
            let mut cache = self.chart_cache.write().await;
            cache.insert((interval, range), result);
        }

        Ok(Chart {
            symbol: self.core.symbol.clone(),
            meta,
            candles,
            interval: Some(interval.as_str().to_string()),
            range: Some(range.as_str().to_string()),
        })
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
    /// use finance_query::{AsyncTicker, Interval, TimeRange};
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let ticker = AsyncTicker::new("AAPL").await?;
    /// let indicators = ticker.indicators(Interval::Daily, TimeRange::OneYear).await?;
    ///
    /// println!("RSI(14): {:?}", indicators.rsi_14);
    /// println!("MACD: {:?}", indicators.macd);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn indicators(
        &self,
        interval: Interval,
        range: TimeRange,
    ) -> Result<crate::indicators::IndicatorsSummary> {
        // Fetch chart data
        let chart = self.chart(interval, range).await?;

        // Calculate indicators from candles
        Ok(crate::indicators::calculate_indicators(&chart.candles))
    }

    /// Get analyst recommendations
    pub async fn recommendations(&self, limit: u32) -> Result<Recommendation> {
        // Check cache
        {
            let cache = self.recommendations_cache.read().await;
            if let Some(cached) = cache.as_ref() {
                return Ok(Recommendation {
                    symbol: self.core.symbol.clone(),
                    recommendations: cached
                        .finance
                        .result
                        .iter()
                        .flat_map(|r| &r.recommended_symbols)
                        .cloned()
                        .collect(),
                });
            }
        }

        // Fetch using client delegation
        let json = self
            .client
            .get_recommendations(&self.core.symbol, limit)
            .await?;
        let response = RecommendationResponse::from_json(json).map_err(|e| {
            crate::error::YahooError::ResponseStructureError {
                field: "finance".to_string(),
                context: e.to_string(),
            }
        })?;

        // Cache
        {
            let mut cache = self.recommendations_cache.write().await;
            *cache = Some(response.clone());
        }

        Ok(Recommendation {
            symbol: self.core.symbol.clone(),
            recommendations: response
                .finance
                .result
                .iter()
                .flat_map(|r| &r.recommended_symbols)
                .cloned()
                .collect(),
        })
    }

    /// Get time series data
    pub async fn timeseries(
        &self,
        types: &[&str],
        period1: i64,
        period2: i64,
    ) -> Result<TimeseriesResponse> {
        let json = self
            .client
            .get_fundamentals_timeseries(&self.core.symbol, period1, period2, types)
            .await?;
        TimeseriesResponse::from_json(json).map_err(|e| {
            crate::error::YahooError::ResponseStructureError {
                field: "timeseries".to_string(),
                context: e.to_string(),
            }
        })
    }

    /// Get financial statements
    pub async fn financials(&self, _freq: &str) -> Result<TimeseriesResponse> {
        let types = &[
            "quarterlyTotalRevenue",
            "quarterlyNetIncome",
            "quarterlyGrossProfit",
        ];
        let period2 = chrono::Utc::now().timestamp();
        let period1 = period2 - 365 * 24 * 60 * 60; // 1 year ago

        let json = self
            .client
            .get_fundamentals_timeseries(&self.core.symbol, period1, period2, types)
            .await?;
        TimeseriesResponse::from_json(json).map_err(|e| {
            crate::error::YahooError::ResponseStructureError {
                field: "timeseries".to_string(),
                context: e.to_string(),
            }
        })
    }

    /// Get news articles
    pub async fn news(&self, count: u32) -> Result<NewsResponse> {
        let json = self.client.get_news(&[&self.core.symbol], count).await?;
        serde_json::from_value(json).map_err(|e| crate::error::YahooError::ResponseStructureError {
            field: "news".to_string(),
            context: e.to_string(),
        })
    }

    /// Get options chain
    pub async fn options(&self, date: Option<i64>) -> Result<OptionsResponse> {
        let json = self.client.get_options(&self.core.symbol, date).await?;
        serde_json::from_value(json).map_err(|e| crate::error::YahooError::ResponseStructureError {
            field: "options".to_string(),
            context: e.to_string(),
        })
    }
}

// Sync ticker implementation

/// Builder for Ticker
///
/// Provides a fluent API for constructing Ticker instances.
pub struct TickerBuilder {
    symbol: String,
    config: ClientConfig,
}

impl TickerBuilder {
    fn new(symbol: impl Into<String>) -> Self {
        Self {
            symbol: symbol.into(),
            config: ClientConfig::default(),
        }
    }

    /// Set the country (automatically sets correct lang and region)
    ///
    /// This is the recommended way to configure regional settings as it ensures
    /// lang and region codes are correctly paired.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use finance_query::{Ticker, Country};
    ///
    /// # fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let ticker = Ticker::builder("7203.T")
    ///     .country(Country::Taiwan)
    ///     .build()?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn country(mut self, country: crate::constants::Country) -> Self {
        self.config.lang = country.lang().to_string();
        self.config.region = country.region().to_string();
        self
    }

    /// Set the language code (e.g., "en-US", "ja-JP", "de-DE")
    ///
    /// For standard countries, prefer using `.country()` instead to ensure
    /// correct lang/region pairing.
    pub fn lang(mut self, lang: impl Into<String>) -> Self {
        self.config.lang = lang.into();
        self
    }

    /// Set the region code (e.g., "US", "JP", "DE")
    ///
    /// For standard countries, prefer using `.country()` instead to ensure
    /// correct lang/region pairing.
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

    /// Set a complete ClientConfig (overrides any previously set individual config fields)
    pub fn config(mut self, config: ClientConfig) -> Self {
        self.config = config;
        self
    }

    /// Build the Ticker instance
    pub fn build(self) -> Result<Ticker> {
        let client = Arc::new(BlockingYahooClient::new(self.config)?);

        Ok(Ticker {
            core: TickerCoreData::new(self.symbol),
            client,
            quote_summary: Arc::new(std::sync::RwLock::new(None)),
            chart_cache: Arc::new(std::sync::RwLock::new(HashMap::new())),
            recommendations_cache: Arc::new(std::sync::RwLock::new(None)),
        })
    }
}

/// Synchronous/blocking ticker for fetching symbol-specific data.
pub struct Ticker {
    core: TickerCoreData,
    client: Arc<BlockingYahooClient>,
    quote_summary: Arc<std::sync::RwLock<Option<QuoteSummaryResponse>>>,
    chart_cache: Arc<std::sync::RwLock<HashMap<(Interval, TimeRange), ChartResult>>>,
    recommendations_cache: Arc<std::sync::RwLock<Option<RecommendationResponse>>>,
}

impl Ticker {
    /// Creates a new sync ticker with default configuration
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
    /// # fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let ticker = Ticker::new("AAPL")?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn new(symbol: impl Into<String>) -> Result<Self> {
        Self::builder(symbol).build()
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
    /// # fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// // Simple case with defaults (same as new())
    /// let ticker = Ticker::builder("AAPL").build()?;
    ///
    /// // With custom configuration
    /// let ticker = Ticker::builder("AAPL")
    ///     .lang("ja-JP")
    ///     .region("JP")
    ///     .build()?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn builder(symbol: impl Into<String>) -> TickerBuilder {
        TickerBuilder::new(symbol)
    }

    /// Returns the ticker symbol
    pub fn symbol(&self) -> &str {
        &self.core.symbol
    }

    /// Ensures quote summary is loaded
    fn ensure_quote_summary_loaded(&self) -> Result<()> {
        // Quick read check
        {
            let cache = self.quote_summary.read().unwrap();
            if cache.is_some() {
                return Ok(());
            }
        }

        // Acquire write lock
        let mut cache = self.quote_summary.write().unwrap();

        // Double-check
        if cache.is_some() {
            return Ok(());
        }

        // Fetch quote summary
        let modules: Vec<&str> = Module::all().iter().map(|m| m.as_str()).collect();
        let json = self.client.get_quote_summary(&self.core.symbol, &modules)?;
        let response = self.core.parse_quote_summary(json)?;
        *cache = Some(response);

        Ok(())
    }

    /// Get full quote
    pub fn quote(&self) -> Result<Quote> {
        // Ensure quote summary is loaded (fetches all modules)
        self.ensure_quote_summary_loaded()?;

        // Get the cached quote summary
        let cache = self.quote_summary.read().unwrap();
        let response = cache
            .as_ref()
            .ok_or_else(|| crate::error::YahooError::SymbolNotFound {
                symbol: Some(self.core.symbol.clone()),
                context: "Quote summary not loaded".to_string(),
            })?;

        // Convert QuoteSummaryResponse to Quote
        Ok(Quote::from_response(response))
    }

    /// Get historical chart data
    pub fn chart(&self, interval: Interval, range: TimeRange) -> Result<Chart> {
        // Check cache first
        {
            let cache = self.chart_cache.read().unwrap();
            if let Some(cached) = cache.get(&(interval, range)) {
                return Ok(Chart {
                    symbol: self.core.symbol.clone(),
                    meta: cached.meta.clone(),
                    candles: cached.to_candles(),
                    interval: Some(interval.as_str().to_string()),
                    range: Some(range.as_str().to_string()),
                });
            }
        }

        // Fetch using client delegation
        let json = self.client.get_chart(&self.core.symbol, interval, range)?;
        let response = ChartResponse::from_json(json).map_err(|e| {
            crate::error::YahooError::ResponseStructureError {
                field: "chart".to_string(),
                context: e.to_string(),
            }
        })?;

        let mut results =
            response
                .chart
                .result
                .ok_or_else(|| crate::error::YahooError::SymbolNotFound {
                    symbol: Some(self.core.symbol.clone()),
                    context: "Chart data not found".to_string(),
                })?;

        let result = results
            .pop()
            .ok_or_else(|| crate::error::YahooError::SymbolNotFound {
                symbol: Some(self.core.symbol.clone()),
                context: "Chart data empty".to_string(),
            })?;

        let candles = result.to_candles();
        let meta = result.meta.clone();

        // Cache the result
        {
            let mut cache = self.chart_cache.write().unwrap();
            cache.insert((interval, range), result);
        }

        Ok(Chart {
            symbol: self.core.symbol.clone(),
            meta,
            candles,
            interval: Some(interval.as_str().to_string()),
            range: Some(range.as_str().to_string()),
        })
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
    /// # fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let ticker = Ticker::new("AAPL")?;
    /// let indicators = ticker.indicators(Interval::Daily, TimeRange::OneYear)?;
    ///
    /// println!("RSI(14): {:?}", indicators.rsi_14);
    /// println!("MACD: {:?}", indicators.macd);
    /// # Ok(())
    /// # }
    /// ```
    pub fn indicators(
        &self,
        interval: Interval,
        range: TimeRange,
    ) -> Result<crate::indicators::IndicatorsSummary> {
        // Fetch chart data
        let chart = self.chart(interval, range)?;

        // Calculate indicators from candles
        Ok(crate::indicators::calculate_indicators(&chart.candles))
    }

    /// Get analyst recommendations
    pub fn recommendations(&self, limit: u32) -> Result<Recommendation> {
        // Check cache
        {
            let cache = self.recommendations_cache.read().unwrap();
            if let Some(cached) = cache.as_ref() {
                return Ok(Recommendation {
                    symbol: self.core.symbol.clone(),
                    recommendations: cached
                        .finance
                        .result
                        .iter()
                        .flat_map(|r| &r.recommended_symbols)
                        .cloned()
                        .collect(),
                });
            }
        }

        // Fetch using client delegation
        let json = self.client.get_recommendations(&self.core.symbol, limit)?;
        let response = RecommendationResponse::from_json(json).map_err(|e| {
            crate::error::YahooError::ResponseStructureError {
                field: "finance".to_string(),
                context: e.to_string(),
            }
        })?;

        // Cache
        {
            let mut cache = self.recommendations_cache.write().unwrap();
            *cache = Some(response.clone());
        }

        Ok(Recommendation {
            symbol: self.core.symbol.clone(),
            recommendations: response
                .finance
                .result
                .iter()
                .flat_map(|r| &r.recommended_symbols)
                .cloned()
                .collect(),
        })
    }

    /// Get time series data
    pub fn timeseries(
        &self,
        types: &[&str],
        period1: i64,
        period2: i64,
    ) -> Result<TimeseriesResponse> {
        let json =
            self.client
                .get_fundamentals_timeseries(&self.core.symbol, period1, period2, types)?;
        TimeseriesResponse::from_json(json).map_err(|e| {
            crate::error::YahooError::ResponseStructureError {
                field: "timeseries".to_string(),
                context: e.to_string(),
            }
        })
    }

    /// Get financial statements
    pub fn financials(&self, _freq: &str) -> Result<TimeseriesResponse> {
        let types = &[
            "quarterlyTotalRevenue",
            "quarterlyNetIncome",
            "quarterlyGrossProfit",
        ];
        let period2 = chrono::Utc::now().timestamp();
        let period1 = period2 - 365 * 24 * 60 * 60;

        let json =
            self.client
                .get_fundamentals_timeseries(&self.core.symbol, period1, period2, types)?;
        TimeseriesResponse::from_json(json).map_err(|e| {
            crate::error::YahooError::ResponseStructureError {
                field: "timeseries".to_string(),
                context: e.to_string(),
            }
        })
    }

    /// Get news articles
    pub fn news(&self, count: u32) -> Result<NewsResponse> {
        let json = self.client.get_news(&[&self.core.symbol], count)?;
        serde_json::from_value(json).map_err(|e| crate::error::YahooError::ResponseStructureError {
            field: "news".to_string(),
            context: e.to_string(),
        })
    }

    /// Get options chain
    pub fn options(&self, date: Option<i64>) -> Result<OptionsResponse> {
        let json = self.client.get_options(&self.core.symbol, date)?;
        serde_json::from_value(json).map_err(|e| crate::error::YahooError::ResponseStructureError {
            field: "options".to_string(),
            context: e.to_string(),
        })
    }
}
