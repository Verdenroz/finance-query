//! Search model for symbol search results

use serde::{Deserialize, Serialize};

/// Response wrapper for search endpoint
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SearchResponse {
    /// Search query count
    pub count: Option<i32>,
    /// Quote results
    pub quotes: Vec<SearchQuote>,
    /// News results
    pub news: Option<Vec<SearchNews>>,
    /// Total time in milliseconds
    pub total_time: Option<i64>,
    /// Timing breakdown
    pub timing_info: Option<serde_json::Value>,
}

/// A quote result from search
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SearchQuote {
    /// Stock symbol
    pub symbol: String,
    /// Short name
    pub short_name: Option<String>,
    /// Long name
    pub long_name: Option<String>,
    /// Quote type (EQUITY, ETF, etc.)
    pub quote_type: Option<String>,
    /// Exchange
    pub exchange: Option<String>,
    /// Exchange display name
    pub exch_disp: Option<String>,
    /// Type display name
    pub type_disp: Option<String>,
    /// Industry
    pub industry: Option<String>,
    /// Sector
    pub sector: Option<String>,
    /// Is Yahoo Finance equity
    #[serde(rename = "isYahooFinance")]
    pub is_yahoo_finance: Option<bool>,
    /// Relevance score
    pub score: Option<f64>,
    /// Dispaly sec industry
    pub disp_sec_ind: Option<String>,
}

/// A news result from search
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SearchNews {
    /// News UUID
    pub uuid: Option<String>,
    /// Title
    pub title: Option<String>,
    /// Publisher
    pub publisher: Option<String>,
    /// Link
    pub link: Option<String>,
    /// Provider publish time (Unix timestamp)
    pub provider_publish_time: Option<i64>,
    /// Type
    #[serde(rename = "type")]
    pub news_type: Option<String>,
    /// Thumbnail
    pub thumbnail: Option<NewsThumbnail>,
    /// Related tickers
    pub related_tickers: Option<Vec<String>>,
}

/// Thumbnail for news
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewsThumbnail {
    /// Resolutions
    pub resolutions: Option<Vec<ThumbnailResolution>>,
}

/// Thumbnail resolution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThumbnailResolution {
    /// URL
    pub url: Option<String>,
    /// Width
    pub width: Option<i32>,
    /// Height
    pub height: Option<i32>,
    /// Tag
    pub tag: Option<String>,
}

impl SearchResponse {
    /// Parse from JSON value
    pub fn from_json(value: serde_json::Value) -> Result<Self, serde_json::Error> {
        serde_json::from_value(value)
    }
}
