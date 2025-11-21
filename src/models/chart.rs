//! Chart model for historical price data

use serde::{Deserialize, Serialize};

/// Response wrapper for chart endpoint
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChartResponse {
    /// Chart container
    pub chart: ChartContainer,
}

/// Container for chart results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChartContainer {
    /// Chart results
    pub result: Option<Vec<ChartResult>>,
    /// Error if any
    pub error: Option<serde_json::Value>,
}

/// Chart result for a single symbol
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChartResult {
    /// Metadata about the chart
    pub meta: ChartMeta,
    /// Timestamps for each data point
    pub timestamp: Option<Vec<i64>>,
    /// Price indicators (OHLCV)
    pub indicators: ChartIndicators,
}

/// Metadata for chart data
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

/// Chart indicators containing OHLCV data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChartIndicators {
    /// Quote data (OHLCV)
    pub quote: Vec<QuoteIndicator>,
    /// Adjusted close data
    #[serde(rename = "adjclose")]
    pub adj_close: Option<Vec<AdjCloseIndicator>>,
}

/// OHLCV quote indicator
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuoteIndicator {
    /// Open prices
    pub open: Option<Vec<Option<f64>>>,
    /// High prices
    pub high: Option<Vec<Option<f64>>>,
    /// Low prices
    pub low: Option<Vec<Option<f64>>>,
    /// Close prices
    pub close: Option<Vec<Option<f64>>>,
    /// Volume
    pub volume: Option<Vec<Option<i64>>>,
}

/// Adjusted close indicator
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AdjCloseIndicator {
    /// Adjusted close prices
    pub adj_close: Option<Vec<Option<f64>>>,
}

/// A single OHLCV candle/bar
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Candle {
    /// Timestamp (Unix)
    pub timestamp: i64,
    /// Open price
    pub open: f64,
    /// High price
    pub high: f64,
    /// Low price
    pub low: f64,
    /// Close price
    pub close: f64,
    /// Volume
    pub volume: i64,
    /// Adjusted close (if available)
    pub adj_close: Option<f64>,
}

impl ChartResponse {
    /// Parse from JSON value
    pub fn from_json(value: serde_json::Value) -> Result<Self, serde_json::Error> {
        serde_json::from_value(value)
    }
}

impl ChartResult {
    /// Convert chart result to a vector of candles
    pub fn to_candles(&self) -> Vec<Candle> {
        let timestamps = match &self.timestamp {
            Some(ts) => ts,
            None => return vec![],
        };

        let quote = match self.indicators.quote.first() {
            Some(q) => q,
            None => return vec![],
        };

        let opens = quote.open.as_ref();
        let highs = quote.high.as_ref();
        let lows = quote.low.as_ref();
        let closes = quote.close.as_ref();
        let volumes = quote.volume.as_ref();
        let adj_closes = self
            .indicators
            .adj_close
            .as_ref()
            .and_then(|ac| ac.first())
            .and_then(|ac| ac.adj_close.as_ref());

        timestamps
            .iter()
            .enumerate()
            .filter_map(|(i, &ts)| {
                let open = opens.and_then(|o| o.get(i)).and_then(|v| *v)?;
                let high = highs.and_then(|h| h.get(i)).and_then(|v| *v)?;
                let low = lows.and_then(|l| l.get(i)).and_then(|v| *v)?;
                let close = closes.and_then(|c| c.get(i)).and_then(|v| *v)?;
                let volume = volumes.and_then(|v| v.get(i)).and_then(|v| *v).unwrap_or(0);
                let adj_close = adj_closes.and_then(|ac| ac.get(i)).and_then(|v| *v);

                Some(Candle {
                    timestamp: ts,
                    open,
                    high,
                    low,
                    close,
                    volume,
                    adj_close,
                })
            })
            .collect()
    }
}
