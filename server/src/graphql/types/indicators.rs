//! GraphQL types for technical indicators.
//!
//! Mirrors `finance_query::indicators::summary::IndicatorsSummary` and its
//! nested sub-types.

use super::batch::GqlBatchError;
use async_graphql::SimpleObject;
use serde::Deserialize;

// ── Nested sub-types ───────────────────────────────────────────────────────

/// Mirrors `finance_query::indicators::summary::StochasticData`, whose `k`/`d`
/// fields serialize as `"%K"`/`"%D"` (a custom per-field rename, not plain
/// camelCase) — must match exactly for deserialization to find them.
#[derive(SimpleObject, Deserialize, Debug, Clone, Default)]
#[graphql(rename_fields = "camelCase")]
#[serde(default)]
pub struct GqlStochasticData {
    #[serde(rename = "%K")]
    pub k: Option<f64>,
    #[serde(rename = "%D")]
    pub d: Option<f64>,
}

#[derive(SimpleObject, Deserialize, Debug, Clone, Default)]
#[graphql(rename_fields = "camelCase")]
#[serde(rename_all = "camelCase", default)]
pub struct GqlMacdData {
    pub macd: Option<f64>,
    pub signal: Option<f64>,
    pub histogram: Option<f64>,
}

#[derive(SimpleObject, Deserialize, Debug, Clone, Default)]
#[graphql(rename_fields = "camelCase")]
#[serde(rename_all = "camelCase", default)]
pub struct GqlAroonData {
    pub aroon_up: Option<f64>,
    pub aroon_down: Option<f64>,
}

#[derive(SimpleObject, Deserialize, Debug, Clone, Default)]
#[graphql(rename_fields = "camelCase")]
#[serde(rename_all = "camelCase", default)]
pub struct GqlBollingerBandsData {
    pub upper: Option<f64>,
    pub middle: Option<f64>,
    pub lower: Option<f64>,
}

#[derive(SimpleObject, Deserialize, Debug, Clone, Default)]
#[graphql(rename_fields = "camelCase")]
#[serde(rename_all = "camelCase", default)]
pub struct GqlSuperTrendData {
    pub value: Option<f64>,
    pub trend: Option<String>,
}

#[derive(SimpleObject, Deserialize, Debug, Clone, Default)]
#[graphql(rename_fields = "camelCase")]
#[serde(rename_all = "camelCase", default)]
pub struct GqlIchimokuData {
    pub conversion_line: Option<f64>,
    pub base_line: Option<f64>,
    pub leading_span_a: Option<f64>,
    pub leading_span_b: Option<f64>,
    pub lagging_span: Option<f64>,
}

#[derive(SimpleObject, Deserialize, Debug, Clone, Default)]
#[graphql(rename_fields = "camelCase")]
#[serde(rename_all = "camelCase", default)]
pub struct GqlKeltnerChannelsData {
    pub upper: Option<f64>,
    pub middle: Option<f64>,
    pub lower: Option<f64>,
}

#[derive(SimpleObject, Deserialize, Debug, Clone, Default)]
#[graphql(rename_fields = "camelCase")]
#[serde(rename_all = "camelCase", default)]
pub struct GqlDonchianChannelsData {
    pub upper: Option<f64>,
    pub middle: Option<f64>,
    pub lower: Option<f64>,
}

#[derive(SimpleObject, Deserialize, Debug, Clone, Default)]
#[graphql(rename_fields = "camelCase")]
#[serde(rename_all = "camelCase", default)]
pub struct GqlBullBearPowerData {
    pub bull_power: Option<f64>,
    pub bear_power: Option<f64>,
}

#[derive(SimpleObject, Deserialize, Debug, Clone, Default)]
#[graphql(rename_fields = "camelCase")]
#[serde(rename_all = "camelCase", default)]
pub struct GqlElderRayData {
    pub bull_power: Option<f64>,
    pub bear_power: Option<f64>,
}

// ── Main indicators summary ────────────────────────────────────────────────

/// All technical indicators for a symbol (latest values).
///
/// Mirrors the library's `IndicatorsSummary`, which itself has
/// `#[serde(rename_all = "camelCase")]` (its JSON keys are camelCase, e.g.
/// `"sma10"`) — this type must match that for deserialization, in addition
/// to using the same convention for its own GraphQL-facing field names.
#[derive(SimpleObject, Deserialize, Default, Debug, Clone)]
#[graphql(rename_fields = "camelCase")]
#[serde(rename_all = "camelCase", default)]
pub struct GqlIndicatorsSummary {
    // Moving Averages — Simple
    pub sma_10: Option<f64>,
    pub sma_20: Option<f64>,
    pub sma_50: Option<f64>,
    pub sma_100: Option<f64>,
    pub sma_200: Option<f64>,
    // Moving Averages — Exponential
    pub ema_10: Option<f64>,
    pub ema_20: Option<f64>,
    pub ema_50: Option<f64>,
    pub ema_100: Option<f64>,
    pub ema_200: Option<f64>,
    // Moving Averages — Weighted
    pub wma_10: Option<f64>,
    pub wma_20: Option<f64>,
    pub wma_50: Option<f64>,
    pub wma_100: Option<f64>,
    pub wma_200: Option<f64>,
    // Advanced Moving Averages
    pub dema_20: Option<f64>,
    pub tema_20: Option<f64>,
    pub hma_20: Option<f64>,
    pub vwma_20: Option<f64>,
    pub alma_9: Option<f64>,
    pub mcginley_dynamic_20: Option<f64>,
    // Momentum Oscillators
    pub rsi_14: Option<f64>,
    pub stochastic: Option<GqlStochasticData>,
    pub cci_20: Option<f64>,
    pub williams_r_14: Option<f64>,
    pub stochastic_rsi: Option<GqlStochasticData>,
    pub roc_12: Option<f64>,
    pub momentum_10: Option<f64>,
    pub cmo_14: Option<f64>,
    pub awesome_oscillator: Option<f64>,
    pub coppock_curve: Option<f64>,
    // Trend Indicators
    pub macd: Option<GqlMacdData>,
    pub adx_14: Option<f64>,
    pub aroon: Option<GqlAroonData>,
    pub supertrend: Option<GqlSuperTrendData>,
    pub ichimoku: Option<GqlIchimokuData>,
    pub parabolic_sar: Option<f64>,
    pub bull_bear_power: Option<GqlBullBearPowerData>,
    pub elder_ray_index: Option<GqlElderRayData>,
    // Volatility Indicators
    pub bollinger_bands: Option<GqlBollingerBandsData>,
    pub atr_14: Option<f64>,
    pub keltner_channels: Option<GqlKeltnerChannelsData>,
    pub donchian_channels: Option<GqlDonchianChannelsData>,
    pub true_range: Option<f64>,
    pub choppiness_index_14: Option<f64>,
    // Volume Indicators
    pub obv: Option<f64>,
    pub mfi_14: Option<f64>,
    pub cmf_20: Option<f64>,
    pub chaikin_oscillator: Option<f64>,
    pub accumulation_distribution: Option<f64>,
    pub vwap: Option<f64>,
    pub balance_of_power: Option<f64>,
}

/// Wraps a symbol name with its indicators, used by the batch root field.
#[derive(SimpleObject, Debug, Clone)]
#[graphql(rename_fields = "camelCase")]
pub struct GqlSymbolIndicators {
    pub symbol: String,
    pub indicators: GqlIndicatorsSummary,
}

/// Result of the batch `indicatorsBatch` root field: successfully computed
/// indicators plus any per-symbol fetch errors.
#[derive(SimpleObject, Debug, Clone)]
#[graphql(rename_fields = "camelCase")]
pub struct GqlIndicatorsBatch {
    pub indicators: Vec<GqlSymbolIndicators>,
    pub errors: Vec<GqlBatchError>,
}
