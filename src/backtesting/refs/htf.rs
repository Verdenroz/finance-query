//! Higher-timeframe (HTF) condition wrapper.
//!
//! [`htf()`] wraps any [`Condition`] to evaluate it on a resampled
//! higher-timeframe candle series, enabling multi-timeframe confirmation
//! without look-ahead bias. Use [`htf_region()`] when the underlying instrument
//! trades on a non-UTC exchange (e.g. Tokyo, Hong Kong) to ensure weekly and
//! monthly bucket boundaries align with the local calendar.
//!
//! # How it works — fast path (engine pre-computation)
//!
//! When the strategy is built with [`StrategyBuilder`], the engine pre-computes
//! all HTF indicator arrays once before the main simulation loop:
//!
//! 1. Collects [`HtfIndicatorSpec`]s from `HtfCondition::htf_requirements()`.
//! 2. Resamples the full candle history to each unique HTF interval **once**.
//! 3. Computes the inner indicators on the resampled data.
//! 4. Stretches results back to base-timeframe length via `base_to_htf_index`.
//! 5. Stores stretched arrays in `StrategyContext::indicators` under `htf_key`.
//!
//! On each bar, `evaluate()` does only O(k) work (k = # inner indicators):
//! it reads the pre-computed values, builds a tiny 2-element indicator map, and
//! evaluates the inner condition. HTF crossovers work correctly because the map
//! stores `[prev, current]`.
//!
//! # Fallback (dynamic resampling)
//!
//! If the pre-computed arrays are not found in `ctx.indicators` (e.g. a raw
//! [`Strategy`] implementation that doesn't forward `htf_requirements()`), the
//! condition falls back to dynamic resampling — O(n) per bar, O(n²) total.
//! A `tracing::warn!` is emitted once per evaluation so the caller can diagnose
//! the performance issue.
//!
//! # Example
//!
//! ```ignore
//! use finance_query::backtesting::{StrategyBuilder, BacktestConfig, BacktestEngine};
//! use finance_query::backtesting::refs::*;
//! use finance_query::Interval;
//!
//! // Enter only when daily EMA10 crosses EMA30 AND weekly price > SMA20
//! let strategy = StrategyBuilder::new("MTF Confirmation")
//!     .entry(
//!         ema(10).crosses_above_ref(ema(30))
//!             .and(htf(Interval::OneWeek, price().above_ref(sma(20))))
//!     )
//!     .exit(ema(10).crosses_below_ref(ema(30)))
//!     .build();
//! ```
//!
//! For a Tokyo Stock Exchange strategy:
//!
//! ```ignore
//! use finance_query::backtesting::refs::*;
//! use finance_query::{Interval, Region};
//!
//! let weekly_trend = htf_region(Interval::OneWeek, Region::Japan, price().above_ref(sma(20)));
//! ```
//!
//! [`StrategyBuilder`]: crate::backtesting::strategy::StrategyBuilder
//! [`Strategy`]: crate::backtesting::strategy::Strategy

use std::collections::HashMap;

use crate::backtesting::condition::{Condition, HtfIndicatorSpec};
use crate::backtesting::engine::compute_for_candles;
use crate::backtesting::resample::resample;
use crate::backtesting::strategy::StrategyContext;
use crate::constants::{Interval, Region};
use crate::indicators::Indicator;

/// A condition that evaluates its inner condition on a resampled HTF candle series.
///
/// Created by [`htf()`] (UTC-aligned) or [`htf_region()`] (exchange-local calendar).
#[derive(Clone)]
pub struct HtfCondition<C: Condition> {
    interval: Interval,
    inner: C,
    /// UTC offset of the exchange. Shifts bucket boundaries so weekly/monthly
    /// periods align with the exchange's local calendar rather than UTC midnight.
    utc_offset_secs: i64,
    /// Derived once at construction from `inner.required_indicators()`. Purely a
    /// function of `self`, so `evaluate` never has to re-walk the condition tree
    /// or re-format lookup keys on a per-bar basis.
    specs: Vec<HtfIndicatorSpec>,
}

impl<C: Condition> Condition for HtfCondition<C> {
    fn evaluate(&self, ctx: &StrategyContext) -> bool {
        // ── Fast path: use pre-computed stretched arrays from the engine ──────
        // The engine stores stretched HTF values in ctx.indicators under keys of
        // the form "htf_{interval}_{base_key}" (e.g. "htf_1wk_sma_20"), which is
        // exactly what `self.specs` holds.
        //
        // We build a tiny 2-element indicators map [prev, curr] so that the inner
        // condition's crossover helpers (indicator_prev / crossed_above etc.) work
        // correctly, then evaluate with index=1.
        if !self.specs.is_empty() {
            let mut mini_indicators: HashMap<String, Vec<Option<f64>>> =
                HashMap::with_capacity(self.specs.len());
            let mut all_found = true;

            for spec in &self.specs {
                if let Some(stretched) = ctx.indicators.get(&spec.htf_key) {
                    let curr = stretched.get(ctx.index).copied().flatten();
                    let prev = ctx
                        .index
                        .checked_sub(1)
                        .and_then(|pi| stretched.get(pi).copied().flatten());
                    // index=1 → curr; index=0 → prev (via indicator_prev)
                    mini_indicators.insert(spec.base_key.clone(), vec![prev, curr]);
                } else {
                    all_found = false;
                    break;
                }
            }

            if all_found {
                // Use a 2-candle slice [prev_bar, current_bar] so that
                // current_candle() (index=1) returns the correct base bar.
                let start = ctx.index.saturating_sub(1);
                let htf_ctx = StrategyContext {
                    candles: &ctx.candles[start..=ctx.index],
                    index: ctx.index - start, // 1 unless at bar 0
                    position: ctx.position,
                    equity: ctx.equity,
                    indicators: &mini_indicators,
                };
                return self.inner.evaluate(&htf_ctx);
            }
        } else {
            // Pure price condition — no HTF indicators needed.
            // Evaluate directly so price() reads from the current base bar.
            return self.inner.evaluate(ctx);
        }

        // ── Fallback: dynamic resampling (O(n) per bar → O(n²) total) ────────
        // Reached only when the strategy does not implement htf_requirements()
        // (e.g. a raw Strategy impl). StrategyBuilder-based strategies never hit
        // this path because the engine pre-computes the stretched arrays.
        tracing::warn!(
            interval = %self.interval,
            "HtfCondition falling back to O(n²) dynamic resampling — \
             implement Strategy::htf_requirements() or use StrategyBuilder \
             to enable O(1) pre-computed HTF lookups"
        );
        self.evaluate_dynamic(ctx)
    }

    fn required_indicators(&self) -> Vec<(String, Indicator)> {
        // HtfCondition resolves its own indicators on resampled data.
        // The main engine must NOT pre-compute these on the base-TF candles.
        vec![]
    }

    fn htf_requirements(&self) -> Vec<HtfIndicatorSpec> {
        self.specs.clone()
    }

    fn description(&self) -> String {
        format!("htf({}, {})", self.interval, self.inner.description())
    }
}

impl<C: Condition> HtfCondition<C> {
    fn new(interval: Interval, inner: C, utc_offset_secs: i64) -> Self {
        let interval_str = interval.as_str();
        let specs = inner
            .required_indicators()
            .into_iter()
            .map(|(base_key, indicator)| HtfIndicatorSpec {
                interval,
                htf_key: format!("htf_{}_{}", interval_str, base_key),
                base_key,
                indicator,
                utc_offset_secs,
            })
            .collect();
        Self {
            interval,
            inner,
            utc_offset_secs,
            specs,
        }
    }

    /// Dynamic resampling fallback used when pre-computed data is unavailable.
    ///
    /// This is O(n) per bar (O(n²) overall). It finds the most recently
    /// *completed* HTF bar (timestamp < current bar) to avoid look-ahead bias.
    fn evaluate_dynamic(&self, ctx: &StrategyContext) -> bool {
        let htf_candles = resample(ctx.candles, self.interval, self.utc_offset_secs);

        // Find the most recently completed HTF bar — timestamp strictly less than
        // the current base bar's timestamp. In the dynamic path we use `<` (not
        // `<=`) to conservatively exclude the in-progress period: we only have
        // candles up to ctx.index, so the last HTF bar may be partial.
        let current_ts = ctx.current_candle().timestamp;
        let htf_idx = match htf_candles.iter().rposition(|c| c.timestamp < current_ts) {
            Some(i) => i,
            None => return false, // No completed HTF bar yet
        };

        let htf_indicators = if self.specs.is_empty() {
            HashMap::new()
        } else {
            let required = self
                .specs
                .iter()
                .map(|s| (s.base_key.clone(), s.indicator))
                .collect();
            match compute_for_candles(&htf_candles, required) {
                Ok(map) => map,
                Err(e) => {
                    tracing::warn!("HTF indicator computation failed: {}", e);
                    return false;
                }
            }
        };

        let htf_ctx = StrategyContext {
            candles: &htf_candles,
            index: htf_idx,
            position: ctx.position,
            equity: ctx.equity,
            indicators: &htf_indicators,
        };

        self.inner.evaluate(&htf_ctx)
    }
}

/// Wrap a condition to be evaluated on a higher-timeframe candle series.
///
/// Bucket boundaries are UTC-aligned (offset = 0). For non-UTC exchanges use
/// [`htf_region()`] instead.
///
/// # Arguments
///
/// * `interval` – Target higher timeframe (e.g. `Interval::OneWeek`)
/// * `cond` – Any condition to evaluate on the HTF candles
///
/// # Example
///
/// ```ignore
/// use finance_query::backtesting::refs::*;
/// use finance_query::Interval;
///
/// // Entry only when weekly price is above its 20-bar SMA
/// let weekly_uptrend = htf(Interval::OneWeek, price().above_ref(sma(20)));
/// let entry = ema(10).crosses_above_ref(ema(30)).and(weekly_uptrend);
/// ```
pub fn htf<C: Condition>(interval: Interval, cond: C) -> HtfCondition<C> {
    HtfCondition::new(interval, cond, 0)
}

/// Wrap a condition to be evaluated on a higher-timeframe candle series,
/// with bucket boundaries aligned to the exchange's local calendar.
///
/// Weekly and monthly boundaries are shifted by `region.utc_offset_secs()` so
/// that, for example, a Tokyo-listed stock's "Monday" starts at the correct
/// local midnight rather than UTC midnight.
///
/// # Arguments
///
/// * `interval` – Target higher timeframe (e.g. `Interval::OneWeek`)
/// * `region`   – Exchange region used to derive the UTC offset
/// * `cond`     – Any condition to evaluate on the HTF candles
///
/// # Example
///
/// ```ignore
/// use finance_query::backtesting::refs::*;
/// use finance_query::{Interval, Region};
///
/// let weekly_trend = htf_region(Interval::OneWeek, Region::Japan, price().above_ref(sma(20)));
/// ```
pub fn htf_region<C: Condition>(interval: Interval, region: Region, cond: C) -> HtfCondition<C> {
    HtfCondition::new(interval, cond, region.utc_offset_secs())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::backtesting::config::BacktestConfig;
    use crate::backtesting::engine::BacktestEngine;
    use crate::backtesting::refs::{IndicatorRefExt, price, sma};
    use crate::backtesting::strategy::StrategyBuilder;
    use crate::models::chart::Candle;

    const DAY: i64 = 86_400;

    /// Daily bars starting 1970-01-05 (a Monday) so weekly buckets are aligned.
    fn daily_candles(prices: &[f64]) -> Vec<Candle> {
        prices
            .iter()
            .enumerate()
            .map(|(i, &p)| Candle {
                timestamp: 4 * DAY + i as i64 * DAY,
                open: p,
                high: p * 1.01,
                low: p * 0.99,
                close: p,
                volume: 1_000,
                adj_close: Some(p),
                provider_id: None,
            })
            .collect()
    }

    /// Rises, falls, then rises again — enough regime changes to produce
    /// multiple weekly HTF crossings of the SMA.
    fn zigzag_prices(n: usize) -> Vec<f64> {
        (0..n)
            .map(|i| {
                let t = i as f64;
                100.0 + 20.0 * (t / 14.0).sin() + t * 0.05
            })
            .collect()
    }

    fn run_htf_backtest() -> Vec<(i64, i64, f64, f64)> {
        let candles = daily_candles(&zigzag_prices(180));
        let config = BacktestConfig::builder()
            .commission_pct(0.0)
            .slippage_pct(0.0)
            .build()
            .unwrap();

        let strategy = StrategyBuilder::new("HTF Characterization")
            .entry(htf(Interval::OneWeek, price().above_ref(sma(3))))
            .exit(htf(Interval::OneWeek, price().below_ref(sma(3))))
            .build();

        let result = BacktestEngine::new(config)
            .run("TEST", &candles, strategy)
            .unwrap();

        result
            .trades
            .iter()
            .map(|t| {
                (
                    t.entry_timestamp,
                    t.exit_timestamp,
                    (t.entry_price * 1e6).round() / 1e6,
                    (t.exit_price * 1e6).round() / 1e6,
                )
            })
            .collect()
    }

    /// Golden trade sequence. Precomputing the HTF lookup keys must not change
    /// a single entry/exit.
    #[test]
    fn test_htf_backtest_trade_sequence_is_stable() {
        let expected = vec![
            (2_160_000, 2_937_600, 120.9999, 118.315742),
            (6_739_200, 10_627_200, 86.897962, 121.919742),
            (14_256_000, 15_811_200, 90.540957, 113.301781),
        ];
        assert_eq!(run_htf_backtest(), expected);
    }

    #[test]
    fn test_precomputed_keys_match_htf_requirements() {
        let cond = htf(Interval::OneWeek, price().above_ref(sma(20)));
        let specs = cond.htf_requirements();
        assert_eq!(specs.len(), 1);
        assert_eq!(specs[0].base_key, "sma_20");
        assert_eq!(specs[0].htf_key, "htf_1wk_sma_20");
    }

    #[test]
    fn test_pure_price_condition_has_no_htf_requirements() {
        let cond = htf(Interval::OneWeek, price().above(100.0));
        assert!(cond.htf_requirements().is_empty());
    }
}
