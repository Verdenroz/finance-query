//! Shared types for Financial Modeling Prep API responses.

use serde::{Deserialize, Serialize};

// ============================================================================
// Enums
// ============================================================================

/// Financial statement period.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Period {
    /// Annual financial statements.
    Annual,
    /// Quarterly financial statements.
    Quarter,
}

impl Period {
    /// Convert to FMP API parameter string.
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Annual => "annual",
            Self::Quarter => "quarter",
        }
    }
}

// ============================================================================
// Quote types
// ============================================================================

/// Real-time quote from FMP.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct FmpQuote {
    /// Ticker symbol.
    pub symbol: String,
    /// Company name.
    pub name: Option<String>,
    /// Current price.
    pub price: Option<f64>,
    /// Price change.
    pub change: Option<f64>,
    /// Price change percentage.
    #[serde(rename = "changesPercentage")]
    pub changes_percentage: Option<f64>,
    /// Day low.
    #[serde(rename = "dayLow")]
    pub day_low: Option<f64>,
    /// Day high.
    #[serde(rename = "dayHigh")]
    pub day_high: Option<f64>,
    /// 52-week low.
    #[serde(rename = "yearLow")]
    pub year_low: Option<f64>,
    /// 52-week high.
    #[serde(rename = "yearHigh")]
    pub year_high: Option<f64>,
    /// Market capitalization.
    #[serde(rename = "marketCap")]
    pub market_cap: Option<f64>,
    /// Trading volume.
    pub volume: Option<f64>,
    /// Average volume.
    #[serde(rename = "avgVolume")]
    pub avg_volume: Option<f64>,
    /// Open price.
    pub open: Option<f64>,
    /// Previous close price.
    #[serde(rename = "previousClose")]
    pub previous_close: Option<f64>,
    /// Earnings per share.
    pub eps: Option<f64>,
    /// Price-to-earnings ratio.
    pub pe: Option<f64>,
    /// Unix timestamp.
    pub timestamp: Option<i64>,
    /// Exchange name.
    pub exchange: Option<String>,
    /// Earnings announcement date.
    #[serde(rename = "earningsAnnouncement")]
    pub earnings_announcement: Option<String>,
}

// ============================================================================
// Historical price types
// ============================================================================

/// A single historical daily price point.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct HistoricalPrice {
    /// Date (YYYY-MM-DD).
    pub date: Option<String>,
    /// Open price.
    pub open: Option<f64>,
    /// High price.
    pub high: Option<f64>,
    /// Low price.
    pub low: Option<f64>,
    /// Close price.
    pub close: Option<f64>,
    /// Adjusted close price.
    #[serde(rename = "adjClose")]
    pub adj_close: Option<f64>,
    /// Trading volume.
    pub volume: Option<f64>,
    /// Unadjusted volume.
    #[serde(rename = "unadjustedVolume")]
    pub unadjusted_volume: Option<f64>,
    /// Price change.
    pub change: Option<f64>,
    /// Price change percentage.
    #[serde(rename = "changePercent")]
    pub change_percent: Option<f64>,
    /// Volume-weighted average price.
    pub vwap: Option<f64>,
    /// Human-readable date label.
    pub label: Option<String>,
    /// Change over time (cumulative).
    #[serde(rename = "changeOverTime")]
    pub change_over_time: Option<f64>,
}

/// Historical price response wrapper (FMP wraps daily history in this).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct HistoricalPriceResponse {
    /// Ticker symbol.
    pub symbol: Option<String>,
    /// Historical price data points.
    pub historical: Vec<HistoricalPrice>,
}

/// A single intraday price point (1min, 5min, 15min, 30min, 1hour, 4hour).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct IntradayPrice {
    /// Datetime string.
    pub date: Option<String>,
    /// Open price.
    pub open: Option<f64>,
    /// High price.
    pub high: Option<f64>,
    /// Low price.
    pub low: Option<f64>,
    /// Close price.
    pub close: Option<f64>,
    /// Trading volume.
    pub volume: Option<f64>,
}
