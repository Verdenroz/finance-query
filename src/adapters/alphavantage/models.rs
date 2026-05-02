//! Shared types and deserialization helpers for Alpha Vantage responses.

use serde::{Deserialize, Deserializer, Serialize};

/// Deserialize a string value to `Option<f64>`.
///
/// Alpha Vantage returns numeric values as strings, and uses `"None"` or `"."`
/// for missing data.
#[allow(dead_code)]
pub(crate) fn deserialize_optional_f64<'de, D>(
    deserializer: D,
) -> std::result::Result<Option<f64>, D::Error>
where
    D: Deserializer<'de>,
{
    let s: Option<String> = Option::deserialize(deserializer)?;
    match s.as_deref() {
        Some("None") | Some(".") | Some("-") | Some("") | None => Ok(None),
        Some(v) => v.parse::<f64>().ok().map_or(Ok(None), |n| Ok(Some(n))),
    }
}

/// Deserialize a string to `f64`, defaulting to `0.0` on failure.
#[allow(dead_code)]
pub(crate) fn deserialize_f64_from_str<'de, D>(
    deserializer: D,
) -> std::result::Result<f64, D::Error>
where
    D: Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    Ok(s.parse::<f64>().unwrap_or(0.0))
}

// ============================================================================
// Interval and SeriesType enums for Alpha Vantage API parameters
// ============================================================================

/// Time interval for Alpha Vantage time series and indicator requests.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AvInterval {
    /// 1-minute intervals
    OneMin,
    /// 5-minute intervals
    FiveMin,
    /// 15-minute intervals
    FifteenMin,
    /// 30-minute intervals
    ThirtyMin,
    /// 60-minute intervals
    SixtyMin,
    /// Daily intervals
    Daily,
    /// Weekly intervals
    Weekly,
    /// Monthly intervals
    Monthly,
}

impl AvInterval {
    /// Convert to the Alpha Vantage API parameter string.
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::OneMin => "1min",
            Self::FiveMin => "5min",
            Self::FifteenMin => "15min",
            Self::ThirtyMin => "30min",
            Self::SixtyMin => "60min",
            Self::Daily => "daily",
            Self::Weekly => "weekly",
            Self::Monthly => "monthly",
        }
    }
}

/// Price series type for technical indicator calculations.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SeriesType {
    /// Close price
    Close,
    /// Open price
    Open,
    /// High price
    High,
    /// Low price
    Low,
}

impl SeriesType {
    /// Convert to the Alpha Vantage API parameter string.
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Close => "close",
            Self::Open => "open",
            Self::High => "high",
            Self::Low => "low",
        }
    }
}

/// Output size for time series requests.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OutputSize {
    /// Returns the latest 100 data points (default).
    Compact,
    /// Returns up to 20+ years of historical data.
    Full,
}

impl OutputSize {
    /// Convert to the Alpha Vantage API parameter string.
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Compact => "compact",
            Self::Full => "full",
        }
    }
}

// ============================================================================
// Core time series response types
// ============================================================================

/// A single OHLCV data point from a time series response.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct TimeSeriesEntry {
    /// Timestamp or date string
    pub timestamp: String,
    /// Opening price
    pub open: f64,
    /// Highest price
    pub high: f64,
    /// Lowest price
    pub low: f64,
    /// Closing price
    pub close: f64,
    /// Trading volume
    pub volume: f64,
}

/// A single adjusted OHLCV data point including dividend and split information.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct AdjustedTimeSeriesEntry {
    /// Timestamp or date string
    pub timestamp: String,
    /// Opening price
    pub open: f64,
    /// Highest price
    pub high: f64,
    /// Lowest price
    pub low: f64,
    /// Closing price
    pub close: f64,
    /// Adjusted closing price
    pub adjusted_close: f64,
    /// Trading volume
    pub volume: f64,
    /// Dividend amount
    pub dividend_amount: f64,
    /// Split coefficient (only in daily adjusted)
    pub split_coefficient: Option<f64>,
}

/// Time series response containing metadata and OHLCV entries.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct TimeSeries {
    /// Symbol (e.g., `"AAPL"`)
    pub symbol: String,
    /// Last refreshed timestamp
    pub last_refreshed: String,
    /// Time series data points
    pub entries: Vec<TimeSeriesEntry>,
}

/// Adjusted time series response.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct AdjustedTimeSeries {
    /// Symbol (e.g., `"AAPL"`)
    pub symbol: String,
    /// Last refreshed timestamp
    pub last_refreshed: String,
    /// Adjusted time series data points
    pub entries: Vec<AdjustedTimeSeriesEntry>,
}

// ============================================================================
// Global quote
// ============================================================================

/// Real-time quote for a single ticker.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct GlobalQuote {
    /// Ticker symbol
    pub symbol: String,
    /// Open price
    pub open: f64,
    /// High price
    pub high: f64,
    /// Low price
    pub low: f64,
    /// Current/last price
    pub price: f64,
    /// Trading volume
    pub volume: f64,
    /// Latest trading day
    pub latest_trading_day: String,
    /// Previous close
    pub previous_close: f64,
    /// Price change
    pub change: f64,
    /// Percent change (as string, e.g. `"1.23%"`)
    pub change_percent: String,
}

/// A single quote within a bulk quotes response.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct BulkQuote {
    /// Ticker symbol
    pub symbol: String,
    /// Open price
    pub open: Option<f64>,
    /// High price
    pub high: Option<f64>,
    /// Low price
    pub low: Option<f64>,
    /// Current/last price
    pub price: Option<f64>,
    /// Trading volume
    pub volume: Option<f64>,
    /// Latest trading day
    pub latest_trading_day: Option<String>,
    /// Previous close
    pub previous_close: Option<f64>,
    /// Price change
    pub change: Option<f64>,
    /// Percent change
    pub change_percent: Option<String>,
}

// ============================================================================
// Symbol search
// ============================================================================

/// A single match result from a symbol search.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct SymbolMatch {
    /// Ticker symbol
    pub symbol: String,
    /// Company/security name
    pub name: String,
    /// Asset type (e.g., `"Equity"`, `"ETF"`)
    pub asset_type: String,
    /// Stock exchange region
    pub region: String,
    /// Market open time
    pub market_open: String,
    /// Market close time
    pub market_close: String,
    /// Timezone
    pub timezone: String,
    /// Currency
    pub currency: String,
    /// Match score (0.0 to 1.0)
    pub match_score: f64,
}

// ============================================================================
// Market status
// ============================================================================

/// Status of a single market/exchange.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct MarketStatus {
    /// Market type (e.g., `"Equity"`, `"Forex"`)
    pub market_type: String,
    /// Region name
    pub region: String,
    /// Primary exchanges
    pub primary_exchanges: String,
    /// Local open time
    pub local_open: String,
    /// Local close time
    pub local_close: String,
    /// Current status (e.g., `"open"`, `"closed"`)
    pub current_status: String,
    /// Notes
    pub notes: String,
}

// ============================================================================
// Options
// ============================================================================

/// A single options contract.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct OptionContract {
    /// Contract ID
    pub contractid: String,
    /// Underlying symbol
    pub symbol: String,
    /// Expiration date
    pub expiration: String,
    /// Strike price
    pub strike: f64,
    /// Option type: `"call"` or `"put"`
    pub option_type: String,
    /// Last traded price
    pub last: Option<f64>,
    /// Current mark/mid price
    pub mark: Option<f64>,
    /// Bid price
    pub bid: Option<f64>,
    /// Bid size
    pub bid_size: Option<f64>,
    /// Ask price
    pub ask: Option<f64>,
    /// Ask size
    pub ask_size: Option<f64>,
    /// Trading volume
    pub volume: Option<f64>,
    /// Open interest
    pub open_interest: Option<f64>,
    /// Implied volatility
    pub implied_volatility: Option<f64>,
    /// Delta
    pub delta: Option<f64>,
    /// Gamma
    pub gamma: Option<f64>,
    /// Theta
    pub theta: Option<f64>,
    /// Vega
    pub vega: Option<f64>,
    /// Rho
    pub rho: Option<f64>,
}

/// Options chain response.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct OptionsChain {
    /// Underlying symbol
    pub symbol: String,
    /// Option contracts
    pub contracts: Vec<OptionContract>,
}

// ============================================================================
// Alpha Intelligence
// ============================================================================

/// A single news article with sentiment data.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct NewsArticle {
    /// Article title
    pub title: String,
    /// Article URL
    pub url: String,
    /// Published timestamp
    pub time_published: String,
    /// Source name
    pub source: String,
    /// Summary text
    pub summary: String,
    /// Overall sentiment score (-1.0 to 1.0)
    pub overall_sentiment_score: Option<f64>,
    /// Overall sentiment label
    pub overall_sentiment_label: Option<String>,
    /// Per-ticker sentiment
    pub ticker_sentiment: Vec<TickerSentiment>,
}

/// Sentiment data for a specific ticker within a news article.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct TickerSentiment {
    /// Ticker symbol
    pub ticker: String,
    /// Relevance score (0.0 to 1.0)
    pub relevance_score: Option<f64>,
    /// Sentiment score (-1.0 to 1.0)
    pub ticker_sentiment_score: Option<f64>,
    /// Sentiment label
    pub ticker_sentiment_label: Option<String>,
}

/// Earnings call transcript.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct EarningsCallTranscript {
    /// Ticker symbol
    pub symbol: String,
    /// Quarter identifier (e.g., `"2024Q1"`)
    pub quarter: String,
    /// Transcript text
    pub transcript: String,
}

/// A top gainer, loser, or most actively traded ticker.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct TopMoverTicker {
    /// Ticker symbol
    pub ticker: String,
    /// Current price
    pub price: String,
    /// Absolute change
    pub change_amount: String,
    /// Percentage change
    pub change_percentage: String,
    /// Trading volume
    pub volume: String,
}

/// Top gainers, losers, and most actively traded tickers.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct TopMovers {
    /// Last updated timestamp
    pub last_updated: String,
    /// Top gaining tickers
    pub top_gainers: Vec<TopMoverTicker>,
    /// Top losing tickers
    pub top_losers: Vec<TopMoverTicker>,
    /// Most actively traded tickers
    pub most_actively_traded: Vec<TopMoverTicker>,
}

// ============================================================================
// Fundamental data
// ============================================================================

/// Company overview / profile.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct CompanyOverview {
    /// Ticker symbol
    pub symbol: String,
    /// Asset type
    pub asset_type: Option<String>,
    /// Company name
    pub name: Option<String>,
    /// Company description
    pub description: Option<String>,
    /// Exchange
    pub exchange: Option<String>,
    /// Currency
    pub currency: Option<String>,
    /// Country
    pub country: Option<String>,
    /// Sector
    pub sector: Option<String>,
    /// Industry
    pub industry: Option<String>,
    /// Market capitalization
    pub market_capitalization: Option<f64>,
    /// Price-to-earnings ratio (trailing)
    pub pe_ratio: Option<f64>,
    /// Price-to-earnings-growth ratio
    pub peg_ratio: Option<f64>,
    /// Book value per share
    pub book_value: Option<f64>,
    /// Dividend per share
    pub dividend_per_share: Option<f64>,
    /// Dividend yield
    pub dividend_yield: Option<f64>,
    /// Earnings per share
    pub eps: Option<f64>,
    /// Revenue per share (TTM)
    pub revenue_per_share_ttm: Option<f64>,
    /// Profit margin
    pub profit_margin: Option<f64>,
    /// Operating margin (TTM)
    pub operating_margin_ttm: Option<f64>,
    /// Return on assets (TTM)
    pub return_on_assets_ttm: Option<f64>,
    /// Return on equity (TTM)
    pub return_on_equity_ttm: Option<f64>,
    /// Revenue (TTM)
    pub revenue_ttm: Option<f64>,
    /// Gross profit (TTM)
    pub gross_profit_ttm: Option<f64>,
    /// EBITDA
    pub ebitda: Option<f64>,
    /// 52-week high
    pub week_52_high: Option<f64>,
    /// 52-week low
    pub week_52_low: Option<f64>,
    /// 50-day moving average
    pub moving_average_50day: Option<f64>,
    /// 200-day moving average
    pub moving_average_200day: Option<f64>,
    /// Shares outstanding
    pub shares_outstanding: Option<f64>,
    /// Beta
    pub beta: Option<f64>,
    /// Forward PE
    pub forward_pe: Option<f64>,
    /// Price-to-sales ratio (TTM)
    pub price_to_sales_ratio_ttm: Option<f64>,
    /// Price-to-book ratio
    pub price_to_book_ratio: Option<f64>,
    /// Analyst target price
    pub analyst_target_price: Option<f64>,
    /// Analyst rating: strong buy count
    pub analyst_rating_strong_buy: Option<u32>,
    /// Analyst rating: buy count
    pub analyst_rating_buy: Option<u32>,
    /// Analyst rating: hold count
    pub analyst_rating_hold: Option<u32>,
    /// Analyst rating: sell count
    pub analyst_rating_sell: Option<u32>,
    /// Analyst rating: strong sell count
    pub analyst_rating_strong_sell: Option<u32>,
}

/// ETF profile and holdings.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct EtfProfile {
    /// ETF symbol
    pub symbol: String,
    /// ETF name
    pub name: Option<String>,
    /// Asset type
    pub asset_type: Option<String>,
    /// Net assets
    pub net_assets: Option<f64>,
    /// Expense ratio
    pub net_expense_ratio: Option<f64>,
    /// Turnover ratio
    pub portfolio_turnover: Option<f64>,
    /// Dividend yield
    pub dividend_yield: Option<f64>,
    /// Inception date
    pub inception_date: Option<String>,
    /// Top holdings
    pub holdings: Vec<EtfHolding>,
}

/// A single ETF holding.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct EtfHolding {
    /// Symbol of the held security
    pub symbol: Option<String>,
    /// Description
    pub description: Option<String>,
    /// Weight in the ETF portfolio (as percentage)
    pub weight: Option<f64>,
}

/// A single row in an income statement, balance sheet, or cash flow statement.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct FinancialReport {
    /// Fiscal date ending
    pub fiscal_date_ending: String,
    /// Reported currency
    pub reported_currency: String,
    /// All fields as key-value pairs (field names vary by statement type)
    #[serde(flatten)]
    pub fields: std::collections::HashMap<String, serde_json::Value>,
}

/// Financial statements (income statement, balance sheet, or cash flow).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct FinancialStatements {
    /// Ticker symbol
    pub symbol: String,
    /// Annual reports
    pub annual_reports: Vec<FinancialReport>,
    /// Quarterly reports
    pub quarterly_reports: Vec<FinancialReport>,
}

/// A single dividend payment event.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct DividendEvent {
    /// Ex-dividend date
    pub ex_dividend_date: Option<String>,
    /// Declaration date
    pub declaration_date: Option<String>,
    /// Record date
    pub record_date: Option<String>,
    /// Payment date
    pub payment_date: Option<String>,
    /// Dividend amount
    pub amount: Option<f64>,
}

/// A single stock split event.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct SplitEvent {
    /// Effective date
    pub effective_date: Option<String>,
    /// Split ratio (e.g., `"4:1"`)
    pub split_ratio: Option<String>,
}

/// Earnings data for a single quarter.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct EarningsData {
    /// Fiscal date ending
    pub fiscal_date_ending: Option<String>,
    /// Reported date
    pub reported_date: Option<String>,
    /// Reported EPS
    pub reported_eps: Option<f64>,
    /// Estimated EPS
    pub estimated_eps: Option<f64>,
    /// Surprise
    pub surprise: Option<f64>,
    /// Surprise percentage
    pub surprise_percentage: Option<f64>,
}

/// Earnings history with annual and quarterly data.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct EarningsHistory {
    /// Ticker symbol
    pub symbol: String,
    /// Annual earnings
    pub annual_earnings: Vec<EarningsData>,
    /// Quarterly earnings
    pub quarterly_earnings: Vec<EarningsData>,
}

/// A single earnings calendar entry.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct EarningsCalendarEntry {
    /// Ticker symbol
    pub symbol: String,
    /// Company name
    pub name: Option<String>,
    /// Report date
    pub report_date: Option<String>,
    /// Fiscal date ending
    pub fiscal_date_ending: Option<String>,
    /// Estimated EPS
    pub estimate: Option<f64>,
    /// Currency
    pub currency: Option<String>,
}

/// A single IPO calendar entry.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct IpoCalendarEntry {
    /// Ticker symbol
    pub symbol: Option<String>,
    /// Company name
    pub name: Option<String>,
    /// IPO date
    pub ipo_date: Option<String>,
    /// Price range (e.g., `"$15-$17"`)
    pub price_range: Option<String>,
    /// Exchange
    pub exchange: Option<String>,
}

/// Listing status entry.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct ListingEntry {
    /// Ticker symbol
    pub symbol: String,
    /// Security name
    pub name: Option<String>,
    /// Exchange
    pub exchange: Option<String>,
    /// Asset type
    pub asset_type: Option<String>,
    /// IPO date
    pub ipo_date: Option<String>,
    /// Delisting date (if applicable)
    pub delisting_date: Option<String>,
    /// Status (`"Active"` or `"Delisted"`)
    pub status: Option<String>,
}

// ============================================================================
// Forex
// ============================================================================

/// Real-time exchange rate between two currencies.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct ExchangeRate {
    /// Source currency code
    pub from_currency_code: String,
    /// Source currency name
    pub from_currency_name: String,
    /// Target currency code
    pub to_currency_code: String,
    /// Target currency name
    pub to_currency_name: String,
    /// Exchange rate
    pub exchange_rate: f64,
    /// Last refreshed timestamp
    pub last_refreshed: String,
    /// Bid price
    pub bid_price: f64,
    /// Ask price
    pub ask_price: f64,
}

/// A single forex time series data point.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct ForexEntry {
    /// Timestamp or date string
    pub timestamp: String,
    /// Opening price
    pub open: f64,
    /// Highest price
    pub high: f64,
    /// Lowest price
    pub low: f64,
    /// Closing price
    pub close: f64,
}

/// Forex time series response.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct ForexTimeSeries {
    /// Source currency code
    pub from_symbol: String,
    /// Target currency code
    pub to_symbol: String,
    /// Last refreshed timestamp
    pub last_refreshed: String,
    /// Data entries
    pub entries: Vec<ForexEntry>,
}

// ============================================================================
// Crypto
// ============================================================================

/// A single cryptocurrency time series data point.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct CryptoEntry {
    /// Timestamp or date string
    pub timestamp: String,
    /// Opening price
    pub open: f64,
    /// Highest price
    pub high: f64,
    /// Lowest price
    pub low: f64,
    /// Closing price
    pub close: f64,
    /// Trading volume
    pub volume: f64,
}

/// Cryptocurrency time series response.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct CryptoTimeSeries {
    /// Cryptocurrency symbol (e.g., `"BTC"`)
    pub symbol: String,
    /// Market/exchange currency (e.g., `"USD"`)
    pub market: String,
    /// Last refreshed timestamp
    pub last_refreshed: String,
    /// Data entries
    pub entries: Vec<CryptoEntry>,
}

// ============================================================================
// Commodities
// ============================================================================

/// A single commodity data point.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct CommodityDataPoint {
    /// Date string (YYYY-MM-DD)
    pub date: String,
    /// Value
    pub value: Option<f64>,
}

/// Commodity time series response.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct CommoditySeries {
    /// Commodity name/identifier
    pub name: String,
    /// Interval
    pub interval: String,
    /// Unit
    pub unit: String,
    /// Data points
    pub data: Vec<CommodityDataPoint>,
}

// ============================================================================
// Economic indicators
// ============================================================================

/// A single economic indicator data point.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct EconomicDataPoint {
    /// Date string (YYYY-MM-DD)
    pub date: String,
    /// Value
    pub value: Option<f64>,
}

/// Economic indicator time series response.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct EconomicSeries {
    /// Indicator name
    pub name: String,
    /// Interval
    pub interval: String,
    /// Unit
    pub unit: String,
    /// Data points
    pub data: Vec<EconomicDataPoint>,
}

// ============================================================================
// Technical indicators
// ============================================================================

/// A single technical indicator data point.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct IndicatorDataPoint {
    /// Timestamp or date string
    pub timestamp: String,
    /// Indicator values as key-value pairs (keys vary by indicator).
    /// e.g., for SMA: `{"SMA": 150.5}`, for BBANDS: `{"Real Upper Band": ..., "Real Middle Band": ..., "Real Lower Band": ...}`
    pub values: std::collections::HashMap<String, f64>,
}

/// Technical indicator response.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct TechnicalIndicator {
    /// Indicator function name (e.g., `"SMA"`, `"RSI"`, `"MACD"`)
    pub indicator: String,
    /// Symbol
    pub symbol: String,
    /// Last refreshed timestamp
    pub last_refreshed: String,
    /// Interval used
    pub interval: String,
    /// Data points
    pub data: Vec<IndicatorDataPoint>,
}
