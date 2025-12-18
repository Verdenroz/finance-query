use super::contract::OptionContract;
use serde::{Deserialize, Serialize};

/// Options chain data for a specific expiration
///
/// Note: This struct cannot be manually constructed - use `Ticker::options()` to obtain options data.
#[non_exhaustive]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OptionChain {
    /// Expiration date (Unix timestamp)
    pub expiration_date: i64,

    /// Whether all data is fetched
    pub has_mini_options: Option<bool>,

    /// Call options
    pub calls: Vec<OptionContract>,

    /// Put options
    pub puts: Vec<OptionContract>,
}

/// Quote data included with options response
///
/// Note: This struct cannot be manually constructed - obtain via `Ticker::options()`.
#[non_exhaustive]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OptionsQuote {
    /// Symbol
    pub symbol: String,

    /// Short name
    pub short_name: Option<String>,

    /// Regular market price
    pub regular_market_price: Option<f64>,

    /// Regular market time (Unix timestamp)
    pub regular_market_time: Option<i64>,
}
