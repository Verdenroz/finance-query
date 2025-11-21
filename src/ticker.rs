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
use crate::models::chart::{ChartResponse, ChartResult};
use crate::models::quote_summary::{Module, QuoteSummaryData};
use crate::models::recommendation::RecommendationResponse;
use crate::models::timeseries::TimeseriesResponse;

/// Main ticker struct for fetching financial data for a specific symbol
///
/// All data is fetched when the ticker is created, and properties provide
/// synchronous access to the cached data.
pub struct Ticker {
    /// The stock symbol (e.g., "AAPL", "MSFT")
    symbol: String,
    /// All fetched quote summary data
    data: QuoteSummaryData,
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
        let response = client.request_with_crumb(&full_url).await?;
        let json = response.json::<serde_json::Value>().await?;

        // Parse into response and then convert to structured data
        let quote_response =
            crate::models::quote_summary::QuoteSummaryResponse::from_json(json, &symbol)?;
        let data = quote_response.into_data()?;

        Ok(Self {
            symbol,
            data,
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
    ///
    /// Returns summary detail information (PE ratios, dividends, 52-week range)
    ///
    /// Note: Currently returns raw JSON. Will be strongly typed in future releases.
    pub fn summary_detail(&self) -> Option<&serde_json::Value> {
        self.data.summary_detail.as_ref()
    }

    /// Returns financial data (revenue, margins, cash flow)
    ///
    /// Note: Currently returns raw JSON. Will be strongly typed in future releases.
    pub fn financial_data(&self) -> Option<&serde_json::Value> {
        self.data.financial_data.as_ref()
    }

    /// Returns key statistics (enterprise value, shares outstanding, beta)
    ///
    /// Note: Currently returns raw JSON. Will be strongly typed in future releases.
    pub fn key_stats(&self) -> Option<&serde_json::Value> {
        self.data.key_stats.as_ref()
    }

    /// Returns asset profile (company info, sector, industry, officers)
    ///
    /// Note: Currently returns raw JSON. Will be strongly typed in future releases.
    pub fn asset_profile(&self) -> Option<&serde_json::Value> {
        self.data.asset_profile.as_ref()
    }

    /// Returns calendar events (earnings date, dividend date)
    pub fn calendar_events(&self) -> Option<&serde_json::Value> {
        self.data.calendar_events.as_ref()
    }

    /// Returns earnings data
    pub fn earnings(&self) -> Option<&serde_json::Value> {
        self.data.earnings.as_ref()
    }

    /// Returns earnings trend
    pub fn earnings_trend(&self) -> Option<&serde_json::Value> {
        self.data.earnings_trend.as_ref()
    }

    /// Returns earnings history
    pub fn earnings_history(&self) -> Option<&serde_json::Value> {
        self.data.earnings_history.as_ref()
    }

    /// Returns recommendation trend
    pub fn recommendation_trend(&self) -> Option<&serde_json::Value> {
        self.data.recommendation_trend.as_ref()
    }

    /// Returns insider holders
    pub fn insider_holders(&self) -> Option<&serde_json::Value> {
        self.data.insider_holders.as_ref()
    }

    /// Returns insider transactions
    pub fn insider_transactions(&self) -> Option<&serde_json::Value> {
        self.data.insider_transactions.as_ref()
    }

    /// Returns institution ownership
    pub fn institution_ownership(&self) -> Option<&serde_json::Value> {
        self.data.institution_ownership.as_ref()
    }

    /// Returns fund ownership
    pub fn fund_ownership(&self) -> Option<&serde_json::Value> {
        self.data.fund_ownership.as_ref()
    }

    /// Returns major holders breakdown
    pub fn major_holders(&self) -> Option<&serde_json::Value> {
        self.data.major_holders.as_ref()
    }

    /// Returns net share purchase activity
    pub fn share_purchase_activity(&self) -> Option<&serde_json::Value> {
        self.data.share_purchase_activity.as_ref()
    }

    /// Returns quote type information
    pub fn quote_type(&self) -> Option<&serde_json::Value> {
        self.data.quote_type.as_ref()
    }

    /// Returns summary profile
    pub fn summary_profile(&self) -> Option<&serde_json::Value> {
        self.data.summary_profile.as_ref()
    }

    /// Returns SEC filings
    pub fn sec_filings(&self) -> Option<&serde_json::Value> {
        self.data.sec_filings.as_ref()
    }

    /// Returns upgrade/downgrade history
    pub fn grading_history(&self) -> Option<&serde_json::Value> {
        self.data.grading_history.as_ref()
    }

    /// Returns the complete underlying data structure
    ///
    /// This provides access to all modules at once.
    pub fn data(&self) -> &QuoteSummaryData {
        &self.data
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
