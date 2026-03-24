//! Sequential model-based (Bayesian) parameter optimisation.
//!
//! [`BayesianSearch`] finds near-optimal strategy parameters in far fewer
//! backtests than exhaustive [`GridSearch`] — typically 50–200 evaluations
//! instead of thousands — by building a statistical surrogate model of the
//! objective and directing search toward promising, under-explored regions.
//!
//! # Algorithm (SAMBO — Sequential Adaptive Model-Based Optimisation)
//!
//! 1. **Exploration phase** — Sample `initial_points` parameter sets using
//!    [Latin Hypercube Sampling] (LHS) to guarantee good initial coverage of
//!    the search space.
//! 2. **Sequential phase** — Fit a [Nadaraya-Watson kernel regression]
//!    surrogate to all `(params, score)` observations. Generate `N_CANDIDATES`
//!    random candidates and score each with the [Upper Confidence Bound] (UCB)
//!    acquisition function `a(x) = μ(x) + β·σ(x)`. Run the backtest for the
//!    highest-scoring candidate, add the observation, and repeat.
//! 3. **Convergence** — Stop when `max_evaluations` is reached.
//!
//! [Latin Hypercube Sampling]: https://en.wikipedia.org/wiki/Latin_hypercube_sampling
//! [Nadaraya-Watson kernel regression]: https://en.wikipedia.org/wiki/Kernel_regression
//! [Upper Confidence Bound]: https://en.wikipedia.org/wiki/Multi-armed_bandit#Upper_confidence_bound
//!
//! # Example
//!
//! ```ignore
//! use finance_query::backtesting::{
//!     BacktestConfig, SmaCrossover,
//!     optimizer::{BayesianSearch, OptimizeMetric, ParamRange},
//! };
//!
//! # fn example(candles: &[finance_query::models::chart::Candle]) {
//! let report = BayesianSearch::new()
//!     .param("fast", ParamRange::int_bounds(5, 50))
//!     .param("slow", ParamRange::int_bounds(20, 200))
//!     .param("rsi_period", ParamRange::int_bounds(7, 21))
//!     .param("threshold", ParamRange::float_bounds(0.3, 0.7))
//!     .optimize_for(OptimizeMetric::SharpeRatio)
//!     .max_evaluations(100)
//!     .run("AAPL", &candles, &BacktestConfig::default(), |params| {
//!         SmaCrossover::new(
//!             params["fast"].as_int() as usize,
//!             params["slow"].as_int() as usize,
//!         )
//!     })
//!     .unwrap();
//!
//! println!("Best params:  {:?}", report.best.params);
//! println!("Best Sharpe:  {:.2}", report.best.result.metrics.sharpe_ratio);
//! println!("Evaluations:  {}", report.n_evaluations);
//! # }
//! ```

use std::collections::HashMap;

use crate::models::chart::Candle;

use super::super::config::BacktestConfig;
use super::super::engine::BacktestEngine;
use super::super::error::{BacktestError, Result};
use super::super::monte_carlo::Xorshift64;
use super::super::strategy::Strategy;
use super::{
    OptimizationReport, OptimizationResult, OptimizeMetric, ParamRange, ParamValue,
    sort_results_best_first,
};

// ── Defaults ──────────────────────────────────────────────────────────────────

const DEFAULT_MAX_EVALUATIONS: usize = 100;
const DEFAULT_INITIAL_POINTS: usize = 10;
/// β = 2.0 balances exploitation and exploration for objectives in [0, 1].
const DEFAULT_UCB_BETA: f64 = 2.0;
const DEFAULT_SEED: u64 = 42;
/// Candidates evaluated per acquisition step. 1 000 reliably finds the UCB
/// maximum without meaningful overhead (pure floating-point math, no backtests).
const N_CANDIDATES: usize = 1_000;

// ── BayesianSearch ────────────────────────────────────────────────────────────

/// Sequential model-based (Bayesian) parameter optimiser.
///
/// Finds near-optimal strategy parameters in a fraction of the evaluations
/// required by exhaustive [`GridSearch`], making it practical for
/// high-dimensional spaces or continuous float ranges.
///
/// Returns the same [`OptimizationReport`] as [`GridSearch`], so the two are
/// drop-in interchangeable and both work with [`WalkForwardConfig`].
///
/// # Overfitting Warning
///
/// Results are **in-sample only**. Follow up with [`WalkForwardConfig`] or a
/// held-out test window to obtain an unbiased out-of-sample estimate.
///
/// [`WalkForwardConfig`]: super::super::walk_forward::WalkForwardConfig
#[derive(Debug, Clone, Default)]
pub struct BayesianSearch {
    params: Vec<(String, ParamRange)>,
    metric: Option<OptimizeMetric>,
    max_evaluations: Option<usize>,
    initial_points: Option<usize>,
    ucb_beta: Option<f64>,
    seed: Option<u64>,
}

impl BayesianSearch {
    /// Create a new Bayesian search with no parameters defined yet.
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a named parameter range to search over.
    ///
    /// Use [`ParamRange::int_bounds`] / [`ParamRange::float_bounds`] for
    /// continuous ranges (recommended) or any [`ParamRange`] variant.
    pub fn param(mut self, name: impl Into<String>, range: ParamRange) -> Self {
        self.params.push((name.into(), range));
        self
    }

    /// Set the metric to optimise for (defaults to [`OptimizeMetric::SharpeRatio`]).
    pub fn optimize_for(mut self, metric: OptimizeMetric) -> Self {
        self.metric = Some(metric);
        self
    }

    /// Maximum total strategy evaluations, including the initial LHS phase (default: 100).
    pub fn max_evaluations(mut self, n: usize) -> Self {
        self.max_evaluations = Some(n);
        self
    }

    /// Number of initial random (LHS) samples before the surrogate is fitted (default: 10).
    ///
    /// Clamped to `[2, max_evaluations]`. More initial points improve surrogate
    /// quality at the cost of fewer sequential refinement steps.
    pub fn initial_points(mut self, n: usize) -> Self {
        self.initial_points = Some(n);
        self
    }

    /// UCB exploration–exploitation coefficient β (default: 2.0).
    ///
    /// Higher values drive broader exploration of uncertain regions;
    /// lower values concentrate search near already-good parameter sets.
    pub fn ucb_beta(mut self, beta: f64) -> Self {
        self.ucb_beta = Some(beta);
        self
    }

    /// PRNG seed for reproducible runs (default: 42).
    pub fn seed(mut self, seed: u64) -> Self {
        self.seed = Some(seed);
        self
    }

    /// Run the Bayesian search.
    ///
    /// `symbol` is used only for labelling in the returned results.
    ///
    /// `factory` receives the current parameter map and returns a strategy
    /// instance. Parameter sets incompatible with the candle series (warmup
    /// too long) are silently skipped.
    ///
    /// Returns an error only when no parameters are defined or every evaluation
    /// was skipped due to insufficient data.
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
                "BayesianSearch requires at least one parameter range",
            ));
        }

        let d = self.params.len();
        let metric = self.metric.unwrap_or(OptimizeMetric::SharpeRatio);
        let max_eval = self.max_evaluations.unwrap_or(DEFAULT_MAX_EVALUATIONS);
        let n_init = self
            .initial_points
            .unwrap_or(DEFAULT_INITIAL_POINTS)
            .max(2)
            .min(max_eval);
        let beta = self.ucb_beta.unwrap_or(DEFAULT_UCB_BETA);
        let seed = self.seed.unwrap_or(DEFAULT_SEED);

        let mut rng = Xorshift64::new(seed);
        // (unit-hypercube coords, metric score) for all successful evaluations.
        let mut observations: Vec<(Vec<f64>, f64)> = Vec::with_capacity(max_eval);
        let mut all_results: Vec<OptimizationResult> = Vec::with_capacity(max_eval);
        // Running best score after each successful evaluation (non-decreasing).
        let mut convergence_curve: Vec<f64> = Vec::with_capacity(max_eval);
        let mut n_evaluations: usize = 0;
        let mut best_score: Option<f64> = None;

        // ── Phase 1: Latin Hypercube initial sampling ──────────────────────────

        for norm_point in latin_hypercube_sample(n_init, d, &mut rng) {
            n_evaluations += 1;
            if let Some(opt_result) = run_one(
                symbol,
                candles,
                config,
                &metric,
                &factory,
                &norm_point,
                &self.params,
            ) {
                let score = metric.score(&opt_result.result);
                if score.is_finite() {
                    update_best(&mut best_score, score);
                    observations.push((norm_point, score));
                }
                if let Some(b) = best_score {
                    convergence_curve.push(b);
                }
                all_results.push(opt_result);
            }
        }

        // ── Phase 2: Sequential surrogate-guided search ────────────────────────

        for _ in 0..max_eval.saturating_sub(n_init) {
            let norm_point = if observations.len() < 2 {
                // Too few observations for a reliable surrogate — fall back to random.
                (0..d).map(|_| rng.next_f64_positive()).collect()
            } else {
                let surrogate = Surrogate::fit(&observations, beta);
                // Reuse a single candidate buffer across all N_CANDIDATES evaluations,
                // eliminating N_CANDIDATES heap allocations per sequential step.
                let mut candidate = vec![0.0_f64; d];
                let mut best_ucb = f64::NEG_INFINITY;
                let mut best = vec![0.0_f64; d];
                for _ in 0..N_CANDIDATES {
                    for xi in candidate.iter_mut() {
                        *xi = rng.next_f64_positive();
                    }
                    let ucb = surrogate.acquisition(&candidate);
                    if ucb > best_ucb {
                        best_ucb = ucb;
                        best.copy_from_slice(&candidate);
                    }
                }
                best
            };

            n_evaluations += 1;
            if let Some(opt_result) = run_one(
                symbol,
                candles,
                config,
                &metric,
                &factory,
                &norm_point,
                &self.params,
            ) {
                let score = metric.score(&opt_result.result);
                if score.is_finite() {
                    update_best(&mut best_score, score);
                    observations.push((norm_point, score));
                }
                if let Some(b) = best_score {
                    convergence_curve.push(b);
                }
                all_results.push(opt_result);
            }
        }

        // ── Finalise ───────────────────────────────────────────────────────────

        if all_results.is_empty() {
            return Err(BacktestError::invalid_param(
                "candles",
                "no parameter set had enough data to run a backtest",
            ));
        }

        sort_results_best_first(&mut all_results, metric);

        if metric.score(&all_results[0].result).is_nan() {
            return Err(BacktestError::invalid_param(
                "metric",
                "all parameter sets produced NaN for the target metric",
            ));
        }

        let strategy_name = all_results[0].result.strategy_name.clone();
        let best = all_results[0].clone();
        let total_combinations = all_results.len();

        Ok(OptimizationReport {
            strategy_name,
            total_combinations,
            results: all_results,
            best,
            skipped_errors: 0,
            convergence_curve,
            n_evaluations,
        })
    }
}

// ── Internal helpers ──────────────────────────────────────────────────────────

#[inline]
fn update_best(best: &mut Option<f64>, score: f64) {
    match best {
        None => *best = Some(score),
        Some(b) if score > *b => *b = score,
        _ => {}
    }
}

/// Run one backtest for a unit-hypercube point; returns `None` for
/// `InsufficientData` errors (silently skipped).
fn run_one<S, F>(
    symbol: &str,
    candles: &[Candle],
    config: &BacktestConfig,
    _metric: &OptimizeMetric,
    factory: &F,
    norm_point: &[f64],
    param_specs: &[(String, ParamRange)],
) -> Option<OptimizationResult>
where
    S: Strategy,
    F: Fn(&HashMap<String, ParamValue>) -> S,
{
    let params = denormalize(norm_point, param_specs);
    let strategy = factory(&params);
    match BacktestEngine::new(config.clone()).run(symbol, candles, strategy) {
        Ok(result) => Some(OptimizationResult { params, result }),
        Err(BacktestError::InsufficientData { .. }) => None,
        Err(e) => {
            tracing::warn!(
                params = ?params,
                error = %e,
                "BayesianSearch: skipping candidate due to unexpected error"
            );
            None
        }
    }
}

/// Convert unit-hypercube coordinates `t[i] ∈ (0, 1]` into named [`ParamValue`]s.
fn denormalize(
    norm_point: &[f64],
    param_specs: &[(String, ParamRange)],
) -> HashMap<String, ParamValue> {
    norm_point
        .iter()
        .zip(param_specs.iter())
        .map(|(&t, (name, range))| (name.clone(), range.sample_at(t)))
        .collect()
}

// ── Latin Hypercube Sampling ──────────────────────────────────────────────────

/// Generate `n` stratified random samples in the `d`-dimensional unit hypercube.
///
/// Each dimension is divided into `n` equal strata; exactly one sample is drawn
/// from each stratum per dimension. Stratum assignments are independently
/// shuffled across dimensions, giving good marginal coverage with low
/// inter-dimension correlation — significantly better than IID uniform sampling.
fn latin_hypercube_sample(n: usize, d: usize, rng: &mut Xorshift64) -> Vec<Vec<f64>> {
    if n == 0 {
        return vec![];
    }

    let mut samples = vec![vec![0.0_f64; d]; n];

    #[allow(clippy::needless_range_loop)]
    for dim in 0..d {
        // One value per stratum [i/n, (i+1)/n).
        let mut stratum_values: Vec<f64> = (0..n)
            .map(|i| {
                let lo = i as f64 / n as f64;
                let hi = (i + 1) as f64 / n as f64;
                lo + rng.next_f64_positive() * (hi - lo)
            })
            .collect();

        // Fisher-Yates shuffle of stratum assignments for this dimension.
        for i in (1..n).rev() {
            let j = rng.next_usize(i + 1);
            stratum_values.swap(i, j);
        }

        for i in 0..n {
            samples[i][dim] = stratum_values[i];
        }
    }

    samples
}

// ── Surrogate model ───────────────────────────────────────────────────────────

/// Nadaraya-Watson kernel regression surrogate with UCB acquisition.
///
/// Given observed `(x, y)` pairs (unit-hypercube coords and metric scores),
/// models the objective surface as a Gaussian-kernel-weighted average.
///
/// **Why kernel regression?** It is dependency-free, numerically stable,
/// non-parametric, and the mean/variance formulas are five lines of arithmetic.
/// The trade-off vs. a Gaussian Process is that it does not provide a
/// calibrated predictive distribution, but UCB acquisition works well in
/// practice for backtesting parameter search.
struct Surrogate<'a> {
    observations: &'a [(Vec<f64>, f64)],
    beta: f64,
    /// Pre-computed `2h²` denominator for the RBF kernel exponent.
    bandwidth_sq: f64,
}

impl<'a> Surrogate<'a> {
    /// Fit the surrogate to a set of `(unit-hypercube coords, score)` pairs.
    ///
    /// Bandwidth: `h = n^(-1/(d+4))` (Silverman-inspired), floored at 0.1 to
    /// avoid near-degenerate kernels with very few data points.
    fn fit(observations: &'a [(Vec<f64>, f64)], beta: f64) -> Self {
        let n = observations.len() as f64;
        let d = observations.first().map_or(1, |(x, _)| x.len()) as f64;
        let h = n.powf(-1.0 / (d + 4.0)).max(0.1);
        Self {
            observations,
            beta,
            bandwidth_sq: 2.0 * h * h,
        }
    }

    /// UCB acquisition: `μ(x) + β·σ(x)`.
    fn acquisition(&self, x: &[f64]) -> f64 {
        let (mean, std) = self.predict(x);
        mean + self.beta * std
    }

    /// Nadaraya-Watson mean and weighted standard deviation at `x`.
    ///
    /// Returns `(0.0, 1.0)` — maximum uncertainty — when all observations are
    /// too distant to contribute meaningful kernel weight.
    ///
    /// Uses Chan's single-pass online weighted mean+variance algorithm,
    /// evaluating each RBF weight exactly once (vs. the two-pass approach
    /// that would call `rbf` twice per observation).
    fn predict(&self, x: &[f64]) -> (f64, f64) {
        let mut w_sum = 0.0_f64;
        let mut mean = 0.0_f64;
        let mut s = 0.0_f64; // weighted sum of squared deviations

        for (xi, yi) in self.observations {
            let w = self.rbf(x, xi);
            if w < f64::EPSILON {
                continue;
            }
            let w_new = w_sum + w;
            let delta = yi - mean;
            mean += (w / w_new) * delta;
            s += w * delta * (yi - mean);
            w_sum = w_new;
        }

        if w_sum < f64::EPSILON {
            return (0.0, 1.0);
        }

        let std = (s / w_sum).max(0.0).sqrt();
        (mean, std)
    }

    /// Gaussian (RBF) kernel: `exp(-‖x − xᵢ‖² / (2h²))`.
    #[inline]
    fn rbf(&self, x: &[f64], xi: &[f64]) -> f64 {
        let dist_sq: f64 = x.iter().zip(xi.iter()).map(|(a, b)| (a - b).powi(2)).sum();
        (-dist_sq / self.bandwidth_sq).exp()
    }
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
                volume: 1_000,
                adj_close: Some(p),
            })
            .collect()
    }

    fn trending_prices(n: usize) -> Vec<f64> {
        (0..n).map(|i| 100.0 + i as f64 * 0.5).collect()
    }

    // ── LHS ───────────────────────────────────────────────────────────────────

    #[test]
    fn test_lhs_shape() {
        let mut rng = Xorshift64::new(1);
        let samples = latin_hypercube_sample(8, 3, &mut rng);
        assert_eq!(samples.len(), 8);
        assert!(samples.iter().all(|p| p.len() == 3));
    }

    #[test]
    fn test_lhs_stratification() {
        let n = 10;
        let mut rng = Xorshift64::new(99);
        let samples = latin_hypercube_sample(n, 2, &mut rng);

        for dim in 0..2 {
            let mut counts = vec![0usize; n];
            for point in &samples {
                let stratum = (point[dim] * n as f64).floor() as usize;
                counts[stratum.min(n - 1)] += 1;
            }
            assert!(
                counts.iter().all(|&c| c == 1),
                "dim {dim}: expected one sample per stratum, got {counts:?}"
            );
        }
    }

    #[test]
    fn test_lhs_values_in_unit_cube() {
        let mut rng = Xorshift64::new(7);
        for point in latin_hypercube_sample(20, 4, &mut rng) {
            for v in point {
                assert!(v > 0.0 && v <= 1.0, "value {v} outside (0, 1]");
            }
        }
    }

    // ── Surrogate ─────────────────────────────────────────────────────────────

    #[test]
    fn test_surrogate_predicts_near_observation() {
        let obs = vec![(vec![0.5_f64], 1.0_f64)];
        let s = Surrogate::fit(&obs, 2.0);
        let (mean, _) = s.predict(&[0.5]);
        assert!((mean - 1.0).abs() < 1e-6);
    }

    /// A point so far from all observations that `exp(-dist²/2h²) < ε` triggers
    /// the maximum-uncertainty fallback path, returning `(0.0, 1.0)`.
    #[test]
    fn test_surrogate_max_uncertainty_fallback_for_very_distant_point() {
        // At x=100 the kernel weight is exp(-10000/bandwidth_sq) which underflows
        // to exactly 0.0 in f64, so w_sum < EPSILON and the fallback is taken.
        let obs = vec![(vec![0.0_f64], 0.5_f64), (vec![0.1], 0.6)];
        let s = Surrogate::fit(&obs, 2.0);
        let (mean, std) = s.predict(&[100.0]);
        assert!(
            (mean - 0.0).abs() < 1e-6,
            "expected fallback mean=0.0, got {mean}"
        );
        assert!(
            (std - 1.0).abs() < 1e-6,
            "expected fallback std=1.0, got {std}"
        );
    }

    /// When two nearby observations have very different scores, the surrogate
    /// should report non-trivial variance at the midpoint.
    #[test]
    fn test_surrogate_std_nonzero_with_disagreeing_observations() {
        let obs = vec![(vec![0.0_f64], 0.1_f64), (vec![0.05], 0.9)];
        let s = Surrogate::fit(&obs, 2.0);
        let (_, std) = s.predict(&[0.025]); // midpoint — equal weight to both
        assert!(
            std > 0.1,
            "expected non-trivial std for disagreeing observations, got {std}"
        );
    }

    #[test]
    fn test_acquisition_favours_uncertain_regions_with_high_beta() {
        let obs = vec![(vec![0.0_f64], 0.5_f64), (vec![0.1], 0.6)];
        let s = Surrogate::fit(&obs, 10.0); // high β → exploration-heavy
        assert!(
            s.acquisition(&[1.0]) > s.acquisition(&[0.05]),
            "far point should have higher UCB with β=10"
        );
    }

    // ── BayesianSearch integration ────────────────────────────────────────────

    #[test]
    fn test_bayesian_search_runs() {
        let candles = make_candles(&trending_prices(200));
        let config = BacktestConfig::builder()
            .commission_pct(0.0)
            .slippage_pct(0.0)
            .build()
            .unwrap();

        let report = BayesianSearch::new()
            .param("fast", ParamRange::int_bounds(3, 10))
            .param("slow", ParamRange::int_bounds(10, 30))
            .optimize_for(OptimizeMetric::TotalReturn)
            .max_evaluations(20)
            .seed(1)
            .run("TEST", &candles, &config, |params| {
                SmaCrossover::new(
                    params["fast"].as_int() as usize,
                    params["slow"].as_int() as usize,
                )
            })
            .unwrap();

        assert!(!report.results.is_empty());
        assert_eq!(report.strategy_name, "SMA Crossover");
        assert!(report.n_evaluations > 0);
        assert!(!report.convergence_curve.is_empty());
    }

    #[test]
    fn test_convergence_curve_is_nondecreasing() {
        let candles = make_candles(&trending_prices(200));
        let config = BacktestConfig::builder()
            .commission_pct(0.0)
            .slippage_pct(0.0)
            .build()
            .unwrap();

        let report = BayesianSearch::new()
            .param("fast", ParamRange::int_bounds(3, 15))
            .param("slow", ParamRange::int_bounds(15, 40))
            .max_evaluations(30)
            .seed(2)
            .run("TEST", &candles, &config, |params| {
                SmaCrossover::new(
                    params["fast"].as_int() as usize,
                    params["slow"].as_int() as usize,
                )
            })
            .unwrap();

        for window in report.convergence_curve.windows(2) {
            assert!(
                window[1] >= window[0] - 1e-12,
                "convergence curve not non-decreasing: {window:?}"
            );
        }
    }

    #[test]
    fn test_results_sorted_best_first() {
        let candles = make_candles(&trending_prices(150));
        let config = BacktestConfig::builder()
            .commission_pct(0.0)
            .slippage_pct(0.0)
            .build()
            .unwrap();

        let report = BayesianSearch::new()
            .param("fast", ParamRange::int_bounds(3, 10))
            .param("slow", ParamRange::int_bounds(10, 25))
            .optimize_for(OptimizeMetric::TotalReturn)
            .max_evaluations(15)
            .seed(3)
            .run("TEST", &candles, &config, |params| {
                SmaCrossover::new(
                    params["fast"].as_int() as usize,
                    params["slow"].as_int() as usize,
                )
            })
            .unwrap();

        if report.results.len() > 1 {
            let first = OptimizeMetric::TotalReturn.score(&report.results[0].result);
            let second = OptimizeMetric::TotalReturn.score(&report.results[1].result);
            assert!(first >= second - 1e-12);
        }
    }

    #[test]
    fn test_best_matches_results_first() {
        let candles = make_candles(&trending_prices(150));
        let config = BacktestConfig::builder()
            .commission_pct(0.0)
            .slippage_pct(0.0)
            .build()
            .unwrap();

        let report = BayesianSearch::new()
            .param("fast", ParamRange::int_bounds(3, 10))
            .param("slow", ParamRange::int_bounds(10, 25))
            .max_evaluations(15)
            .seed(4)
            .run("TEST", &candles, &config, |params| {
                SmaCrossover::new(
                    params["fast"].as_int() as usize,
                    params["slow"].as_int() as usize,
                )
            })
            .unwrap();

        let best = OptimizeMetric::SharpeRatio.score(&report.best.result);
        let first = OptimizeMetric::SharpeRatio.score(&report.results[0].result);
        assert!((best - first).abs() < 1e-12);
    }

    #[test]
    fn test_no_params_returns_error() {
        let candles = make_candles(&trending_prices(100));
        let config = BacktestConfig::default();
        assert!(
            BayesianSearch::new()
                .run("TEST", &candles, &config, |_| SmaCrossover::new(5, 20))
                .is_err()
        );
    }

    #[test]
    fn test_seeded_runs_are_reproducible() {
        let candles = make_candles(&trending_prices(200));
        let config = BacktestConfig::builder()
            .commission_pct(0.0)
            .slippage_pct(0.0)
            .build()
            .unwrap();

        let search = BayesianSearch::new()
            .param("fast", ParamRange::int_bounds(3, 12))
            .param("slow", ParamRange::int_bounds(12, 30))
            .max_evaluations(15)
            .seed(77);

        let factory = |p: &HashMap<String, ParamValue>| {
            SmaCrossover::new(p["fast"].as_int() as usize, p["slow"].as_int() as usize)
        };

        let r1 = search
            .clone()
            .run("TEST", &candles, &config, factory)
            .unwrap();
        let r2 = search.run("TEST", &candles, &config, factory).unwrap();

        assert_eq!(r1.n_evaluations, r2.n_evaluations);
        assert_eq!(r1.convergence_curve, r2.convergence_curve);
        assert_eq!(
            r1.best.result.metrics.total_return_pct,
            r2.best.result.metrics.total_return_pct
        );
    }
}
