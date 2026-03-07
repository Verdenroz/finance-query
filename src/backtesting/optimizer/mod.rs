//! Parameter optimisation for backtesting strategies.
//!
//! Two optimisers are available, both returning the same [`OptimizationReport`]
//! so they are drop-in interchangeable and both work with [`WalkForwardConfig`]:
//!
//! | Optimiser | Evaluations | When to use |
//! |-----------|-------------|-------------|
//! | [`GridSearch`] | O(nᵏ) — all combinations | ≤ 3 parameters, small step counts |
//! | [`BayesianSearch`] | configurable (default 100) | 4+ parameters or continuous float ranges |
//!
//! [`WalkForwardConfig`]: super::walk_forward::WalkForwardConfig

mod bayesian;
mod grid;

pub use bayesian::BayesianSearch;
pub use grid::GridSearch;

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use super::result::BacktestResult;

// ── Parameter types ───────────────────────────────────────────────────────────

/// A single parameter value — either an integer period or a float multiplier.
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

// ── Parameter ranges ──────────────────────────────────────────────────────────

/// Defines the search space for a single strategy parameter.
///
/// | Constructor | Compatible with | Typical use |
/// |-------------|-----------------|-------------|
/// | [`int_range(start, end, step)`] | GridSearch + BayesianSearch | Integer period with explicit grid step |
/// | [`float_range(start, end, step)`] | GridSearch + BayesianSearch | Float multiplier with explicit grid step |
/// | [`int_bounds(start, end)`] | GridSearch (step=1) + BayesianSearch | Integer period, let Bayesian sample freely |
/// | [`float_bounds(start, end)`] | **BayesianSearch only** | Continuous float range |
/// | [`Values(vec)`] | GridSearch + BayesianSearch | Explicit list of values |
///
/// [`int_range(start, end, step)`]: ParamRange::int_range
/// [`float_range(start, end, step)`]: ParamRange::float_range
/// [`int_bounds(start, end)`]: ParamRange::int_bounds
/// [`float_bounds(start, end)`]: ParamRange::float_bounds
/// [`Values(vec)`]: ParamRange::Values
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
    /// Stepped integer range — compatible with both [`GridSearch`] and [`BayesianSearch`].
    pub fn int_range(start: i64, end: i64, step: i64) -> Self {
        Self::IntRange { start, end, step }
    }

    /// Stepped float range — compatible with both [`GridSearch`] and [`BayesianSearch`].
    pub fn float_range(start: f64, end: f64, step: f64) -> Self {
        Self::FloatRange { start, end, step }
    }

    /// Continuous integer bounds for [`BayesianSearch`].
    ///
    /// Equivalent to `int_range(start, end, 1)`. Also usable with [`GridSearch`]
    /// (enumerates every integer in `[start, end]`), but prefer `int_range` with a
    /// wider step when the grid would be very large.
    pub fn int_bounds(start: i64, end: i64) -> Self {
        Self::IntRange {
            start,
            end,
            step: 1,
        }
    }

    /// Continuous float bounds — **[`BayesianSearch`] only**.
    ///
    /// A step of `0.0` intentionally makes [`GridSearch`] return an error, giving
    /// a clear signal when the wrong optimiser is used with this range type.
    pub fn float_bounds(start: f64, end: f64) -> Self {
        Self::FloatRange {
            start,
            end,
            step: 0.0,
        }
    }

    /// Map a normalised position `t ∈ [0.0, 1.0]` to a concrete [`ParamValue`].
    ///
    /// Used by [`BayesianSearch`] to translate unit-hypercube coordinates into
    /// the actual parameter space.
    pub(crate) fn sample_at(&self, t: f64) -> ParamValue {
        let t = t.clamp(0.0, 1.0);
        match self {
            ParamRange::IntRange { start, end, .. } => {
                // Map t uniformly over [start, end] (inclusive on both ends).
                let span = (*end - *start) as f64;
                let v = *start + (t * (span + 1.0)).floor() as i64;
                ParamValue::Int(v.min(*end))
            }
            ParamRange::FloatRange { start, end, .. } => {
                ParamValue::Float(start + t * (end - start))
            }
            ParamRange::Values(vals) if vals.is_empty() => ParamValue::Int(0),
            ParamRange::Values(vals) => {
                let idx = (t * vals.len() as f64).floor() as usize;
                vals[idx.min(vals.len() - 1)].clone()
            }
        }
    }

    /// Expand the range into a flat `Vec<ParamValue>` for grid enumeration.
    ///
    /// Returns an empty `Vec` when the step is `≤ 0`, which causes [`GridSearch`]
    /// to return an error. This is intentional for [`float_bounds`] ranges.
    ///
    /// [`float_bounds`]: ParamRange::float_bounds
    pub(crate) fn expand(&self) -> Vec<ParamValue> {
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
                // Round the step count to avoid accumulated floating-point error.
                // The last value is clamped to exactly `end` regardless of rounding.
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

// ── Metric selection ──────────────────────────────────────────────────────────

/// Which performance metric to optimise for.
///
/// All metrics are maximised internally; [`MinDrawdown`] is negated so that
/// a smaller drawdown produces a higher score.
///
/// [`MinDrawdown`]: OptimizeMetric::MinDrawdown
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum OptimizeMetric {
    /// Maximise total return percentage
    TotalReturn,
    /// Maximise Sharpe ratio (risk-adjusted, uses `risk_free_rate` from config)
    SharpeRatio,
    /// Maximise Sortino ratio
    SortinoRatio,
    /// Maximise Calmar ratio
    CalmarRatio,
    /// Maximise profit factor (gross profit / gross loss)
    ProfitFactor,
    /// Maximise win rate
    WinRate,
    /// Minimise maximum drawdown (negated internally — lower drawdown = higher score)
    MinDrawdown,
}

impl OptimizeMetric {
    /// Extract the target score from a [`BacktestResult`]. Higher is always better.
    pub(crate) fn score(&self, result: &BacktestResult) -> f64 {
        match self {
            OptimizeMetric::TotalReturn => result.metrics.total_return_pct,
            OptimizeMetric::SharpeRatio => result.metrics.sharpe_ratio,
            OptimizeMetric::SortinoRatio => result.metrics.sortino_ratio,
            OptimizeMetric::CalmarRatio => result.metrics.calmar_ratio,
            OptimizeMetric::ProfitFactor => result.metrics.profit_factor,
            OptimizeMetric::WinRate => result.metrics.win_rate,
            OptimizeMetric::MinDrawdown => -result.metrics.max_drawdown_pct,
        }
    }
}

// ── Result types ──────────────────────────────────────────────────────────────

/// Result of a single parameter set evaluation.
#[non_exhaustive]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OptimizationResult {
    /// Parameter values used for this run
    pub params: HashMap<String, ParamValue>,
    /// The backtest result for these parameter values
    pub result: BacktestResult,
}

/// Optimisation report returned by both [`GridSearch`] and [`BayesianSearch`].
///
/// # Overfitting Warning
///
/// All metrics are **in-sample** — the same candle data used to optimise the
/// parameters is used to score them. In-sample results almost always overstate
/// real-world performance.
///
/// **Always validate best parameters on unseen data** — use [`WalkForwardConfig`]
/// for an unbiased out-of-sample estimate, or reserve a held-out test period.
///
/// [`WalkForwardConfig`]: super::walk_forward::WalkForwardConfig
#[non_exhaustive]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OptimizationReport {
    /// Name of the strategy being optimised
    pub strategy_name: String,
    /// Total number of successful parameter evaluations
    pub total_combinations: usize,
    /// All results sorted best-first by the target metric.
    ///
    /// Sets that fail due to insufficient data are silently skipped.
    /// **In-sample only** — see struct-level docs.
    pub results: Vec<OptimizationResult>,
    /// The single best result (same object as `results[0]`).
    ///
    /// **In-sample only** — see struct-level docs.
    pub best: OptimizationResult,
    /// Number of combinations skipped due to unexpected errors (not insufficient
    /// data). A non-zero value indicates a configuration problem.
    pub skipped_errors: usize,
    /// Running best metric value after each **successful** evaluation, in order.
    ///
    /// [`BayesianSearch`] populates this as a non-decreasing convergence trace.
    /// [`GridSearch`] leaves it empty — parallel execution has no sequential order.
    pub convergence_curve: Vec<f64>,
    /// Total strategy evaluations **attempted** (including those skipped for
    /// insufficient data).
    ///
    /// For [`GridSearch`]: equals `total_combinations`.
    /// For [`BayesianSearch`]: equals `max_evaluations` (or fewer if data is short).
    pub n_evaluations: usize,
}

/// Sort a `Vec<OptimizationResult>` best-first for a given metric.
///
/// NaN scores sort last so they never appear as "best".  Shared by both
/// [`GridSearch`] and [`BayesianSearch`] to keep the sorting logic in one place.
pub(crate) fn sort_results_best_first(results: &mut [OptimizationResult], metric: OptimizeMetric) {
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
}
