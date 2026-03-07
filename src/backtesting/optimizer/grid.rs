//! Exhaustive grid-search parameter optimisation.
//!
//! Use [`GridSearch`] to sweep over all combinations of named parameter ranges
//! and rank them by a chosen metric. All combinations run in parallel via
//! `rayon`, and results are returned sorted best-first.
//!
//! # Example
//!
//! ```ignore
//! use finance_query::backtesting::{
//!     BacktestConfig, SmaCrossover,
//!     optimizer::{GridSearch, OptimizeMetric, ParamRange, ParamValue},
//! };
//!
//! # fn example(candles: &[finance_query::models::chart::Candle]) {
//! let report = GridSearch::new()
//!     .param("fast", ParamRange::int_range(5, 50, 5))
//!     .param("slow", ParamRange::int_range(20, 200, 10))
//!     .optimize_for(OptimizeMetric::SharpeRatio)
//!     .run("AAPL", candles, &BacktestConfig::default(), |params| {
//!         SmaCrossover::new(
//!             params["fast"].as_int() as usize,
//!             params["slow"].as_int() as usize,
//!         )
//!     })
//!     .unwrap();
//!
//! println!("Best params: {:?}", report.best.params);
//! println!("Best Sharpe: {:.2}", report.best.result.metrics.sharpe_ratio);
//! # }
//! ```

use std::collections::HashMap;
use std::sync::atomic::{AtomicUsize, Ordering};

use rayon::prelude::*;

use crate::models::chart::Candle;

use super::super::config::BacktestConfig;
use super::super::engine::BacktestEngine;
use super::super::error::{BacktestError, Result};
use super::super::strategy::Strategy;
use super::{
    OptimizationReport, OptimizationResult, OptimizeMetric, ParamRange, ParamValue,
    sort_results_best_first,
};

// ── GridSearch ────────────────────────────────────────────────────────────────

/// Exhaustive grid-search optimiser for backtesting strategy parameters.
///
/// Evaluates every combination of the supplied parameter ranges in parallel.
/// Use [`BayesianSearch`] instead when the cartesian product would exceed
/// a few thousand combinations or when float ranges without a step are needed.
///
/// # Overfitting Warning
///
/// Results are **in-sample only**. Follow up with [`WalkForwardConfig`] or a
/// held-out test window to obtain an unbiased out-of-sample estimate.
///
/// [`BayesianSearch`]: super::BayesianSearch
/// [`WalkForwardConfig`]: super::super::walk_forward::WalkForwardConfig
#[derive(Debug, Clone, Default)]
pub struct GridSearch {
    /// Named parameter ranges, in insertion order (for reproducibility).
    params: Vec<(String, ParamRange)>,
    /// Metric to maximise (defaults to `SharpeRatio`).
    metric: Option<OptimizeMetric>,
}

impl GridSearch {
    /// Create a new grid search with no parameters defined yet.
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a named parameter range to sweep.
    ///
    /// Parameters are expanded in cartesian-product order: the last parameter
    /// added cycles fastest (inner loop).
    pub fn param(mut self, name: impl Into<String>, range: ParamRange) -> Self {
        self.params.push((name.into(), range));
        self
    }

    /// Set the metric to optimise for (defaults to [`OptimizeMetric::SharpeRatio`]).
    pub fn optimize_for(mut self, metric: OptimizeMetric) -> Self {
        self.metric = Some(metric);
        self
    }

    /// Run the grid search.
    ///
    /// `symbol` is used only for labelling in the returned results.
    ///
    /// `factory` receives the current parameter map and returns a strategy
    /// instance. Combinations that exceed the strategy's warmup period are
    /// silently skipped.
    ///
    /// Returns an error when the grid is empty or all combinations were skipped.
    pub fn run<S, F>(
        &self,
        symbol: &str,
        candles: &[Candle],
        config: &BacktestConfig,
        factory: F,
    ) -> Result<OptimizationReport>
    where
        S: Strategy + Send,
        F: Fn(&HashMap<String, ParamValue>) -> S + Send + Sync,
    {
        if self.params.is_empty() {
            return Err(BacktestError::invalid_param(
                "params",
                "grid search requires at least one parameter range",
            ));
        }

        let metric = self.metric.unwrap_or(OptimizeMetric::SharpeRatio);

        let expanded: Vec<(&str, Vec<ParamValue>)> = self
            .params
            .iter()
            .map(|(name, range)| (name.as_str(), range.expand()))
            .collect();

        let combinations = cartesian_product(&expanded);
        let total_combinations = combinations.len();

        if total_combinations == 0 {
            return Err(BacktestError::invalid_param(
                "params",
                "all parameter ranges produced empty value sets \
                 (hint: float_bounds is not compatible with GridSearch — use BayesianSearch)",
            ));
        }

        if total_combinations > 10_000 {
            tracing::warn!(
                total_combinations,
                "grid search: large combination count — consider BayesianSearch or wider steps"
            );
        }

        let skipped_errors = AtomicUsize::new(0);
        let mut results: Vec<OptimizationResult> = combinations
            .into_par_iter()
            .filter_map(|params| {
                let strategy = factory(&params);
                match BacktestEngine::new(config.clone()).run(symbol, candles, strategy) {
                    Ok(result) => Some(OptimizationResult { params, result }),
                    Err(BacktestError::InsufficientData { .. }) => None,
                    Err(e) => {
                        tracing::warn!(
                            params = ?params,
                            error = %e,
                            "grid search: skipping combination due to unexpected error"
                        );
                        skipped_errors.fetch_add(1, Ordering::Relaxed);
                        None
                    }
                }
            })
            .collect();
        let skipped_errors = skipped_errors.into_inner();

        if results.is_empty() {
            return Err(BacktestError::invalid_param(
                "candles",
                "no parameter combination had enough data to run",
            ));
        }

        sort_results_best_first(&mut results, metric);

        if metric.score(&results[0].result).is_nan() {
            return Err(BacktestError::invalid_param(
                "metric",
                "all parameter combinations produced NaN for the target metric",
            ));
        }

        let strategy_name = results[0].result.strategy_name.clone();
        let best = results[0].clone();
        let n_evaluations = total_combinations;

        Ok(OptimizationReport {
            strategy_name,
            total_combinations,
            results,
            best,
            skipped_errors,
            // GridSearch runs all combinations in parallel — no sequential ordering,
            // so the convergence curve is meaningless and left empty.
            convergence_curve: vec![],
            n_evaluations,
        })
    }
}

// ── Internal helpers ──────────────────────────────────────────────────────────

/// Compute the cartesian product of named parameter value lists.
///
/// Returns a `Vec` of `HashMap`s, one per combination. The last parameter
/// cycles fastest (inner loop).
fn cartesian_product(params: &[(&str, Vec<ParamValue>)]) -> Vec<HashMap<String, ParamValue>> {
    if params.is_empty() {
        return vec![];
    }

    let mut result: Vec<HashMap<String, ParamValue>> = vec![HashMap::new()];

    for (name, values) in params {
        let mut next = Vec::with_capacity(result.len() * values.len());
        for existing in &result {
            for value in values {
                let mut combo = existing.clone();
                combo.insert(name.to_string(), value.clone());
                next.push(combo);
            }
        }
        result = next;
    }

    result
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::backtesting::{BacktestConfig, SmaCrossover};
    use crate::models::chart::Candle;

    fn make_candles(prices: &[f64]) -> Vec<Candle> {
        prices
            .iter()
            .enumerate()
            .map(|(i, &p)| Candle {
                timestamp: i as i64,
                open: p,
                high: p * 1.01,
                low: p * 0.99,
                close: p,
                volume: 1000,
                adj_close: Some(p),
            })
            .collect()
    }

    fn trending_prices(n: usize) -> Vec<f64> {
        (0..n).map(|i| 100.0 + i as f64 * 0.5).collect()
    }

    // ── ParamValue ────────────────────────────────────────────────────────────

    #[test]
    fn test_param_value_conversion() {
        let iv = ParamValue::Int(10);
        assert_eq!(iv.as_int(), 10);
        assert!((iv.as_float() - 10.0).abs() < f64::EPSILON);

        let fv = ParamValue::Float(1.5);
        assert_eq!(fv.as_int(), 1);
        assert!((fv.as_float() - 1.5).abs() < f64::EPSILON);
    }

    // ── ParamRange expansion (grid path) ─────────────────────────────────────

    #[test]
    fn test_int_range_expand() {
        let r = ParamRange::int_range(5, 20, 5);
        let vals = r.expand();
        assert_eq!(
            vals,
            vec![
                ParamValue::Int(5),
                ParamValue::Int(10),
                ParamValue::Int(15),
                ParamValue::Int(20),
            ]
        );
    }

    #[test]
    fn test_float_range_expand() {
        let r = ParamRange::float_range(0.1, 0.3, 0.1);
        let vals = r.expand();
        assert_eq!(vals.len(), 3);
        assert!((vals[0].as_float() - 0.1).abs() < 1e-9);
        assert!((vals[2].as_float() - 0.3).abs() < 1e-9);
    }

    /// Floating-point arithmetic can produce `start + N * step` slightly above
    /// `end`. The endpoint must be clamped exactly to `end` with no extra values.
    #[test]
    fn test_float_range_endpoint_clamping() {
        let vals = ParamRange::float_range(0.1, 0.5, 0.1).expand();
        assert_eq!(vals.len(), 5, "should have exactly 5 values [0.1…0.5]");
        assert!(
            (vals[4].as_float() - 0.5).abs() < 1e-12,
            "endpoint must be exactly 0.5"
        );

        // step that doesn't evenly divide the range
        let vals2 = ParamRange::float_range(0.1, 0.5, 0.15).expand();
        assert_eq!(vals2.len(), 4);
        assert!((vals2[3].as_float() - 0.5).abs() < 1e-12);
    }

    #[test]
    fn test_float_bounds_expand_returns_empty() {
        // float_bounds has step=0.0, which is intentionally invalid for GridSearch.
        let r = ParamRange::float_bounds(0.1, 0.9);
        assert!(r.expand().is_empty());
    }

    // ── ParamRange sampling (Bayesian path) ───────────────────────────────────

    #[test]
    fn test_int_bounds_sample_at() {
        let r = ParamRange::int_bounds(5, 50);
        assert_eq!(r.sample_at(0.0), ParamValue::Int(5));
        assert_eq!(r.sample_at(1.0), ParamValue::Int(50));
        assert!(matches!(r.sample_at(0.5), ParamValue::Int(_)));
    }

    #[test]
    fn test_float_bounds_sample_at() {
        let r = ParamRange::float_bounds(0.3, 0.7);
        assert!((r.sample_at(0.0).as_float() - 0.3).abs() < 1e-12);
        assert!((r.sample_at(1.0).as_float() - 0.7).abs() < 1e-12);
        assert!((r.sample_at(0.5).as_float() - 0.5).abs() < 1e-12);
        assert!(matches!(r.sample_at(0.5), ParamValue::Float(_)));
    }

    #[test]
    fn test_sample_at_int_range() {
        let r = ParamRange::int_bounds(0, 9);
        assert_eq!(r.sample_at(0.0), ParamValue::Int(0));
        assert_eq!(r.sample_at(1.0), ParamValue::Int(9));
        assert_eq!(r.sample_at(0.5), ParamValue::Int(5));
    }

    #[test]
    fn test_sample_at_values_range() {
        let r = ParamRange::Values(vec![
            ParamValue::Int(10),
            ParamValue::Int(20),
            ParamValue::Int(30),
        ]);
        assert_eq!(r.sample_at(0.0), ParamValue::Int(10));
        assert_eq!(r.sample_at(1.0), ParamValue::Int(30));
        assert_eq!(r.sample_at(0.5), ParamValue::Int(20));
    }

    // ── cartesian_product ─────────────────────────────────────────────────────

    #[test]
    fn test_cartesian_product() {
        let params: Vec<(&str, Vec<ParamValue>)> = vec![
            ("a", vec![ParamValue::Int(1), ParamValue::Int(2)]),
            ("b", vec![ParamValue::Int(10), ParamValue::Int(20)]),
        ];
        let combos = cartesian_product(&params);
        assert_eq!(combos.len(), 4);
    }

    // ── GridSearch integration ────────────────────────────────────────────────

    #[test]
    fn test_grid_search_runs() {
        let prices = trending_prices(100);
        let candles = make_candles(&prices);
        let config = BacktestConfig::builder()
            .commission_pct(0.0)
            .slippage_pct(0.0)
            .build()
            .unwrap();

        let report = GridSearch::new()
            .param("fast", ParamRange::int_range(3, 10, 3))
            .param("slow", ParamRange::int_range(10, 20, 10))
            .optimize_for(OptimizeMetric::TotalReturn)
            .run("TEST", &candles, &config, |params| {
                SmaCrossover::new(
                    params["fast"].as_int() as usize,
                    params["slow"].as_int() as usize,
                )
            })
            .unwrap();

        assert!(!report.results.is_empty());
        assert_eq!(report.strategy_name, "SMA Crossover");
        assert!(
            report.convergence_curve.is_empty(),
            "GridSearch curve should be empty"
        );
        assert_eq!(report.n_evaluations, report.total_combinations);

        if report.results.len() > 1 {
            let first = OptimizeMetric::TotalReturn.score(&report.results[0].result);
            let second = OptimizeMetric::TotalReturn.score(&report.results[1].result);
            assert!(first >= second);
        }
    }

    #[test]
    fn test_grid_search_no_params_errors() {
        let candles = make_candles(&trending_prices(50));
        let config = BacktestConfig::default();
        let result = GridSearch::new().run("TEST", &candles, &config, |_| SmaCrossover::new(5, 10));
        assert!(result.is_err());
    }

    #[test]
    fn test_grid_search_float_bounds_errors() {
        // float_bounds is incompatible with GridSearch (step=0.0 → empty expansion).
        let candles = make_candles(&trending_prices(100));
        let config = BacktestConfig::default();
        let result = GridSearch::new()
            .param("x", ParamRange::float_bounds(0.1, 0.9))
            .run("TEST", &candles, &config, |_| SmaCrossover::new(5, 20));
        assert!(result.is_err());
    }

    #[test]
    fn test_optimize_metric_min_drawdown() {
        let prices = trending_prices(60);
        let candles = make_candles(&prices);
        let config = BacktestConfig::builder()
            .commission_pct(0.0)
            .slippage_pct(0.0)
            .build()
            .unwrap();

        let report = GridSearch::new()
            .param("fast", ParamRange::int_range(3, 9, 3))
            .param("slow", ParamRange::int_range(10, 20, 10))
            .optimize_for(OptimizeMetric::MinDrawdown)
            .run("TEST", &candles, &config, |params| {
                SmaCrossover::new(
                    params["fast"].as_int() as usize,
                    params["slow"].as_int() as usize,
                )
            })
            .unwrap();

        assert!(!report.results.is_empty());
        if report.results.len() > 1 {
            let first = report.results[0].result.metrics.max_drawdown_pct;
            let second = report.results[1].result.metrics.max_drawdown_pct;
            assert!(first <= second + 1e-9, "best has smallest drawdown");
        }
    }
}
