//! Indicators summary module.
//!
//! Provides the `IndicatorsSummary` type which calculates and returns the latest
//! values for all 52+ technical indicators at once.
//!
//! This module reuses the main indicator implementations and extracts the last value,
//! ensuring consistency and eliminating code duplication.

use crate::Candle;
use crate::indicators::{
    accumulation_distribution, adx, alma, aroon, atr, awesome_oscillator, balance_of_power,
    bollinger_bands, bull_bear_power, cci, chaikin_oscillator, choppiness_index, cmf, cmo,
    coppock_curve, dema, donchian_channels, elder_ray, ema, hma, ichimoku, keltner_channels, macd,
    mcginley_dynamic, mfi, momentum, obv, parabolic_sar, roc, rsi, sma, stochastic, stochastic_rsi,
    supertrend, tema, true_range, vwap, vwma, williams_r, wma,
};

/// Extract the last non-None value from a time series.
///
/// Iterates from the end of the series to find the most recent valid value.
#[inline]
fn last_value(series: &[Option<f64>]) -> Option<f64> {
    series.iter().rev().find_map(|&v| v)
}

/// Helper to extract last value from Result-returning indicators
#[inline]
fn last_from_result(result: crate::indicators::Result<Vec<Option<f64>>>) -> Option<f64> {
    result.ok().and_then(|v| last_value(&v))
}

/// Calculate all technical indicators from candle data.
///
/// Returns the latest values for all implemented indicators.
/// Reuses the main indicator implementations for consistency.
pub(crate) fn calculate_indicators(candles: &[Candle]) -> IndicatorsSummary {
    if candles.is_empty() {
        return IndicatorsSummary::default();
    }

    // Extract price data from candles
    let closes: Vec<f64> = candles.iter().map(|c| c.close).collect();
    let highs: Vec<f64> = candles.iter().map(|c| c.high).collect();
    let lows: Vec<f64> = candles.iter().map(|c| c.low).collect();
    let opens: Vec<f64> = candles.iter().map(|c| c.open).collect();
    let volumes: Vec<f64> = candles.iter().map(|c| c.volume as f64).collect();

    IndicatorsSummary {
        // === MOVING AVERAGES ===
        // Simple Moving Averages
        sma_10: last_value(&sma(&closes, 10)),
        sma_20: last_value(&sma(&closes, 20)),
        sma_50: last_value(&sma(&closes, 50)),
        sma_100: last_value(&sma(&closes, 100)),
        sma_200: last_value(&sma(&closes, 200)),

        // Exponential Moving Averages
        ema_10: last_value(&ema(&closes, 10)),
        ema_20: last_value(&ema(&closes, 20)),
        ema_50: last_value(&ema(&closes, 50)),
        ema_100: last_value(&ema(&closes, 100)),
        ema_200: last_value(&ema(&closes, 200)),

        // Weighted Moving Averages (Result types)
        wma_10: wma(&closes, 10).ok().and_then(|v| last_value(&v)),
        wma_20: wma(&closes, 20).ok().and_then(|v| last_value(&v)),
        wma_50: wma(&closes, 50).ok().and_then(|v| last_value(&v)),
        wma_100: wma(&closes, 100).ok().and_then(|v| last_value(&v)),
        wma_200: wma(&closes, 200).ok().and_then(|v| last_value(&v)),

        // Advanced Moving Averages (Result types)
        dema_20: dema(&closes, 20).ok().and_then(|v| last_value(&v)),
        tema_20: tema(&closes, 20).ok().and_then(|v| last_value(&v)),
        hma_20: hma(&closes, 20).ok().and_then(|v| last_value(&v)),
        vwma_20: vwma(&closes, &volumes, 20)
            .ok()
            .and_then(|v| last_value(&v)),
        alma_9: alma(&closes, 9, 0.85, 6.0)
            .ok()
            .and_then(|v| last_value(&v)),
        mcginley_dynamic_20: mcginley_dynamic(&closes, 20)
            .ok()
            .and_then(|v| last_value(&v)),

        // === MOMENTUM OSCILLATORS ===
        rsi_14: last_from_result(rsi(&closes, 14)),
        stochastic: {
            stochastic(&highs, &lows, &closes, 14, 1, 3)
                .ok()
                .map(|result| StochasticData {
                    k: last_value(&result.k),
                    d: last_value(&result.d),
                })
        },
        stochastic_rsi: {
            stochastic_rsi(&closes, 14, 14, 3, 3)
                .ok()
                .map(|result| StochasticData {
                    k: last_value(&result.k),
                    d: last_value(&result.d),
                })
        },
        cci_20: last_from_result(cci(&highs, &lows, &closes, 20)),
        williams_r_14: last_from_result(williams_r(&highs, &lows, &closes, 14)),
        roc_12: last_from_result(roc(&closes, 12)),
        momentum_10: last_from_result(momentum(&closes, 10)),
        cmo_14: last_from_result(cmo(&closes, 14)),
        awesome_oscillator: last_from_result(awesome_oscillator(&highs, &lows, 5, 34)),
        coppock_curve: last_from_result(coppock_curve(&closes, 14, 11, 10)),

        // === TREND INDICATORS ===
        macd: {
            macd(&closes, 12, 26, 9).ok().map(|result| MacdData {
                macd: last_value(&result.macd_line),
                signal: last_value(&result.signal_line),
                histogram: last_value(&result.histogram),
            })
        },
        adx_14: last_from_result(adx(&highs, &lows, &closes, 14)),
        aroon: {
            aroon(&highs, &lows, 25).ok().map(|result| AroonData {
                aroon_up: last_value(&result.aroon_up),
                aroon_down: last_value(&result.aroon_down),
            })
        },
        supertrend: {
            supertrend(&highs, &lows, &closes, 10, 3.0)
                .ok()
                .map(|result| SuperTrendData {
                    value: last_value(&result.value),
                    trend: result.is_uptrend.last().and_then(|&v| v).map(|v| {
                        if v {
                            "up".to_string()
                        } else {
                            "down".to_string()
                        }
                    }),
                })
        },
        ichimoku: {
            ichimoku(&highs, &lows, &closes, 9, 26, 26, 26)
                .ok()
                .map(|result| IchimokuData {
                    conversion_line: last_value(&result.conversion_line),
                    base_line: last_value(&result.base_line),
                    leading_span_a: last_value(&result.leading_span_a),
                    leading_span_b: last_value(&result.leading_span_b),
                    lagging_span: last_value(&result.lagging_span),
                })
        },
        parabolic_sar: last_from_result(parabolic_sar(&highs, &lows, &closes, 0.02, 0.2)),
        bull_bear_power: {
            bull_bear_power(&highs, &lows, &closes, 13)
                .ok()
                .map(|result| BullBearPowerData {
                    bull_power: last_value(&result.bull_power),
                    bear_power: last_value(&result.bear_power),
                })
        },
        elder_ray_index: {
            elder_ray(&highs, &lows, &closes, 13)
                .ok()
                .map(|result| ElderRayData {
                    bull_power: last_value(&result.bull_power),
                    bear_power: last_value(&result.bear_power),
                })
        },

        // === VOLATILITY INDICATORS ===
        bollinger_bands: {
            bollinger_bands(&closes, 20, 2.0)
                .ok()
                .map(|result| BollingerBandsData {
                    upper: last_value(&result.upper),
                    middle: last_value(&result.middle),
                    lower: last_value(&result.lower),
                })
        },
        keltner_channels: {
            keltner_channels(&highs, &lows, &closes, 20, 10, 2.0)
                .ok()
                .map(|result| KeltnerChannelsData {
                    upper: last_value(&result.upper),
                    middle: last_value(&result.middle),
                    lower: last_value(&result.lower),
                })
        },
        donchian_channels: {
            donchian_channels(&highs, &lows, 20)
                .ok()
                .map(|result| DonchianChannelsData {
                    upper: last_value(&result.upper),
                    middle: last_value(&result.middle),
                    lower: last_value(&result.lower),
                })
        },
        atr_14: last_from_result(atr(&highs, &lows, &closes, 14)),
        true_range: last_from_result(true_range(&highs, &lows, &closes)),
        choppiness_index_14: last_from_result(choppiness_index(&highs, &lows, &closes, 14)),

        // === VOLUME INDICATORS ===
        obv: last_from_result(obv(&closes, &volumes)),
        mfi_14: last_from_result(mfi(&highs, &lows, &closes, &volumes, 14)),
        cmf_20: last_from_result(cmf(&highs, &lows, &closes, &volumes, 20)),
        chaikin_oscillator: last_from_result(chaikin_oscillator(&highs, &lows, &closes, &volumes)),
        accumulation_distribution: last_from_result(accumulation_distribution(
            &highs, &lows, &closes, &volumes,
        )),
        vwap: last_from_result(vwap(&highs, &lows, &closes, &volumes)),
        balance_of_power: last_from_result(balance_of_power(&opens, &highs, &lows, &closes, None)),
    }
}

/// Summary of all calculated technical indicators
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
#[cfg_attr(feature = "dataframe", derive(crate::ToDataFrame))]
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
