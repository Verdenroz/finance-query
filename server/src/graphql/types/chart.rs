//! GraphQL types for chart / OHLCV candle data.

use super::batch::GqlBatchError;
use async_graphql::SimpleObject;
use serde::Deserialize;

/// A single OHLCV candle / price bar.
#[derive(SimpleObject, Deserialize, Debug, Clone)]
#[graphql(rename_fields = "camelCase")]
#[serde(rename_all = "camelCase")]
pub struct GqlCandle {
    pub timestamp: i64,
    pub open: f64,
    pub high: f64,
    pub low: f64,
    pub close: f64,
    pub volume: i64,
    pub adj_close: Option<f64>,
}

/// Metadata associated with a chart response.
#[derive(SimpleObject, Deserialize, Debug, Clone, Default)]
#[graphql(rename_fields = "camelCase")]
#[serde(rename_all = "camelCase", default)]
pub struct GqlChartMeta {
    pub symbol: String,
    pub currency: Option<String>,
    pub exchange_name: Option<String>,
    pub full_exchange_name: Option<String>,
    pub instrument_type: Option<String>,
    pub first_trade_date: Option<i64>,
    pub regular_market_time: Option<i64>,
    pub has_pre_post_market_data: Option<bool>,
    pub gmt_offset: Option<i64>,
    pub timezone: Option<String>,
    pub exchange_timezone_name: Option<String>,
    pub regular_market_price: Option<f64>,
    pub fifty_two_week_high: Option<f64>,
    pub fifty_two_week_low: Option<f64>,
    pub regular_market_day_high: Option<f64>,
    pub regular_market_day_low: Option<f64>,
    pub regular_market_volume: Option<i64>,
    pub chart_previous_close: Option<f64>,
    pub previous_close: Option<f64>,
    pub price_hint: Option<i32>,
    pub data_granularity: Option<String>,
    pub range: Option<String>,
}

/// Historical chart data for a single symbol.
#[derive(SimpleObject, Deserialize, Debug, Clone)]
#[graphql(rename_fields = "camelCase")]
#[serde(rename_all = "camelCase")]
pub struct GqlChart {
    pub symbol: String,
    pub meta: GqlChartMeta,
    pub candles: Vec<GqlCandle>,
}

/// Wraps a symbol name with its chart data, used by the batch `charts` root field.
#[derive(SimpleObject, Deserialize, Debug, Clone)]
#[graphql(rename_fields = "camelCase")]
#[serde(rename_all = "camelCase")]
pub struct GqlSymbolChart {
    pub symbol: String,
    pub chart: GqlChart,
}

/// Result of the batch `charts` root field: successfully fetched charts plus
/// any per-symbol fetch errors.
#[derive(SimpleObject, Debug, Clone)]
#[graphql(rename_fields = "camelCase")]
pub struct GqlChartsBatch {
    pub charts: Vec<GqlSymbolChart>,
    pub errors: Vec<GqlBatchError>,
}

/// Lightweight sparkline data for a single symbol (close prices only).
#[derive(SimpleObject, Deserialize, Debug, Clone)]
#[graphql(rename_fields = "camelCase")]
#[serde(rename_all = "camelCase")]
pub struct GqlSpark {
    pub symbol: String,
    pub meta: GqlChartMeta,
    pub timestamps: Vec<i64>,
    pub closes: Vec<f64>,
    pub interval: Option<String>,
    pub range: Option<String>,
}

/// Result of the batch `spark` root field: successfully fetched sparklines
/// plus any per-symbol fetch errors.
#[derive(SimpleObject, Debug, Clone)]
#[graphql(rename_fields = "camelCase")]
pub struct GqlSparkBatch {
    pub sparks: Vec<GqlSpark>,
    pub errors: Vec<GqlBatchError>,
}
