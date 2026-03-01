//! Grid-search parameter optimisation for backtesting strategies.
//!
//! Use [`GridSearch`] to sweep over parameter combinations and rank them by a
//! chosen metric. Results are returned as an [`OptimizationReport`] sorted
//! best-first.
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
//!     .run(
//!         "AAPL",
//!         candles,
//!         &BacktestConfig::default(),
//!         |params| SmaCrossover::new(
//!             params["fast"].as_int() as usize,
//!             params["slow"].as_int() as usize,
//!         ),
//!     )
//!     .unwrap();
//!
//! println!("Best params: {:?}", report.best.params);
//! println!("Best Sharpe: {:.2}", report.best.result.metrics.sharpe_ratio);
//! # }
//! ```

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::models::chart::Candle;

use super::config::BacktestConfig;
use super::engine::BacktestEngine;
use super::error::{BacktestError, Result};
use super::result::BacktestResult;
use super::strategy::Strategy;

// ── Parameter types ──────────────────────────────────────────────────────────

/// A single parameter value used in grid-search.
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ParamValue {
    /// Integer parameter (e.g. a period length)
    Int(i64),
    /// Floating-point parameter (e.g. a multiplier or percentage)
    Float(f64),
}

impl ParamValue {
    /// Return the value as `i64`, truncating floats.
    pub fn as_int(&self) -> i64 {
        match self {
            ParamValue::Int(v) => *v,
            ParamValue::Float(v) => *v as i64,
        }
    }

    /// Return the value as `f64`.
    pub fn as_float(&self) -> f64 {
        match self {
            ParamValue::Int(v) => *v as f64,
            ParamValue::Float(v) => *v,
        }
    }
}

impl std::fmt::Display for ParamValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ParamValue::Int(v) => write!(f, "{v}"),
            ParamValue::Float(v) => write!(f, "{v:.4}"),
        }
    }
}

/// A range of parameter values to sweep over during grid search.
#[non_exhaustive]
#[derive(Debug, Clone)]
pub enum ParamRange {
    /// Inclusive integer range with a step size.
    IntRange {
        /// First value to include
        start: i64,
        /// Last value to include (inclusive)
        end: i64,
        /// Increment between values
        step: i64,
    },
    /// Inclusive float range with a step size.
    FloatRange {
        /// First value in the range
        start: f64,
        /// Last value to include (inclusive)
        end: f64,
        /// Increment between values
        step: f64,
    },
    /// Explicit list of values.
    Values(Vec<ParamValue>),
}

impl ParamRange {
    /// Convenience constructor for integer ranges.
    pub fn int_range(start: i64, end: i64, step: i64) -> Self {
        Self::IntRange { start, end, step }
    }

    /// Convenience constructor for float ranges.
    pub fn float_range(start: f64, end: f64, step: f64) -> Self {
        Self::FloatRange { start, end, step }
    }

    /// Expand the range into a flat `Vec<ParamValue>`.
    fn expand(&self) -> Vec<ParamValue> {
        match self {
            ParamRange::IntRange { start, end, step } => {
                if *step <= 0 {
                    return vec![];
                }
                let mut v = Vec::new();
                let mut cur = *start;
                while cur <= *end {
                    v.push(ParamValue::Int(cur));
                    cur += step;
                }
                v
            }
            ParamRange::FloatRange { start, end, step } => {
                if *step <= 0.0 {
                    return vec![];
                }
                // Compute the number of steps by rounding to avoid accumulated
                // floating-point error in repeated addition.  The last value is
                // clamped to exactly `end` so callers always see the requested
                // boundary regardless of rounding noise (e.g. 0.1 + 4*0.1 might
                // evaluate to 0.5000000000000001 without the clamp).
                let steps = ((end - start) / step).round() as usize;
                (0..=steps)
                    .map(|i| {
                        let v = if i == steps {
                            *end
                        } else {
                            start + i as f64 * step
                        };
                        ParamValue::Float(v)
                    })
                    .collect()
            }
            ParamRange::Values(vals) => vals.clone(),
        }
    }
}

// ── Metric selection ─────────────────────────────────────────────────────────

/// Which metric to optimise for during grid search.
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum OptimizeMetric {
    /// Maximise total return percentage
    TotalReturn,
    /// Maximise Sharpe ratio (accounts for risk-free rate from config)
    SharpeRatio,
    /// Maximise Sortino ratio
    SortinoRatio,
    /// Maximise Calmar ratio
    CalmarRatio,
    /// Maximise profit factor (gross profit / gross loss)
    ProfitFactor,
    /// Maximise win rate
    WinRate,
    /// Minimise maximum drawdown (lower is better; metric is negated for sorting)
    MinDrawdown,
}

impl OptimizeMetric {
    /// Extract the target value from a `BacktestResult`. Higher is always better.
    fn score(&self, result: &BacktestResult) -> f64 {
        match self {
            OptimizeMetric::TotalReturn => result.metrics.total_return_pct,
            OptimizeMetric::SharpeRatio => result.metrics.sharpe_ratio,
            OptimizeMetric::SortinoRatio => result.metrics.sortino_ratio,
            OptimizeMetric::CalmarRatio => result.metrics.calmar_ratio,
            OptimizeMetric::ProfitFactor => result.metrics.profit_factor,
            OptimizeMetric::WinRate => result.metrics.win_rate,
            // Negate drawdown so that a lower drawdown gives a higher score
            OptimizeMetric::MinDrawdown => -result.metrics.max_drawdown_pct,
        }
    }
}

// ── Result types ─────────────────────────────────────────────────────────────

/// Result of a single parameter combination in a grid search.
#[non_exhaustive]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OptimizationResult {
    /// Parameter values used for this run
    pub params: HashMap<String, ParamValue>,
    /// The backtest result for these parameter values
    pub result: BacktestResult,
}

/// Full grid-search report returned by [`GridSearch::run`].
#[non_exhaustive]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OptimizationReport {
    /// Name of the strategy being optimised
    pub strategy_name: String,
    /// Total number of parameter combinations evaluated
    pub total_combinations: usize,
    /// All results sorted best-first by the target metric.
    ///
    /// Combinations that fail to run (e.g. insufficient data) are silently
    /// skipped and not included here.
    pub results: Vec<OptimizationResult>,
    /// The single best result (same as `results[0]`)
    pub best: OptimizationResult,
    /// Number of combinations skipped due to unexpected errors (not just
    /// insufficient data). A non-zero value indicates a configuration problem.
    pub skipped_errors: usize,
}

// ── GridSearch ────────────────────────────────────────────────────────────────

/// Grid-search optimiser for backtesting strategy parameters.
///
/// Build with [`GridSearch::new`], add parameter ranges with [`GridSearch::param`],
/// select a target metric with [`GridSearch::optimize_for`], then call
/// [`GridSearch::run`] with a closure that constructs your strategy from the
/// parameter map.
#[derive(Debug, Clone, Default)]
pub struct GridSearch {
    /// Named parameter ranges, in insertion order (for reproducibility)
    params: Vec<(String, ParamRange)>,
    /// Metric to maximise
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
    /// `factory` is a closure that receives a reference to the current parameter
    /// map and returns a strategy instance. Any parameter combination that
    /// produces insufficient data for the strategy's warmup period is silently
    /// skipped.
    ///
    /// Returns an error only when the grid is empty (no parameters defined).
    pub fn run<S, F>(
        &self,
        symbol: &str,
        candles: &[Candle],
        config: &BacktestConfig,
        factory: F,
    ) -> Result<OptimizationReport>
    where
        S: Strategy,
        F: Fn(&HashMap<String, ParamValue>) -> S,
    {
        if self.params.is_empty() {
            return Err(BacktestError::invalid_param(
                "params",
                "grid search requires at least one parameter range",
            ));
        }

        let metric = self.metric.unwrap_or(OptimizeMetric::SharpeRatio);
        let engine = BacktestEngine::new(config.clone());

        // Expand each parameter range into a list of values
        let expanded: Vec<(&str, Vec<ParamValue>)> = self
            .params
            .iter()
            .map(|(name, range)| (name.as_str(), range.expand()))
            .collect();

        // Generate the cartesian product of all parameter combinations
        let combinations = cartesian_product(&expanded);
        let total_combinations = combinations.len();

        if total_combinations == 0 {
            return Err(BacktestError::invalid_param(
                "params",
                "all parameter ranges produced empty value sets",
            ));
        }

        // Run the engine for each combination, skip insufficient-data errors
        let mut skipped_errors: usize = 0;
        let mut results: Vec<OptimizationResult> = combinations
            .into_iter()
            .filter_map(|params| {
                let strategy = factory(&params);
                match engine.run(symbol, candles, strategy) {
                    Ok(result) => Some(OptimizationResult { params, result }),
                    Err(BacktestError::InsufficientData { .. }) => None,
                    Err(e) => {
                        tracing::warn!(
                            params = ?params,
                            error = %e,
                            "grid search: skipping combination due to unexpected error"
                        );
                        skipped_errors += 1;
                        None
                    }
                }
            })
            .collect();

        if results.is_empty() {
            return Err(BacktestError::invalid_param(
                "candles",
                "no parameter combination had enough data to run",
            ));
        }

        // Sort best-first; NaN scores sort last so they never appear as "best".
        results.sort_by(|a, b| {
            let sa = metric.score(&a.result);
            let sb = metric.score(&b.result);
            match (sa.is_nan(), sb.is_nan()) {
                (true, true) => std::cmp::Ordering::Equal,
                (true, false) => std::cmp::Ordering::Greater, // NaN → last
                (false, true) => std::cmp::Ordering::Less,    // non-NaN → first
                (false, false) => sb.partial_cmp(&sa).unwrap_or(std::cmp::Ordering::Equal),
            }
        });

        // All combinations returned NaN for the chosen metric — nothing useful to report.
        if metric.score(&results[0].result).is_nan() {
            return Err(BacktestError::invalid_param(
                "metric",
                "all parameter combinations produced NaN for the target metric",
            ));
        }

        let strategy_name = results[0].result.strategy_name.clone();
        let best = results[0].clone();

        Ok(OptimizationReport {
            strategy_name,
            total_combinations,
            results,
            best,
            skipped_errors,
        })
    }
}

// ── Internal helpers ──────────────────────────────────────────────────────────

/// Compute the cartesian product of named parameter value lists.
///
/// Returns a `Vec` of `HashMap`s, one per combination. The last parameter
/// in `params` cycles fastest (inner loop).
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
    use crate::backtesting::SmaCrossover;
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

    /// Generate a simple trending price series (gradually upward).
    fn trending_prices(n: usize) -> Vec<f64> {
        (0..n).map(|i| 100.0 + i as f64 * 0.5).collect()
    }

    #[test]
    fn test_param_value_conversion() {
        let iv = ParamValue::Int(10);
        assert_eq!(iv.as_int(), 10);
        assert!((iv.as_float() - 10.0).abs() < f64::EPSILON);

        let fv = ParamValue::Float(1.5);
        assert_eq!(fv.as_int(), 1);
        assert!((fv.as_float() - 1.5).abs() < f64::EPSILON);
    }

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
    /// `end` (e.g. 0.1 + 4*0.1 = 0.5000000000000001).  The endpoint must be
    /// exactly `end`, not a rounding artefact, and no extra values beyond `end`
    /// should appear.
    #[test]
    fn test_float_range_endpoint_clamping() {
        // (0.5 - 0.1) / 0.1 = 4.0 — exact in theory but sometimes 3.9999…
        let vals = ParamRange::float_range(0.1, 0.5, 0.1).expand();
        assert_eq!(
            vals.len(),
            5,
            "should have exactly 5 values [0.1, 0.2, 0.3, 0.4, 0.5]"
        );
        assert!(
            (vals[4].as_float() - 0.5).abs() < 1e-12,
            "endpoint must be exactly 0.5"
        );

        // step that doesn't evenly divide the range — only 3 steps fit before 0.5
        let vals2 = ParamRange::float_range(0.1, 0.5, 0.15).expand();
        // steps = round((0.5-0.1)/0.15) = round(2.666) = 3 → 4 values: 0.1, 0.25, 0.40, 0.50
        assert_eq!(vals2.len(), 4);
        assert!(
            (vals2[3].as_float() - 0.5).abs() < 1e-12,
            "endpoint must be exactly 0.5"
        );
    }

    #[test]
    fn test_cartesian_product() {
        let params: Vec<(&str, Vec<ParamValue>)> = vec![
            ("a", vec![ParamValue::Int(1), ParamValue::Int(2)]),
            ("b", vec![ParamValue::Int(10), ParamValue::Int(20)]),
        ];
        let combos = cartesian_product(&params);
        assert_eq!(combos.len(), 4);
    }

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

        // Should have run combinations and return the best first
        assert!(!report.results.is_empty());
        assert_eq!(report.strategy_name, "SMA Crossover");
        // Results should be sorted best-first
        if report.results.len() > 1 {
            let first_score = OptimizeMetric::TotalReturn.score(&report.results[0].result);
            let second_score = OptimizeMetric::TotalReturn.score(&report.results[1].result);
            assert!(first_score >= second_score);
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
    fn test_optimize_metric_min_drawdown() {
        // MinDrawdown should prefer results with smaller drawdown (negated score)
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
            assert!(first <= second + 1e-9); // best has smallest drawdown
        }
    }
}
