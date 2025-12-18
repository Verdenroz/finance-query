/// Chart Metadata module
///
/// Contains metadata about chart data including symbol, exchange, timezone, and price information.
use serde::{Deserialize, Serialize};

/// Metadata for chart data
///
/// Note: This struct cannot be manually constructed - obtain via `Ticker::chart()`.
#[non_exhaustive]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ChartMeta {
    /// Stock symbol
    pub symbol: String,
    /// Currency
    pub currency: Option<String>,
    /// Exchange name
    pub exchange_name: Option<String>,
    /// Full exchange name
    pub full_exchange_name: Option<String>,
    /// Instrument type
    pub instrument_type: Option<String>,
    /// First trade date (Unix timestamp)
    pub first_trade_date: Option<i64>,
    /// Regular market time (Unix timestamp)
    pub regular_market_time: Option<i64>,
    /// Has pre/post market data
    pub has_pre_post_market_data: Option<bool>,
    /// GMT offset
    pub gmt_offset: Option<i64>,
    /// Timezone
    pub timezone: Option<String>,
    /// Exchange timezone name
    pub exchange_timezone_name: Option<String>,
    /// Regular market price
    pub regular_market_price: Option<f64>,
    /// Fifty two week high
    pub fifty_two_week_high: Option<f64>,
    /// Fifty two week low
    pub fifty_two_week_low: Option<f64>,
    /// Regular market day high
    pub regular_market_day_high: Option<f64>,
    /// Regular market day low
    pub regular_market_day_low: Option<f64>,
    /// Regular market volume
    pub regular_market_volume: Option<i64>,
    /// Chart previous close
    pub chart_previous_close: Option<f64>,
    /// Previous close
    pub previous_close: Option<f64>,
    /// Price hint (decimal places)
    pub price_hint: Option<i32>,
    /// Data granularity
    pub data_granularity: Option<String>,
    /// Range
    pub range: Option<String>,
}
