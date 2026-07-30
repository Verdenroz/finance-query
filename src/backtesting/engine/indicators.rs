use std::collections::HashMap;

use crate::backtesting::error::Result;
use crate::backtesting::strategy::Strategy;
use crate::indicators::{self, Indicator};
use crate::models::chart::Candle;

use super::BacktestEngine;

/// Returns true if the indicator needs high/low price series.
#[inline]
fn needs_high_low(indicator: &Indicator) -> bool {
    matches!(
        indicator,
        Indicator::Atr(_)
            | Indicator::Supertrend { .. }
            | Indicator::DonchianChannels(_)
            | Indicator::Cci(_)
            | Indicator::WilliamsR(_)
            | Indicator::Adx(_)
            | Indicator::Mfi(_)
            | Indicator::Cmf(_)
            | Indicator::Stochastic { .. }
            | Indicator::Aroon(_)
            | Indicator::Ichimoku { .. }
            | Indicator::ParabolicSar { .. }
            | Indicator::KeltnerChannels { .. }
            | Indicator::TrueRange
            | Indicator::ChoppinessIndex(_)
            | Indicator::Vwap
            | Indicator::ChaikinOscillator
            | Indicator::AccumulationDistribution
            | Indicator::BalanceOfPower(_)
            | Indicator::BullBearPower(_)
            | Indicator::ElderRay(_)
            | Indicator::AwesomeOscillator { .. }
    )
}

/// Returns true if the indicator needs the volume series.
#[inline]
fn needs_volumes(indicator: &Indicator) -> bool {
    matches!(
        indicator,
        Indicator::Obv
            | Indicator::Mfi(_)
            | Indicator::Cmf(_)
            | Indicator::Vwma(_)
            | Indicator::Vwap
            | Indicator::ChaikinOscillator
            | Indicator::AccumulationDistribution
    )
}

/// Compute a single indicator and return all resulting (key, values) pairs.
fn compute_one(
    closes: &[f64],
    highs: &[f64],
    lows: &[f64],
    volumes: &[f64],
    opens: &[f64],
    name: String,
    indicator: Indicator,
) -> Result<Vec<(String, Vec<Option<f64>>)>> {
    let mut out = Vec::with_capacity(5);
    match indicator {
        Indicator::Sma(period) => {
            out.push((name, indicators::sma(closes, period)));
        }
        Indicator::Ema(period) => {
            out.push((name, indicators::ema(closes, period)));
        }
        Indicator::Rsi(period) => {
            out.push((name, indicators::rsi(closes, period)?));
        }
        Indicator::Macd { fast, slow, signal } => {
            let m = indicators::macd(closes, fast, slow, signal)?;
            out.push((format!("macd_line_{fast}_{slow}_{signal}"), m.macd_line));
            out.push((format!("macd_signal_{fast}_{slow}_{signal}"), m.signal_line));
            out.push((
                format!("macd_histogram_{fast}_{slow}_{signal}"),
                m.histogram,
            ));
        }
        Indicator::Bollinger { period, std_dev } => {
            let bb = indicators::bollinger_bands(closes, period, std_dev)?;
            out.push((format!("bollinger_upper_{period}_{std_dev}"), bb.upper));
            out.push((format!("bollinger_middle_{period}_{std_dev}"), bb.middle));
            out.push((format!("bollinger_lower_{period}_{std_dev}"), bb.lower));
        }
        Indicator::Atr(period) => {
            out.push((name, indicators::atr(highs, lows, closes, period)?));
        }
        Indicator::Supertrend { period, multiplier } => {
            let st = indicators::supertrend(highs, lows, closes, period, multiplier)?;
            out.push((format!("supertrend_value_{period}_{multiplier}"), st.value));
            let uptrend: Vec<Option<f64>> = st
                .is_uptrend
                .into_iter()
                .map(|v| v.map(|b| if b { 1.0 } else { 0.0 }))
                .collect();
            out.push((format!("supertrend_uptrend_{period}_{multiplier}"), uptrend));
        }
        Indicator::DonchianChannels(period) => {
            let dc = indicators::donchian_channels(highs, lows, period)?;
            out.push((format!("donchian_upper_{period}"), dc.upper));
            out.push((format!("donchian_middle_{period}"), dc.middle));
            out.push((format!("donchian_lower_{period}"), dc.lower));
        }
        Indicator::Wma(period) => {
            out.push((name, indicators::wma(closes, period)?));
        }
        Indicator::Dema(period) => {
            out.push((name, indicators::dema(closes, period)?));
        }
        Indicator::Tema(period) => {
            out.push((name, indicators::tema(closes, period)?));
        }
        Indicator::Hma(period) => {
            out.push((name, indicators::hma(closes, period)?));
        }
        Indicator::Obv => {
            out.push((name, indicators::obv(closes, volumes)?));
        }
        Indicator::Momentum(period) => {
            out.push((name, indicators::momentum(closes, period)?));
        }
        Indicator::Roc(period) => {
            out.push((name, indicators::roc(closes, period)?));
        }
        Indicator::Cci(period) => {
            out.push((name, indicators::cci(highs, lows, closes, period)?));
        }
        Indicator::WilliamsR(period) => {
            out.push((name, indicators::williams_r(highs, lows, closes, period)?));
        }
        Indicator::Adx(period) => {
            out.push((name, indicators::adx(highs, lows, closes, period)?));
        }
        Indicator::Mfi(period) => {
            out.push((name, indicators::mfi(highs, lows, closes, volumes, period)?));
        }
        Indicator::Cmf(period) => {
            out.push((name, indicators::cmf(highs, lows, closes, volumes, period)?));
        }
        Indicator::Cmo(period) => {
            out.push((name, indicators::cmo(closes, period)?));
        }
        Indicator::Vwma(period) => {
            out.push((name, indicators::vwma(closes, volumes, period)?));
        }
        Indicator::Alma {
            period,
            offset,
            sigma,
        } => {
            out.push((name, indicators::alma(closes, period, offset, sigma)?));
        }
        Indicator::McginleyDynamic(period) => {
            out.push((name, indicators::mcginley_dynamic(closes, period)?));
        }
        Indicator::Stochastic {
            k_period,
            k_slow,
            d_period,
        } => {
            let s = indicators::stochastic(highs, lows, closes, k_period, k_slow, d_period)?;
            out.push((format!("stochastic_k_{k_period}_{k_slow}_{d_period}"), s.k));
            out.push((format!("stochastic_d_{k_period}_{k_slow}_{d_period}"), s.d));
        }
        Indicator::StochasticRsi {
            rsi_period,
            stoch_period,
            k_period,
            d_period,
        } => {
            let s =
                indicators::stochastic_rsi(closes, rsi_period, stoch_period, k_period, d_period)?;
            out.push((
                format!("stoch_rsi_k_{rsi_period}_{stoch_period}_{k_period}_{d_period}"),
                s.k,
            ));
            out.push((
                format!("stoch_rsi_d_{rsi_period}_{stoch_period}_{k_period}_{d_period}"),
                s.d,
            ));
        }
        Indicator::AwesomeOscillator { fast, slow } => {
            out.push((
                name,
                indicators::awesome_oscillator(highs, lows, fast, slow)?,
            ));
        }
        Indicator::CoppockCurve {
            wma_period,
            long_roc,
            short_roc,
        } => {
            out.push((
                name,
                indicators::coppock_curve(closes, long_roc, short_roc, wma_period)?,
            ));
        }
        Indicator::Aroon(period) => {
            let a = indicators::aroon(highs, lows, period)?;
            out.push((format!("aroon_up_{period}"), a.aroon_up));
            out.push((format!("aroon_down_{period}"), a.aroon_down));
        }
        Indicator::Ichimoku {
            conversion,
            base,
            lagging,
            displacement,
        } => {
            let ich =
                indicators::ichimoku(highs, lows, closes, conversion, base, lagging, displacement)?;
            out.push((
                format!("ichimoku_conversion_{conversion}_{base}_{lagging}_{displacement}"),
                ich.conversion_line,
            ));
            out.push((
                format!("ichimoku_base_{conversion}_{base}_{lagging}_{displacement}"),
                ich.base_line,
            ));
            out.push((
                format!("ichimoku_leading_a_{conversion}_{base}_{lagging}_{displacement}"),
                ich.leading_span_a,
            ));
            out.push((
                format!("ichimoku_leading_b_{conversion}_{base}_{lagging}_{displacement}"),
                ich.leading_span_b,
            ));
            out.push((
                format!("ichimoku_lagging_{conversion}_{base}_{lagging}_{displacement}"),
                ich.lagging_span,
            ));
        }
        Indicator::ParabolicSar { step, max } => {
            out.push((
                name,
                indicators::parabolic_sar(highs, lows, closes, step, max)?,
            ));
        }
        Indicator::KeltnerChannels {
            period,
            multiplier,
            atr_period,
        } => {
            let kc =
                indicators::keltner_channels(highs, lows, closes, period, atr_period, multiplier)?;
            out.push((
                format!("keltner_upper_{period}_{multiplier}_{atr_period}"),
                kc.upper,
            ));
            out.push((
                format!("keltner_middle_{period}_{multiplier}_{atr_period}"),
                kc.middle,
            ));
            out.push((
                format!("keltner_lower_{period}_{multiplier}_{atr_period}"),
                kc.lower,
            ));
        }
        Indicator::TrueRange => {
            out.push((name, indicators::true_range(highs, lows, closes)?));
        }
        Indicator::ChoppinessIndex(period) => {
            out.push((
                name,
                indicators::choppiness_index(highs, lows, closes, period)?,
            ));
        }
        Indicator::Vwap => {
            out.push((name, indicators::vwap(highs, lows, closes, volumes)?));
        }
        Indicator::ChaikinOscillator => {
            out.push((
                name,
                indicators::chaikin_oscillator(highs, lows, closes, volumes)?,
            ));
        }
        Indicator::AccumulationDistribution => {
            out.push((
                name,
                indicators::accumulation_distribution(highs, lows, closes, volumes)?,
            ));
        }
        Indicator::BalanceOfPower(period) => {
            out.push((
                name,
                indicators::balance_of_power(opens, highs, lows, closes, period)?,
            ));
        }
        Indicator::BullBearPower(period) => {
            let bbp = indicators::bull_bear_power(highs, lows, closes, period)?;
            out.push((format!("bull_power_{period}"), bbp.bull_power));
            out.push((format!("bear_power_{period}"), bbp.bear_power));
        }
        Indicator::ElderRay(period) => {
            let er = indicators::elder_ray(highs, lows, closes, period)?;
            out.push((format!("elder_bull_{period}"), er.bull_power));
            out.push((format!("elder_bear_{period}"), er.bear_power));
        }
    }
    Ok(out)
}

/// Pre-compute a set of indicators on the given candles.
///
/// Accepts a list of `(key, Indicator)` pairs as returned by
/// [`Condition::required_indicators`] and returns a map from key to
/// the computed `Vec<Option<f64>>`. Multi-output indicators (MACD,
/// Bollinger, Supertrend, etc.) insert additional derived keys,
/// ignoring the supplied key for those variants.
///
/// When 4 or more indicators are requested, computation runs in parallel
/// using rayon's thread pool.
pub(crate) fn compute_for_candles(
    candles: &[Candle],
    required: Vec<(String, Indicator)>,
) -> Result<HashMap<String, Vec<Option<f64>>>> {
    if required.is_empty() {
        return Ok(HashMap::new());
    }

    let use_hl = required.iter().any(|(_, i)| needs_high_low(i));
    let use_vol = required.iter().any(|(_, i)| needs_volumes(i));
    let use_open = required
        .iter()
        .any(|(_, i)| matches!(i, Indicator::BalanceOfPower(_)));

    // Extract price series upfront (single pass each, cache-friendly).
    let closes: Vec<f64> = candles.iter().map(|c| c.close).collect();
    let (highs, lows): (Vec<f64>, Vec<f64>) = if use_hl {
        candles.iter().map(|c| (c.high, c.low)).unzip()
    } else {
        (vec![], vec![])
    };
    let volumes: Vec<f64> = if use_vol {
        candles.iter().map(|c| c.volume as f64).collect()
    } else {
        vec![]
    };
    let opens: Vec<f64> = if use_open {
        candles.iter().map(|c| c.open).collect()
    } else {
        vec![]
    };

    type IndPairs = Vec<(String, Vec<Option<f64>>)>;

    // Parallelise only when both the indicator count and candle count are large
    // enough that rayon task-dispatch overhead is outweighed by the savings.
    // Empirically: ≥4 indicators AND ≥1 000 candles.
    let groups: Result<Vec<IndPairs>> = if required.len() >= 4 && candles.len() >= 1_000 {
        use rayon::prelude::*;
        required
            .into_par_iter()
            .map(|(name, ind)| compute_one(&closes, &highs, &lows, &volumes, &opens, name, ind))
            .collect()
    } else {
        required
            .into_iter()
            .map(|(name, ind)| compute_one(&closes, &highs, &lows, &volumes, &opens, name, ind))
            .collect()
    };

    let groups = groups?;
    let capacity: usize = groups.iter().map(|v| v.len()).sum();
    let mut result = HashMap::with_capacity(capacity);
    for group in groups {
        for (k, v) in group {
            result.insert(k, v);
        }
    }
    Ok(result)
}

impl BacktestEngine {
    /// Pre-compute all indicators required by the strategy
    pub(crate) fn compute_indicators<S: Strategy>(
        &self,
        candles: &[Candle],
        strategy: &S,
    ) -> Result<HashMap<String, Vec<Option<f64>>>> {
        compute_for_candles(candles, strategy.required_indicators())
    }

    /// Pre-compute stretched HTF indicator arrays for all `HtfCondition`s in the strategy.
    ///
    /// For each unique `(interval, utc_offset_secs)` pair:
    /// - Resample the full candle history to the HTF interval
    /// - Compute required indicators on the resampled candles
    /// - Build a mapping from base timeframe to HTF indices
    /// - Stretch each HTF indicator value to base timeframe length
    /// - Store the stretched array in the result map under the `htf_key`
    pub(super) fn compute_htf_indicators<S: Strategy>(
        &self,
        candles: &[Candle],
        strategy: &S,
    ) -> Result<HashMap<String, Vec<Option<f64>>>> {
        use std::collections::HashSet;

        use crate::backtesting::condition::HtfIndicatorSpec;
        use crate::backtesting::resample::{base_to_htf_index, resample};
        use crate::constants::Interval;

        let specs = strategy.htf_requirements();
        if specs.is_empty() {
            return Ok(HashMap::new());
        }

        let mut result = HashMap::new();

        // Group specs by (interval, utc_offset_secs) — one resample per unique pair.
        let mut by_interval: HashMap<(Interval, i64), Vec<HtfIndicatorSpec>> = HashMap::new();
        for spec in specs {
            by_interval
                .entry((spec.interval, spec.utc_offset_secs))
                .or_default()
                .push(spec);
        }

        for ((interval, utc_offset_secs), specs) in by_interval {
            let htf_candles = resample(candles, interval, utc_offset_secs);
            if htf_candles.is_empty() {
                continue;
            }

            // De-duplicate indicators by base_key to avoid recomputing MACD/Bollinger
            // etc. when multiple output keys (line, signal, histogram) are requested.
            let mut required: Vec<(String, crate::indicators::Indicator)> = Vec::new();
            let mut seen_base_keys: HashSet<&str> = HashSet::new();
            for spec in &specs {
                if seen_base_keys.insert(&spec.base_key) {
                    required.push((spec.base_key.clone(), spec.indicator));
                }
            }

            let htf_values = compute_for_candles(&htf_candles, required)?;
            let mapping = base_to_htf_index(candles, &htf_candles);

            for spec in &specs {
                if let Some(htf_vec) = htf_values.get(&spec.base_key) {
                    let stretched: Vec<Option<f64>> = mapping
                        .iter()
                        .map(|htf_idx| htf_idx.and_then(|i| htf_vec.get(i).copied().flatten()))
                        .collect();
                    result.insert(spec.htf_key.clone(), stretched);
                }
            }
        }

        Ok(result)
    }
}
