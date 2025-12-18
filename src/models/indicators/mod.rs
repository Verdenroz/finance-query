//! Internal technical indicators calculation module.
//!
//! This module is NOT part of the public API. Users access indicators through
//! `Ticker::indicators()` and `AsyncTicker::indicators()` methods only.

mod momentum;
mod moving_avg;
mod trend;
mod volatility;
mod volume;

use crate::models::chart::Candle;

/// Calculate all technical indicators from candle data.
///
/// Returns the latest values for all implemented indicators.
pub(crate) fn calculate_indicators(candles: &[Candle]) -> IndicatorsSummary {
    if candles.is_empty() {
        return IndicatorsSummary::default();
    }

    let (closes, highs, lows, opens, volumes) = prepare_data(candles);

    IndicatorsSummary {
        // === MOVING AVERAGES ===
        // Simple Moving Averages
        sma_10: moving_avg::sma(&closes, 10),
        sma_20: moving_avg::sma(&closes, 20),
        sma_50: moving_avg::sma(&closes, 50),
        sma_100: moving_avg::sma(&closes, 100),
        sma_200: moving_avg::sma(&closes, 200),
        // Exponential Moving Averages
        ema_10: moving_avg::ema(&closes, 10),
        ema_20: moving_avg::ema(&closes, 20),
        ema_50: moving_avg::ema(&closes, 50),
        ema_100: moving_avg::ema(&closes, 100),
        ema_200: moving_avg::ema(&closes, 200),
        // Weighted Moving Averages
        wma_10: moving_avg::wma(&closes, 10),
        wma_20: moving_avg::wma(&closes, 20),
        wma_50: moving_avg::wma(&closes, 50),
        wma_100: moving_avg::wma(&closes, 100),
        wma_200: moving_avg::wma(&closes, 200),
        // Advanced Moving Averages
        dema_20: moving_avg::dema(&closes, 20),
        tema_20: moving_avg::tema(&closes, 20),
        hma_20: moving_avg::hma(&closes, 20),
        vwma_20: moving_avg::vwma(&closes, &volumes, 20),
        alma_9: moving_avg::alma(&closes, 9, 0.85, 6.0),
        mcginley_dynamic_20: moving_avg::mcginley_dynamic(&closes, 20),

        // === MOMENTUM OSCILLATORS ===
        rsi_14: momentum::rsi(&closes, 14),
        stochastic: {
            let (k, d) = momentum::stochastic(&highs, &lows, &closes, 14, 3);
            Some(StochasticData { k, d })
        },
        stochastic_rsi: momentum::stochastic_rsi(&closes, 14, 14).map(|k| StochasticData {
            k: Some(k),
            d: None,
        }),
        cci_20: momentum::cci(&highs, &lows, &closes, 20),
        williams_r_14: momentum::williams_r(&highs, &lows, &closes, 14),
        roc_12: momentum::roc(&closes, 12),
        momentum_10: momentum::momentum(&closes, 10),
        cmo_14: momentum::cmo(&closes, 14),
        awesome_oscillator: momentum::awesome_oscillator(&highs, &lows),
        coppock_curve: momentum::coppock_curve(&closes),

        // === TREND INDICATORS ===
        macd: Some(trend::macd(&closes, 12, 26, 9)),
        adx_14: trend::adx(&highs, &lows, &closes, 14),
        aroon: Some(trend::aroon(&highs, &lows, 25)),
        supertrend: Some(trend::supertrend(&highs, &lows, &closes, 10, 3.0)),
        ichimoku: Some(trend::ichimoku(&highs, &lows, &closes)),
        parabolic_sar: trend::parabolic_sar(&highs, &lows, &closes, 0.02, 0.2),
        bull_bear_power: Some(trend::bull_bear_power(&highs, &lows, &closes)),
        elder_ray_index: Some(trend::elder_ray(&highs, &lows, &closes)),

        // === VOLATILITY INDICATORS ===
        bollinger_bands: Some(volatility::bollinger_bands(&closes, 20, 2.0)),
        keltner_channels: Some(volatility::keltner_channels(
            &highs, &lows, &closes, 20, 10, 2.0,
        )),
        donchian_channels: Some(volatility::donchian_channels(&highs, &lows, 20)),
        atr_14: volatility::atr(&highs, &lows, &closes, 14),
        true_range: volatility::true_range(&highs, &lows, &closes),
        choppiness_index_14: volatility::choppiness_index(&highs, &lows, &closes, 14),

        // === VOLUME INDICATORS ===
        obv: volume::obv(&closes, &volumes),
        mfi_14: volume::mfi(&highs, &lows, &closes, &volumes, 14),
        cmf_20: volume::cmf(&highs, &lows, &closes, &volumes, 20),
        chaikin_oscillator: volume::chaikin_oscillator(&highs, &lows, &closes, &volumes),
        accumulation_distribution: volume::accumulation_distribution(
            &highs, &lows, &closes, &volumes,
        ),
        vwap: volume::vwap(&highs, &lows, &closes, &volumes),
        balance_of_power: volume::balance_of_power(&opens, &highs, &lows, &closes),
    }
}

/// Price data extracted from candles
type PriceData = (Vec<f64>, Vec<f64>, Vec<f64>, Vec<f64>, Vec<f64>);

/// Extract price data arrays from candles
fn prepare_data(candles: &[Candle]) -> PriceData {
    let closes: Vec<f64> = candles.iter().map(|c| c.close).collect();
    let highs: Vec<f64> = candles.iter().map(|c| c.high).collect();
    let lows: Vec<f64> = candles.iter().map(|c| c.low).collect();
    let opens: Vec<f64> = candles.iter().map(|c| c.open).collect();
    let volumes: Vec<f64> = candles.iter().map(|c| c.volume as f64).collect();
    (closes, highs, lows, opens, volumes)
}

/// Summary of all calculated technical indicators
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IndicatorsSummary {
    // === MOVING AVERAGES ===
    // Simple Moving Averages
    /// Simple Moving Average (10-period)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sma_10: Option<f64>,
    /// Simple Moving Average (20-period)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sma_20: Option<f64>,
    /// Simple Moving Average (50-period)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sma_50: Option<f64>,
    /// Simple Moving Average (100-period)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sma_100: Option<f64>,
    /// Simple Moving Average (200-period)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sma_200: Option<f64>,

    // Exponential Moving Averages
    /// Exponential Moving Average (10-period)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ema_10: Option<f64>,
    /// Exponential Moving Average (20-period)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ema_20: Option<f64>,
    /// Exponential Moving Average (50-period)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ema_50: Option<f64>,
    /// Exponential Moving Average (100-period)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ema_100: Option<f64>,
    /// Exponential Moving Average (200-period)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ema_200: Option<f64>,

    // Weighted Moving Averages
    /// Weighted Moving Average (10-period)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub wma_10: Option<f64>,
    /// Weighted Moving Average (20-period)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub wma_20: Option<f64>,
    /// Weighted Moving Average (50-period)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub wma_50: Option<f64>,
    /// Weighted Moving Average (100-period)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub wma_100: Option<f64>,
    /// Weighted Moving Average (200-period)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub wma_200: Option<f64>,

    // Advanced Moving Averages
    /// Double Exponential Moving Average (20-period)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dema_20: Option<f64>,
    /// Triple Exponential Moving Average (20-period)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tema_20: Option<f64>,
    /// Hull Moving Average (20-period)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hma_20: Option<f64>,
    /// Volume Weighted Moving Average (20-period)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub vwma_20: Option<f64>,
    /// Arnaud Legoux Moving Average (9-period)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub alma_9: Option<f64>,
    /// McGinley Dynamic (20-period)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mcginley_dynamic_20: Option<f64>,

    // === MOMENTUM OSCILLATORS ===
    /// Relative Strength Index (14-period)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rsi_14: Option<f64>,
    /// Stochastic Oscillator (14, 3, 3)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stochastic: Option<StochasticData>,
    /// Commodity Channel Index (20-period)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cci_20: Option<f64>,
    /// Williams %R (14-period)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub williams_r_14: Option<f64>,
    /// Stochastic RSI (14, 14)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stochastic_rsi: Option<StochasticData>,
    /// Rate of Change (12-period)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub roc_12: Option<f64>,
    /// Momentum (10-period)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub momentum_10: Option<f64>,
    /// Chande Momentum Oscillator (14-period)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cmo_14: Option<f64>,
    /// Awesome Oscillator (5, 34)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub awesome_oscillator: Option<f64>,
    /// Coppock Curve (10, 11, 14)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub coppock_curve: Option<f64>,

    // === TREND INDICATORS ===
    /// Moving Average Convergence Divergence (12, 26, 9)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub macd: Option<MacdData>,
    /// Average Directional Index (14-period)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub adx_14: Option<f64>,
    /// Aroon Indicator (25-period)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub aroon: Option<AroonData>,
    /// SuperTrend Indicator (10, 3.0)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub supertrend: Option<SuperTrendData>,
    /// Ichimoku Cloud (9, 26, 52, 26)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ichimoku: Option<IchimokuData>,
    /// Parabolic SAR (0.02, 0.2)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parabolic_sar: Option<f64>,
    /// Bull Bear Power (13-period EMA based)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bull_bear_power: Option<BullBearPowerData>,
    /// Elder Ray Index (13-period EMA based)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub elder_ray_index: Option<ElderRayData>,

    // === VOLATILITY INDICATORS ===
    /// Bollinger Bands (20, 2.0)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bollinger_bands: Option<BollingerBandsData>,
    /// Average True Range (14-period)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub atr_14: Option<f64>,
    /// Keltner Channels (20, 10, 2.0)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub keltner_channels: Option<KeltnerChannelsData>,
    /// Donchian Channels (20-period)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub donchian_channels: Option<DonchianChannelsData>,
    /// True Range (current period)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub true_range: Option<f64>,
    /// Choppiness Index (14-period)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub choppiness_index_14: Option<f64>,

    // === VOLUME INDICATORS ===
    /// On-Balance Volume
    #[serde(skip_serializing_if = "Option::is_none")]
    pub obv: Option<f64>,
    /// Money Flow Index (14-period)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mfi_14: Option<f64>,
    /// Chaikin Money Flow (20-period)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cmf_20: Option<f64>,
    /// Chaikin Oscillator (3, 10)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub chaikin_oscillator: Option<f64>,
    /// Accumulation/Distribution Line
    #[serde(skip_serializing_if = "Option::is_none")]
    pub accumulation_distribution: Option<f64>,
    /// Volume Weighted Average Price
    #[serde(skip_serializing_if = "Option::is_none")]
    pub vwap: Option<f64>,
    /// Balance of Power
    #[serde(skip_serializing_if = "Option::is_none")]
    pub balance_of_power: Option<f64>,
}

/// Stochastic Oscillator data
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StochasticData {
    /// %K line value
    #[serde(rename = "%K", skip_serializing_if = "Option::is_none")]
    pub k: Option<f64>,
    /// %D line value
    #[serde(rename = "%D", skip_serializing_if = "Option::is_none")]
    pub d: Option<f64>,
}

/// MACD indicator data
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MacdData {
    /// MACD line value
    #[serde(skip_serializing_if = "Option::is_none")]
    pub macd: Option<f64>,
    /// Signal line value
    #[serde(skip_serializing_if = "Option::is_none")]
    pub signal: Option<f64>,
    /// Histogram value
    #[serde(skip_serializing_if = "Option::is_none")]
    pub histogram: Option<f64>,
}

/// Aroon indicator data
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AroonData {
    /// Aroon Up value
    #[serde(skip_serializing_if = "Option::is_none")]
    pub aroon_up: Option<f64>,
    /// Aroon Down value
    #[serde(skip_serializing_if = "Option::is_none")]
    pub aroon_down: Option<f64>,
}

/// Bollinger Bands data
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BollingerBandsData {
    /// Upper band value
    #[serde(skip_serializing_if = "Option::is_none")]
    pub upper: Option<f64>,
    /// Middle band value
    #[serde(skip_serializing_if = "Option::is_none")]
    pub middle: Option<f64>,
    /// Lower band value
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lower: Option<f64>,
}

/// SuperTrend indicator data
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SuperTrendData {
    /// SuperTrend value
    #[serde(skip_serializing_if = "Option::is_none")]
    pub value: Option<f64>,
    /// Trend direction
    #[serde(skip_serializing_if = "Option::is_none")]
    pub trend: Option<String>,
}

/// Ichimoku Cloud data
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IchimokuData {
    /// Conversion line (Tenkan-sen)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub conversion_line: Option<f64>,
    /// Base line (Kijun-sen)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub base_line: Option<f64>,
    /// Leading Span A (Senkou Span A)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub leading_span_a: Option<f64>,
    /// Leading Span B (Senkou Span B)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub leading_span_b: Option<f64>,
    /// Lagging Span (Chikou Span)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lagging_span: Option<f64>,
}

/// Keltner Channels data
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct KeltnerChannelsData {
    /// Upper channel value
    #[serde(skip_serializing_if = "Option::is_none")]
    pub upper: Option<f64>,
    /// Middle channel value
    #[serde(skip_serializing_if = "Option::is_none")]
    pub middle: Option<f64>,
    /// Lower channel value
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lower: Option<f64>,
}

/// Donchian Channels data
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DonchianChannelsData {
    /// Upper channel value
    #[serde(skip_serializing_if = "Option::is_none")]
    pub upper: Option<f64>,
    /// Middle channel value
    #[serde(skip_serializing_if = "Option::is_none")]
    pub middle: Option<f64>,
    /// Lower channel value
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lower: Option<f64>,
}

/// Bull Bear Power indicator data
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BullBearPowerData {
    /// Bull power value
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bull_power: Option<f64>,
    /// Bear power value
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bear_power: Option<f64>,
}

/// Elder Ray Index data
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ElderRayData {
    /// Bull power value
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bull_power: Option<f64>,
    /// Bear power value
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bear_power: Option<f64>,
}

// === Helper Functions ===

/// Round to 2 decimal places
#[inline]
pub(crate) fn round2(value: f64) -> f64 {
    (value * 100.0).round() / 100.0
}

/// Get last value from vector, rounded to 2 decimals, handling NaN/Inf
#[inline]
#[allow(dead_code)]
pub(crate) fn last(values: &[f64]) -> Option<f64> {
    values
        .last()
        .and_then(|&v| if v.is_finite() { Some(round2(v)) } else { None })
}
