//! # Ticker
//!
//! The main entry point for fetching financial data for a specific symbol.
//! Provides a yfinance-like API for Rust.
//!
//! ## Examples
//!
//! ```no_run
//! use finance_query::Ticker;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let ticker = Ticker::new("AAPL").await?;
//!     let price = ticker.price().await?;
//!     println!("Current price: ${}", price.regular_market_price);
//!     Ok(())
//! }
//! ```

use crate::client::{ClientConfig, YahooClient};
use crate::constants::{Interval, TimeRange};
use crate::error::Result;
use crate::models::chart::{Chart, ChartResponse, ChartResult};
use crate::models::quote::{
    DefaultKeyStatistics, FinancialData, Module, Price, Quote, QuoteSummaryResponse, QuoteTypeData,
    SummaryDetail, SummaryProfile,
};
use crate::models::recommendation::{Recommendation, RecommendationResponse};
use crate::models::timeseries::TimeseriesResponse;

/// Main ticker struct for fetching financial data for a specific symbol
///
/// All data is fetched when the ticker is created, and properties provide
/// synchronous access to the cached data.
pub struct Ticker {
    /// The stock symbol (e.g., "AAPL", "MSFT")
    symbol: String,
    /// All fetched quote summary data
    response: QuoteSummaryResponse,
    /// Yahoo Finance client for additional requests
    client: YahooClient,
}

impl Ticker {
    /// Creates a new Ticker instance for the given symbol
    ///
    /// This fetches ALL available modules from Yahoo Finance in a single request
    /// and deserializes them. After creation, all properties provide synchronous
    /// access to the cached data.
    ///
    /// # Arguments
    ///
    /// * `symbol` - The stock ticker symbol (e.g., "AAPL", "MSFT", "TSLA")
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use finance_query::Ticker;
    ///
    /// #[tokio::main]
    /// async fn main() -> Result<(), Box<dyn std::error::Error>> {
    ///     // Fetches all data in one request
    ///     let ticker = Ticker::new("AAPL").await?;
    ///
    ///     // All subsequent accesses are synchronous (no .await)
    ///     if let Some(price) = ticker.price() {
    ///         println!("Current price: ${:.2}", price.current_price().unwrap_or(0.0));
    ///     }
    ///
    ///     Ok(())
    /// }
    /// ```
    pub async fn new(symbol: impl Into<String>) -> Result<Self> {
        let symbol = symbol.into();
        let client = YahooClient::new(ClientConfig::default()).await?;

        // Fetch ALL modules in one request
        let all_modules = Module::all();
        let url = crate::constants::endpoints::quote_summary(&symbol);
        let module_str = all_modules
            .iter()
            .map(|m| m.as_str())
            .collect::<Vec<_>>()
            .join(",");

        let full_url = format!("{}?modules={}", url, module_str);
        let http_response = client.request_with_crumb(&full_url).await?;
        let json = http_response.json::<serde_json::Value>().await?;

        // Parse into quote summary response
        let response = crate::models::quote::QuoteSummaryResponse::from_json(json, &symbol)?;

        Ok(Self {
            symbol,
            response,
            client,
        })
    }

    /// Returns the symbol for this ticker
    pub fn symbol(&self) -> &str {
        &self.symbol
    }

    // ==================== Property Accessors ====================
    // All properties return references to cached data (synchronous access)

    /// Returns detailed pricing data
    ///
    /// Provides access to current price, pre/post market data, volume,
    /// market cap, and exchange information.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use finance_query::Ticker;
    ///
    /// #[tokio::main]
    /// async fn main() -> Result<(), Box<dyn std::error::Error>> {
    ///     let ticker = Ticker::new("AAPL").await?;
    ///
    ///     // Synchronous access - no .await needed!
    ///     if let Some(price) = ticker.price() {
    ///         if let Some(current) = price.current_price() {
    ///             println!("Current price: ${:.2}", current);
    ///         }
    ///
    ///         if let Some(change_pct) = price.day_change_percent() {
    ///             println!("Day change: {:.2}%", change_pct * 100.0);
    ///         }
    ///     }
    ///
    ///     Ok(())
    /// }
    /// ```
    pub fn price(&self) -> Option<Price> {
        self.response.get_typed("price").ok()
    }

    /// Returns summary detail information (PE ratios, dividends, 52-week range)
    ///
    /// Returns strongly typed summary detail data including trading metrics,
    /// valuation data, market cap, volume, and dividend information.
    pub fn summary_detail(&self) -> Option<SummaryDetail> {
        self.response.get_typed("summaryDetail").ok()
    }

    /// Returns financial data (revenue, margins, cash flow)
    ///
    /// Returns strongly typed financial data including current price, margins,
    /// cash flow metrics, and analyst recommendations.
    pub fn financial_data(&self) -> Option<FinancialData> {
        self.response.get_typed("financialData").ok()
    }

    /// Returns key statistics (enterprise value, shares outstanding, beta)
    ///
    /// Returns strongly typed key statistics including valuation metrics,
    /// share data, financial ratios, and other important statistics.
    pub fn key_stats(&self) -> Option<DefaultKeyStatistics> {
        self.response.get_typed("defaultKeyStatistics").ok()
    }

    /// Returns asset profile (company info, sector, industry, officers)
    ///
    /// Note: Currently returns raw JSON. Will be strongly typed in future releases.
    pub fn asset_profile(&self) -> Option<&serde_json::Value> {
        self.response.get_module("assetProfile")
    }

    /// Returns calendar events (earnings date, dividend date)
    pub fn calendar_events(&self) -> Option<&serde_json::Value> {
        self.response.get_module("calendarEvents")
    }

    /// Returns earnings data
    pub fn earnings(&self) -> Option<&serde_json::Value> {
        self.response.get_module("earnings")
    }

    /// Returns earnings trend
    pub fn earnings_trend(&self) -> Option<&serde_json::Value> {
        self.response.get_module("earningsTrend")
    }

    /// Returns earnings history
    pub fn earnings_history(&self) -> Option<&serde_json::Value> {
        self.response.get_module("earningsHistory")
    }

    /// Returns recommendation trend
    pub fn recommendation_trend(&self) -> Option<&serde_json::Value> {
        self.response.get_module("recommendationTrend")
    }

    /// Returns insider holders
    pub fn insider_holders(&self) -> Option<&serde_json::Value> {
        self.response.get_module("insiderHolders")
    }

    /// Returns insider transactions
    pub fn insider_transactions(&self) -> Option<&serde_json::Value> {
        self.response.get_module("insiderTransactions")
    }

    /// Returns institution ownership
    pub fn institution_ownership(&self) -> Option<&serde_json::Value> {
        self.response.get_module("institutionOwnership")
    }

    /// Returns fund ownership
    pub fn fund_ownership(&self) -> Option<&serde_json::Value> {
        self.response.get_module("fundOwnership")
    }

    /// Returns major holders breakdown
    pub fn major_holders(&self) -> Option<&serde_json::Value> {
        self.response.get_module("majorHoldersBreakdown")
    }

    /// Returns net share purchase activity
    pub fn share_purchase_activity(&self) -> Option<&serde_json::Value> {
        self.response.get_module("netSharePurchaseActivity")
    }

    /// Returns quote type information
    ///
    /// Returns strongly typed metadata about the symbol including exchange,
    /// company names, quote type, and timezone information.
    pub fn quote_type(&self) -> Option<QuoteTypeData> {
        self.response.get_typed("quoteType").ok()
    }

    /// Returns summary profile
    ///
    /// Returns strongly typed company profile information including address,
    /// sector, industry, employee count, and business description.
    pub fn summary_profile(&self) -> Option<SummaryProfile> {
        self.response.get_typed("summaryProfile").ok()
    }

    /// Returns SEC filings
    pub fn sec_filings(&self) -> Option<&serde_json::Value> {
        self.response.get_module("secFilings")
    }

    /// Returns upgrade/downgrade history
    pub fn grading_history(&self) -> Option<&serde_json::Value> {
        self.response.get_module("upgradeDowngradeHistory")
    }

    /// Returns the complete underlying quote summary response
    ///
    /// This provides access to all modules at once.
    pub fn response(&self) -> &QuoteSummaryResponse {
        &self.response
    }

    /// Returns all modules as raw JSON
    ///
    /// This is useful for debugging - it returns the complete raw response
    /// from Yahoo Finance without any additional parsing or modeling.
    pub fn raw_modules(&self) -> serde_json::Value {
        let mut map = serde_json::Map::new();
        map.insert(
            "symbol".to_string(),
            serde_json::Value::String(self.symbol.clone()),
        );

        for (key, value) in &self.response.modules {
            map.insert(key.clone(), value.clone());
        }

        serde_json::Value::Object(map)
    }

    /// Converts this ticker's data into a fully typed Quote
    ///
    /// This aggregates all typed modules into a single convenient structure
    /// that's easy to serialize and work with. Recommended for API responses
    /// and data export.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use finance_query::Ticker;
    ///
    /// #[tokio::main]
    /// async fn main() -> Result<(), Box<dyn std::error::Error>> {
    ///     let ticker = Ticker::new("AAPL").await?;
    ///     let quote = ticker.to_quote();
    ///
    ///     // Serialize to JSON
    ///     let json = serde_json::to_string_pretty(&quote)?;
    ///     println!("{}", json);
    ///
    ///     Ok(())
    /// }
    /// ```
    pub fn to_quote(&self) -> Quote {
        Quote::from_response(&self.response)
    }

    // ==================== Async Methods ====================
    // These methods make additional API requests

    /// Fetch historical chart data for this ticker
    ///
    /// # Arguments
    ///
    /// * `interval` - The data interval (e.g., OneDay, OneHour)
    /// * `range` - The time range (e.g., OneMonth, OneYear)
    ///
    /// # Example
    ///
    /// ```no_run
    /// use finance_query::{Ticker, Interval, TimeRange};
    ///
    /// #[tokio::main]
    /// async fn main() -> Result<(), Box<dyn std::error::Error>> {
    ///     let ticker = Ticker::new("AAPL").await?;
    ///     let chart = ticker.chart(Interval::OneDay, TimeRange::OneMonth).await?;
    ///
    ///     let candles = chart.to_candles();
    ///     for candle in candles.iter().take(5) {
    ///         println!("Close: ${:.2}", candle.close);
    ///     }
    ///     Ok(())
    /// }
    /// ```
    pub async fn chart(&self, interval: Interval, range: TimeRange) -> Result<ChartResult> {
        let json = self.client.get_chart(&self.symbol, interval, range).await?;
        let response = ChartResponse::from_json(json)
            .map_err(|e| crate::error::YahooError::ParseError(e.to_string()))?;

        response
            .chart
            .result
            .and_then(|mut r| r.pop())
            .ok_or_else(|| crate::error::YahooError::SymbolNotFound(self.symbol.clone()))
    }

    /// Fetch chart data as a fully typed Chart structure
    ///
    /// This is the recommended method for getting chart data in a clean,
    /// serializable format. It combines metadata and candles into a single
    /// convenient structure.
    ///
    /// # Arguments
    ///
    /// * `interval` - The data interval (e.g., OneDay, OneHour)
    /// * `range` - The time range (e.g., OneMonth, OneYear)
    ///
    /// # Example
    ///
    /// ```no_run
    /// use finance_query::{Ticker, Interval, TimeRange};
    ///
    /// #[tokio::main]
    /// async fn main() -> Result<(), Box<dyn std::error::Error>> {
    ///     let ticker = Ticker::new("AAPL").await?;
    ///     let chart = ticker.to_chart(Interval::OneDay, TimeRange::OneMonth).await?;
    ///
    ///     println!("Symbol: {}", chart.symbol);
    ///     println!("Candles: {}", chart.candles.len());
    ///
    ///     // Serialize to JSON
    ///     let json = serde_json::to_string_pretty(&chart)?;
    ///     println!("{}", json);
    ///
    ///     Ok(())
    /// }
    /// ```
    pub async fn to_chart(&self, interval: Interval, range: TimeRange) -> Result<Chart> {
        let chart_result = self.chart(interval, range).await?;
        Ok(chart_result.to_chart())
    }

    /// Fetch similar/recommended stocks for this ticker
    ///
    /// # Arguments
    ///
    /// * `limit` - Maximum number of recommendations to return
    ///
    /// # Example
    ///
    /// ```no_run
    /// use finance_query::Ticker;
    ///
    /// #[tokio::main]
    /// async fn main() -> Result<(), Box<dyn std::error::Error>> {
    ///     let ticker = Ticker::new("AAPL").await?;
    ///     let similar = ticker.similar(5).await?;
    ///
    ///     for symbol in similar.symbols() {
    ///         println!("Similar: {}", symbol);
    ///     }
    ///     Ok(())
    /// }
    /// ```
    pub async fn similar(&self, limit: u32) -> Result<RecommendationResponse> {
        let json = self.client.get_similar_quotes(&self.symbol, limit).await?;
        RecommendationResponse::from_json(json)
            .map_err(|e| crate::error::YahooError::ParseError(e.to_string()))
    }

    /// Fetch similar stocks as a fully typed Recommendation structure
    ///
    /// This is the recommended method for getting similar stocks in a clean,
    /// serializable format.
    ///
    /// # Arguments
    ///
    /// * `limit` - Maximum number of recommendations to return
    ///
    /// # Example
    ///
    /// ```no_run
    /// use finance_query::Ticker;
    ///
    /// #[tokio::main]
    /// async fn main() -> Result<(), Box<dyn std::error::Error>> {
    ///     let ticker = Ticker::new("AAPL").await?;
    ///     let recommendation = ticker.to_recommendation(5).await?;
    ///
    ///     println!("Similar to {}: {} stocks", recommendation.symbol, recommendation.count());
    ///     for rec in &recommendation.recommendations {
    ///         println!("  {} (score: {:.3})", rec.symbol, rec.score);
    ///     }
    ///
    ///     // Serialize to JSON
    ///     let json = serde_json::to_string_pretty(&recommendation)?;
    ///     println!("{}", json);
    ///
    ///     Ok(())
    /// }
    /// ```
    pub async fn to_recommendation(&self, limit: u32) -> Result<Recommendation> {
        let response = self.similar(limit).await?;
        response
            .finance
            .result
            .first()
            .map(|r| r.to_recommendation())
            .ok_or_else(|| crate::error::YahooError::SymbolNotFound(self.symbol.clone()))
    }

    /// Fetch financial timeseries data (revenue, income, etc.)
    ///
    /// # Arguments
    ///
    /// * `types` - List of fundamental types to fetch
    /// * `period1` - Start Unix timestamp
    /// * `period2` - End Unix timestamp
    ///
    /// # Example
    ///
    /// ```no_run
    /// use finance_query::Ticker;
    /// use finance_query::models::timeseries::fundamental_types::*;
    ///
    /// #[tokio::main]
    /// async fn main() -> Result<(), Box<dyn std::error::Error>> {
    ///     let ticker = Ticker::new("AAPL").await?;
    ///     let types = &[ANNUAL_TOTAL_REVENUE, ANNUAL_NET_INCOME];
    ///     let financials = ticker.financials(types, 0, 9999999999).await?;
    ///     Ok(())
    /// }
    /// ```
    pub async fn financials(
        &self,
        types: &[&str],
        period1: i64,
        period2: i64,
    ) -> Result<TimeseriesResponse> {
        let json = self
            .client
            .get_fundamentals_timeseries(&self.symbol, period1, period2, types)
            .await?;
        TimeseriesResponse::from_json(json)
            .map_err(|e| crate::error::YahooError::ParseError(e.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    #[ignore] // Requires network access
    async fn test_ticker_new() {
        let ticker = Ticker::new("AAPL").await;
        assert!(ticker.is_ok());
        if let Ok(t) = ticker {
            assert_eq!(t.symbol(), "AAPL");
            // Financial data should be available
            assert!(t.financial_data().is_some());
        }
    }

    #[tokio::test]
    #[ignore] // Requires network access
    async fn test_ticker_sync_access() {
        let ticker = Ticker::new("AAPL").await.unwrap();

        // Synchronous access - no .await!
        let financials = ticker.financial_data();
        assert!(financials.is_some());
    }

    #[tokio::test]
    #[ignore] // Requires network access
    async fn test_ticker_multiple_properties() {
        let ticker = Ticker::new("MSFT").await.unwrap();

        // All synchronous accesses
        assert!(ticker.financial_data().is_some());
        // Other modules may or may not be present depending on Yahoo's response
    }
}
