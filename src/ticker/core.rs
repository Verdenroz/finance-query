//! Ticker implementation for accessing symbol-specific data from Yahoo Finance.
//!
//! Provides async interface for fetching quotes, charts, financials, and news.

use super::macros;
use crate::client::{ClientConfig, YahooClient};
use crate::constants::{Interval, TimeRange};
use crate::error::Result;
use crate::models::chart::Chart;
use crate::models::chart::response::ChartResponse;
use crate::models::chart::result::ChartResult;
use crate::models::financials::FinancialStatement;
use crate::models::options::Options;
use crate::models::quote::{
    AssetProfile, CalendarEvents, DefaultKeyStatistics, Earnings, EarningsHistory, EarningsTrend,
    FinancialData, FundOwnership, InsiderHolders, InsiderTransactions, InstitutionOwnership,
    MajorHoldersBreakdown, Module, NetSharePurchaseActivity, Price, Quote, QuoteSummaryResponse,
    QuoteTypeData, RecommendationTrend, SecFilings, SummaryDetail, SummaryProfile,
    UpgradeDowngradeHistory,
};
use crate::models::recommendation::Recommendation;
use crate::models::recommendation::response::RecommendationResponse;
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
    /// lang and region are correctly paired.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use finance_query::{Ticker, Country};
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let ticker = Ticker::builder("7203.T")
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

    /// Build the Ticker instance
    pub async fn build(self) -> Result<Ticker> {
        let client = Arc::new(YahooClient::new(self.config).await?);

        Ok(Ticker {
            core: TickerCoreData::new(self.symbol),
            client,
            quote_summary: Arc::new(tokio::sync::RwLock::new(None)),
            chart_cache: Arc::new(tokio::sync::RwLock::new(HashMap::new())),
            recommendations_cache: Arc::new(tokio::sync::RwLock::new(None)),
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
/// let quote = ticker.quote(false).await?;
/// println!("Price: {:?}", quote.regular_market_price);
///
/// // Get chart data
/// use finance_query::{Interval, TimeRange};
/// let chart = ticker.chart(Interval::OneDay, TimeRange::OneMonth).await?;
/// println!("Candles: {}", chart.candles.len());
/// # Ok(())
/// # }
/// ```
pub struct Ticker {
    core: TickerCoreData,
    client: Arc<YahooClient>,
    quote_summary: Arc<tokio::sync::RwLock<Option<QuoteSummaryResponse>>>,
    chart_cache: Arc<tokio::sync::RwLock<HashMap<(Interval, TimeRange), ChartResult>>>,
    recommendations_cache: Arc<tokio::sync::RwLock<Option<RecommendationResponse>>>,
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
    ///     .region("JP")
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

impl Ticker {
    /// Get full quote data with optional logo URL
    ///
    /// # Arguments
    ///
    /// * `include_logo` - Whether to fetch and include the company logo URL
    ///
    /// When `include_logo` is true, fetches both quote summary and logo URL in parallel
    /// using tokio::join! for minimal latency impact (~0-100ms overhead).
    pub async fn quote(&self, include_logo: bool) -> Result<Quote> {
        if include_logo {
            // Parallel fetch: quoteSummary AND logo
            let (quote_result, logo_result) = tokio::join!(
                self.ensure_quote_summary_loaded(),
                self.client.get_logo_url(&self.core.symbol)
            );

            // Handle quoteSummary result (required)
            quote_result?;

            // Get cached quote summary
            let cache = self.quote_summary.read().await;
            let response =
                cache
                    .as_ref()
                    .ok_or_else(|| crate::error::YahooError::SymbolNotFound {
                        symbol: Some(self.core.symbol.clone()),
                        context: "Quote summary not loaded".to_string(),
                    })?;

            // Create Quote with logos (logo_result is (Option<String>, Option<String>))
            let (logo_url, company_logo_url) = logo_result;
            Ok(Quote::from_response(response, logo_url, company_logo_url))
        } else {
            // Original behavior - no logo fetch
            self.ensure_quote_summary_loaded().await?;

            let cache = self.quote_summary.read().await;
            let response =
                cache
                    .as_ref()
                    .ok_or_else(|| crate::error::YahooError::SymbolNotFound {
                        symbol: Some(self.core.symbol.clone()),
                        context: "Quote summary not loaded".to_string(),
                    })?;

            Ok(Quote::from_response(response, None, None))
        }
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
    /// use finance_query::{Ticker, Interval, TimeRange};
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let ticker = Ticker::new("AAPL").await?;
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
    ) -> Result<crate::models::indicators::IndicatorsSummary> {
        // Fetch chart data
        let chart = self.chart(interval, range).await?;

        // Calculate indicators from candles
        Ok(crate::models::indicators::calculate_indicators(
            &chart.candles,
        ))
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
        self.client
            .get_financials(&self.core.symbol, statement_type, frequency)
            .await
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
        crate::scrapers::stockanalysis::scrape_symbol_news(&self.core.symbol).await
    }

    /// Get options chain
    pub async fn options(&self, date: Option<i64>) -> Result<Options> {
        let json = self.client.get_options(&self.core.symbol, date).await?;
        serde_json::from_value(json).map_err(|e| crate::error::YahooError::ResponseStructureError {
            field: "options".to_string(),
            context: e.to_string(),
        })
    }
}
