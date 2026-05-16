//! Shared types for Polygon.io API responses.
#![allow(dead_code)]

use serde::{Deserialize, Serialize};

// ============================================================================
// Pagination and response envelope
// ============================================================================

/// Generic paginated response envelope from Polygon.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct PaginatedResponseDTO<T> {
    /// Result items (may be absent on empty responses).
    pub results: Option<Vec<T>>,
    /// Response status (e.g., `"OK"`, `"DELAYED"`).
    pub status: Option<String>,
    /// Unique request identifier.
    pub request_id: Option<String>,
    /// Number of results in this page.
    #[serde(rename = "resultsCount")]
    pub results_count: Option<usize>,
    /// Total count across all pages.
    pub count: Option<usize>,
    /// Cursor URL for the next page of results.
    pub next_url: Option<String>,
    /// Ticker symbol (present on some endpoints).
    pub ticker: Option<String>,
    /// Whether results are adjusted for splits.
    pub adjusted: Option<bool>,
    /// Query count (aggregates).
    #[serde(rename = "queryCount")]
    pub query_count: Option<usize>,
}

// ============================================================================
// Enums
// ============================================================================

/// Timespan unit for aggregate bar requests.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Timespan {
    /// 1 second
    Second,
    /// 1 minute
    Minute,
    /// 1 hour
    Hour,
    /// 1 day
    Day,
    /// 1 week
    Week,
    /// 1 month
    Month,
    /// 1 quarter
    Quarter,
    /// 1 year
    Year,
}

impl Timespan {
    /// Convert to Polygon API parameter string.
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Second => "second",
            Self::Minute => "minute",
            Self::Hour => "hour",
            Self::Day => "day",
            Self::Week => "week",
            Self::Month => "month",
            Self::Quarter => "quarter",
            Self::Year => "year",
        }
    }
}

/// Sort direction for paginated results.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Sort {
    /// Ascending (oldest first)
    Asc,
    /// Descending (newest first)
    Desc,
}

impl Sort {
    /// Convert to Polygon API parameter string.
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Asc => "asc",
            Self::Desc => "desc",
        }
    }
}

/// Order parameter for some endpoints.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Order {
    /// Ascending
    Asc,
    /// Descending
    Desc,
}

impl Order {
    /// Convert to Polygon API parameter string.
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Asc => "asc",
            Self::Desc => "desc",
        }
    }
}

// ============================================================================
// Shared OHLCV bar (reused across stocks, options, forex, crypto, indices, futures)
// ============================================================================

/// OHLCV aggregate bar from Polygon.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct AggBarDTO {
    /// Open price.
    #[serde(rename = "o")]
    pub open: f64,
    /// High price.
    #[serde(rename = "h")]
    pub high: f64,
    /// Low price.
    #[serde(rename = "l")]
    pub low: f64,
    /// Close price.
    #[serde(rename = "c")]
    pub close: f64,
    /// Trading volume.
    #[serde(rename = "v")]
    pub volume: f64,
    /// Volume-weighted average price.
    #[serde(rename = "vw")]
    pub vwap: Option<f64>,
    /// Unix millisecond timestamp of the start of the bar.
    #[serde(rename = "t")]
    pub timestamp: i64,
    /// Number of transactions in this bar.
    #[serde(rename = "n")]
    pub transactions: Option<u64>,
    /// Whether this bar is an OTC trade.
    #[serde(rename = "otc")]
    pub otc: Option<bool>,
}

/// Aggregate response with ticker metadata.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct AggregateResponseDTO {
    /// Ticker symbol.
    pub ticker: Option<String>,
    /// Response status.
    pub status: Option<String>,
    /// Whether results are adjusted for splits.
    pub adjusted: Option<bool>,
    /// Number of results in this response.
    #[serde(rename = "resultsCount")]
    pub results_count: Option<usize>,
    /// Query count.
    #[serde(rename = "queryCount")]
    pub query_count: Option<usize>,
    /// Request identifier.
    pub request_id: Option<String>,
    /// Aggregate bars.
    pub results: Option<Vec<AggBarDTO>>,
    /// Next page URL.
    pub next_url: Option<String>,
}

/// Daily open/close response for a single ticker.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct DailyOpenCloseDTO {
    /// Ticker symbol.
    pub symbol: Option<String>,
    /// Date (YYYY-MM-DD).
    pub from: Option<String>,
    /// Open price.
    pub open: Option<f64>,
    /// High price.
    pub high: Option<f64>,
    /// Low price.
    pub low: Option<f64>,
    /// Close price.
    pub close: Option<f64>,
    /// Volume.
    pub volume: Option<f64>,
    /// After-hours price.
    #[serde(rename = "afterHours")]
    pub after_hours: Option<f64>,
    /// Pre-market price.
    #[serde(rename = "preMarket")]
    pub pre_market: Option<f64>,
}

// ============================================================================
// TradeDTO and quote types
// ============================================================================

/// A single trade.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct TradeDTO {
    /// TradeDTO conditions.
    pub conditions: Option<Vec<i32>>,
    /// TradeDTO correction indicator.
    pub correction: Option<i32>,
    /// Exchange ID.
    pub exchange: Option<i32>,
    /// TradeDTO ID.
    pub id: Option<String>,
    /// Participant timestamp (nanoseconds).
    pub participant_timestamp: Option<i64>,
    /// Price.
    pub price: Option<f64>,
    /// Sequence number.
    pub sequence_number: Option<i64>,
    /// SIP timestamp (nanoseconds).
    pub sip_timestamp: Option<i64>,
    /// TradeDTO size.
    pub size: Option<f64>,
    /// Tape.
    pub tape: Option<i32>,
    /// TRF ID.
    pub trf_id: Option<i32>,
    /// TRF timestamp.
    pub trf_timestamp: Option<i64>,
}

/// A single last-trade response.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct LastTradeDTO {
    /// Ticker symbol.
    #[serde(rename = "T")]
    pub ticker: Option<String>,
    /// Conditions.
    pub conditions: Option<Vec<i32>>,
    /// Correction.
    pub correction: Option<i32>,
    /// Exchange ID.
    pub exchange: Option<i32>,
    /// TradeDTO ID.
    pub id: Option<String>,
    /// Participant timestamp.
    pub participant_timestamp: Option<i64>,
    /// Price.
    pub price: Option<f64>,
    /// Sequence number.
    pub sequence_number: Option<i64>,
    /// SIP timestamp.
    pub sip_timestamp: Option<i64>,
    /// Size.
    pub size: Option<f64>,
    /// Tape.
    pub tape: Option<i32>,
    /// TRF ID.
    pub trf_id: Option<i32>,
    /// TRF timestamp.
    pub trf_timestamp: Option<i64>,
}

/// NBBO quote.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct QuoteDTO {
    /// Ask exchange.
    pub ask_exchange: Option<i32>,
    /// Ask price.
    pub ask_price: Option<f64>,
    /// Ask size.
    pub ask_size: Option<f64>,
    /// Bid exchange.
    pub bid_exchange: Option<i32>,
    /// Bid price.
    pub bid_price: Option<f64>,
    /// Bid size.
    pub bid_size: Option<f64>,
    /// Conditions.
    pub conditions: Option<Vec<i32>>,
    /// Indicators.
    pub indicators: Option<Vec<i32>>,
    /// Participant timestamp.
    pub participant_timestamp: Option<i64>,
    /// Sequence number.
    pub sequence_number: Option<i64>,
    /// SIP timestamp.
    pub sip_timestamp: Option<i64>,
    /// Tape.
    pub tape: Option<i32>,
    /// TRF timestamp.
    pub trf_timestamp: Option<i64>,
}

/// Last quote (NBBO) response wrapper.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct LastQuoteResponseDTO {
    /// Request ID.
    pub request_id: Option<String>,
    /// Response status.
    pub status: Option<String>,
    /// The last quote.
    pub results: Option<QuoteDTO>,
}

/// Last trade response wrapper.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct LastTradeResponseDTO {
    /// Request ID.
    pub request_id: Option<String>,
    /// Response status.
    pub status: Option<String>,
    /// The last trade.
    pub results: Option<LastTradeDTO>,
}

// ============================================================================
// Snapshot types
// ============================================================================

/// Day aggregate data within a snapshot.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct SnapshotAggDTO {
    /// Open price.
    #[serde(rename = "o")]
    pub open: Option<f64>,
    /// High price.
    #[serde(rename = "h")]
    pub high: Option<f64>,
    /// Low price.
    #[serde(rename = "l")]
    pub low: Option<f64>,
    /// Close price.
    #[serde(rename = "c")]
    pub close: Option<f64>,
    /// Volume.
    #[serde(rename = "v")]
    pub volume: Option<f64>,
    /// VWAP.
    #[serde(rename = "vw")]
    pub vwap: Option<f64>,
    /// Timestamp.
    #[serde(rename = "t")]
    pub timestamp: Option<i64>,
    /// Transactions.
    #[serde(rename = "n")]
    pub transactions: Option<u64>,
}

/// A single ticker snapshot.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct TickerSnapshotDTO {
    /// Ticker symbol.
    pub ticker: Option<String>,
    /// Today's change amount.
    #[serde(rename = "todaysChange")]
    pub todays_change: Option<f64>,
    /// Today's change percent.
    #[serde(rename = "todaysChangePerc")]
    pub todays_change_perc: Option<f64>,
    /// Updated timestamp (nanoseconds).
    pub updated: Option<i64>,
    /// Current day aggregate.
    pub day: Option<SnapshotAggDTO>,
    /// Previous day aggregate.
    #[serde(rename = "prevDay")]
    pub prev_day: Option<SnapshotAggDTO>,
    /// Last trade.
    #[serde(rename = "lastTrade")]
    pub last_trade: Option<LastTradeDTO>,
    /// Last quote.
    #[serde(rename = "lastQuote")]
    pub last_quote: Option<QuoteDTO>,
    /// Minute aggregate.
    pub min: Option<SnapshotAggDTO>,
}

/// Snapshot response for a single ticker.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct SingleSnapshotResponseDTO {
    /// Request ID.
    pub request_id: Option<String>,
    /// Response status.
    pub status: Option<String>,
    /// Snapshot data.
    pub ticker: Option<TickerSnapshotDTO>,
}

/// Snapshot response for multiple tickers.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct SnapshotsResponseDTO {
    /// Response status.
    pub status: Option<String>,
    /// Request ID.
    pub request_id: Option<String>,
    /// Number of results.
    pub count: Option<usize>,
    /// Ticker snapshots.
    pub tickers: Option<Vec<TickerSnapshotDTO>>,
}

// ============================================================================
// Technical indicator types
// ============================================================================

/// A single technical indicator data point.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct IndicatorValueDTO {
    /// Timestamp.
    pub timestamp: Option<i64>,
    /// Indicator value.
    pub value: Option<f64>,
    /// Signal value (MACD).
    pub signal: Option<f64>,
    /// Histogram value (MACD).
    pub histogram: Option<f64>,
}

/// Technical indicator response.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct IndicatorResponseDTO {
    /// Response status.
    pub status: Option<String>,
    /// Request ID.
    pub request_id: Option<String>,
    /// Indicator results.
    pub results: Option<IndicatorResultsDTO>,
    /// Next page URL.
    pub next_url: Option<String>,
}

/// Nested indicator results containing underlying data and values.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct IndicatorResultsDTO {
    /// Underlying aggregate bars.
    pub underlying: Option<IndicatorUnderlyingDTO>,
    /// Indicator values.
    pub values: Option<Vec<IndicatorValueDTO>>,
}

/// Underlying data for indicator responses.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct IndicatorUnderlyingDTO {
    /// Underlying aggregate bars URL.
    pub url: Option<String>,
    /// Underlying aggregates.
    pub aggregates: Option<Vec<AggBarDTO>>,
}

// ============================================================================
// Optional aggregate parameters
// ============================================================================

/// Optional parameters for aggregate bar requests.
#[derive(Debug, Clone, Default)]
pub struct AggregateParams {
    /// Whether results are adjusted for splits. Default: true.
    pub adjusted: Option<bool>,
    /// Sort direction.
    pub sort: Option<Sort>,
    /// Maximum number of results. Default: 5000, max: 50000.
    pub limit: Option<u32>,
}
