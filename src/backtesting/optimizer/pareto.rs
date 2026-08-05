//! Multi-objective (Pareto-front) parameter optimisation.
//!
//! Given 2 or more [`OptimizeMetric`] objectives, [`build_pareto_report`]
//! computes the set of non-dominated parameter combinations from a batch of
//! already-evaluated [`OptimizationResult`]s, rather than collapsing them to a
//! single winner. Used by [`GridSearch::run_pareto`](super::GridSearch::run_pareto)
//! and [`BayesianSearch::run_pareto`](super::BayesianSearch::run_pareto).
//!
//! # Example
//!
//! ```ignore
//! use finance_query::backtesting::{
//!     BacktestConfig, SmaCrossover,
//!     optimizer::{GridSearch, OptimizeMetric, ParamRange},
//! };
//!
//! # fn example(candles: &[finance_query::models::chart::Candle]) {
//! let report = GridSearch::new()
//!     .param("fast", ParamRange::int_range(5, 50, 5))
//!     .param("slow", ParamRange::int_range(20, 200, 10))
//!     .run_pareto(
//!         "AAPL",
//!         candles,
//!         &BacktestConfig::default(),
//!         &[OptimizeMetric::SharpeRatio, OptimizeMetric::MinDrawdown],
//!         |params| {
//!             SmaCrossover::new(
//!                 params["fast"].as_int() as usize,
//!                 params["slow"].as_int() as usize,
//!             )
//!         },
//!     )
//!     .unwrap();
//!
//! println!("Non-dominated combinations: {}", report.front.len());
//! # }
//! ```

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use super::super::error::{BacktestError, Result};
use super::super::result::BacktestResult;
use super::{OptimizationResult, OptimizeMetric, ParamValue};

/// One non-dominated solution in a [`ParetoReport::front`].
///
/// A point is *non-dominated* when no other evaluated combination scores at
/// least as well on every objective while scoring strictly better on at
/// least one.
#[non_exhaustive]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParetoPoint {
    /// Parameter values used for this run.
    pub params: HashMap<String, ParamValue>,
    /// The full backtest result for these parameter values.
    pub result: BacktestResult,
    /// Objective scores, aligned with [`ParetoReport::objectives`] (each
    /// already normalised so higher is better, matching [`OptimizeMetric::score`]).
    pub scores: Vec<f64>,
}

/// Multi-objective optimisation report: the Pareto front of non-dominated
/// parameter combinations, rather than a single best-by-one-metric winner.
///
/// # Overfitting Warning
///
/// As with [`OptimizationReport`](super::OptimizationReport), all metrics are
/// **in-sample**. Validate front members on held-out data before trusting them.
#[non_exhaustive]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParetoReport {
    /// Name of the strategy being optimised.
    pub strategy_name: String,
    /// The objectives this front was computed against, in score order.
    pub objectives: Vec<OptimizeMetric>,
    /// Non-dominated parameter combinations, sorted by the first objective's
    /// score descending.
    pub front: Vec<ParetoPoint>,
    /// Total parameter combinations evaluated (before filtering non-finite
    /// scores and computing the front).
    pub total_evaluated: usize,
    /// Combinations excluded from `front` — either dominated by another
    /// combination, or produced a non-finite score for at least one objective.
    pub dominated_count: usize,
}

/// `a` dominates `b` when `a`'s score is `>=` on every objective and strictly
/// `>` on at least one — the standard Pareto-dominance relation for
/// already-maximising scores.
fn dominates(a: &[f64], b: &[f64]) -> bool {
    let mut strictly_better = false;
    for (ai, bi) in a.iter().zip(b.iter()) {
        if ai < bi {
            return false;
        }
        if ai > bi {
            strictly_better = true;
        }
    }
    strictly_better
}

/// Compute the Pareto front over a batch of already-evaluated results.
///
/// `objectives` must contain at least 2 metrics. Each [`OptimizeMetric`]
/// already normalises its score so that higher is better (see
/// [`OptimizeMetric::score`]), so dominance is a straightforward per-objective
/// `>=` comparison — no separate maximise/minimise direction is needed.
///
/// Combinations that produce a non-finite (NaN/infinite) score for *any*
/// objective are excluded from front computation and counted in
/// `dominated_count` alongside genuinely dominated combinations.
///
/// Runs in O(n²) over the evaluated batch — acceptable for the batch sizes
/// `GridSearch`/`BayesianSearch` produce (typically hundreds to a few
/// thousand), but not intended for scaling to arbitrarily large sweeps.
pub(crate) fn build_pareto_report(
    results: Vec<OptimizationResult>,
    objectives: &[OptimizeMetric],
) -> Result<ParetoReport> {
    if objectives.len() < 2 {
        return Err(BacktestError::invalid_param(
            "objectives",
            "Pareto search requires at least 2 objectives",
        ));
    }
    if results.is_empty() {
        return Err(BacktestError::invalid_param(
            "candles",
            "no parameter combination had enough data to run",
        ));
    }

    let total_evaluated = results.len();
    let strategy_name = results[0].result.strategy_name.clone();

    let candidates: Vec<(Vec<f64>, OptimizationResult)> = results
        .into_iter()
        .filter_map(|r| {
            let scores: Vec<f64> = objectives.iter().map(|m| m.score(&r.result)).collect();
            if scores.iter().all(|s| s.is_finite()) {
                Some((scores, r))
            } else {
                None
            }
        })
        .collect();

    if candidates.is_empty() {
        return Err(BacktestError::invalid_param(
            "objectives",
            "every evaluation produced a non-finite score for at least one objective",
        ));
    }

    let n = candidates.len();
    let mut dominated = vec![false; n];
    for i in 0..n {
        if dominated[i] {
            continue;
        }
        for j in 0..n {
            if i == j {
                continue;
            }
            if dominates(&candidates[j].0, &candidates[i].0) {
                dominated[i] = true;
                break;
            }
        }
    }

    let mut front: Vec<ParetoPoint> = candidates
        .into_iter()
        .zip(dominated)
        .filter(|(_, dom)| !*dom)
        .map(|((scores, r), _)| ParetoPoint {
            params: r.params,
            result: r.result,
            scores,
        })
        .collect();

    front.sort_by(|a, b| {
        b.scores[0]
            .partial_cmp(&a.scores[0])
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    let dominated_count = total_evaluated - front.len();

    Ok(ParetoReport {
        strategy_name,
        objectives: objectives.to_vec(),
        front,
        total_evaluated,
        dominated_count,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dominates_basic() {
        assert!(dominates(&[2.0, 2.0], &[1.0, 1.0])); // strictly better on both
        assert!(dominates(&[2.0, 1.0], &[1.0, 1.0])); // equal on one, better on other
        assert!(!dominates(&[1.0, 1.0], &[1.0, 1.0])); // identical: no domination
        assert!(!dominates(&[2.0, 0.0], &[1.0, 1.0])); // better on one, worse on other
    }

    #[test]
    fn test_dominates_requires_strict_improvement() {
        // Equal on every objective: neither dominates the other.
        assert!(!dominates(&[1.0, 2.0, 3.0], &[1.0, 2.0, 3.0]));
    }

    // ── build_pareto_report — synthetic non-domination example ────────────────

    use crate::backtesting::config::BacktestConfig;
    use crate::backtesting::result::{BacktestResult, EquityPoint, PerformanceMetrics};
    use std::collections::HashMap;

    /// Minimal `PerformanceMetrics` with only sharpe/drawdown set.
    fn metrics_with(sharpe_ratio: f64, max_drawdown_pct: f64) -> PerformanceMetrics {
        PerformanceMetrics {
            total_return_pct: 0.0,
            annualized_return_pct: 0.0,
            sharpe_ratio,
            sortino_ratio: 0.0,
            calmar_ratio: 0.0,
            max_drawdown_pct,
            max_drawdown_duration: 0,
            win_rate: 0.0,
            profit_factor: 1.0,
            avg_trade_return_pct: 0.0,
            avg_win_pct: 0.0,
            avg_loss_pct: 0.0,
            avg_trade_duration: 0.0,
            total_trades: 1,
            winning_trades: 1,
            losing_trades: 0,
            largest_win: 0.0,
            largest_loss: 0.0,
            max_consecutive_wins: 0,
            max_consecutive_losses: 0,
            total_commission: 0.0,
            long_trades: 1,
            short_trades: 0,
            total_signals: 1,
            executed_signals: 1,
            avg_win_duration: 0.0,
            avg_loss_duration: 0.0,
            time_in_market_pct: 0.5,
            max_idle_period: 0,
            total_dividend_income: 0.0,
            kelly_criterion: 0.0,
            sqn: 0.0,
            expectancy: 0.0,
            omega_ratio: 1.0,
            tail_ratio: 1.0,
            recovery_factor: 1.0,
            ulcer_index: 0.0,
            serenity_ratio: 0.0,
            total_borrow_cost: 0.0,
        }
    }

    fn make_opt_result(label: &str, sharpe: f64, max_drawdown_pct: f64) -> OptimizationResult {
        let result = BacktestResult {
            symbol: "TEST".to_owned(),
            strategy_name: "test".to_owned(),
            config: BacktestConfig::default(),
            start_timestamp: 0,
            end_timestamp: 1_000_000,
            initial_capital: 10_000.0,
            final_equity: 10_000.0,
            metrics: metrics_with(sharpe, max_drawdown_pct),
            trades: vec![],
            equity_curve: vec![EquityPoint {
                timestamp: 0,
                equity: 10_000.0,
                drawdown_pct: 0.0,
            }],
            signals: vec![],
            open_position: None,
            benchmark: None,
            diagnostics: vec![],
        };
        let mut params = HashMap::new();
        params.insert("label".to_string(), ParamValue::Int(label.len() as i64));
        OptimizationResult { params, result }
    }

    #[test]
    fn test_pareto_front_synthetic_example() {
        // Objectives: maximise Sharpe AND minimise max-drawdown (MinDrawdown
        // negates internally, so higher score = lower drawdown).
        let objectives = [OptimizeMetric::SharpeRatio, OptimizeMetric::MinDrawdown];

        // A: high Sharpe, high drawdown — non-dominated (best on Sharpe)
        // B: low Sharpe, low drawdown  — non-dominated (best on drawdown)
        // C: dominated by both A and B on every axis
        // D: dominated by A (same drawdown as A, worse Sharpe)
        let results = vec![
            make_opt_result("A", 2.0, 0.30),
            make_opt_result("B", 0.5, 0.05),
            make_opt_result("C", 0.4, 0.35),
            make_opt_result("D", 1.0, 0.30),
        ];

        let report = build_pareto_report(results, &objectives).unwrap();

        assert_eq!(report.total_evaluated, 4);
        assert_eq!(
            report.front.len(),
            2,
            "only A and B should be non-dominated"
        );
        assert_eq!(report.dominated_count, 2);

        let front_sharpes: Vec<f64> = report
            .front
            .iter()
            .map(|p| p.result.metrics.sharpe_ratio)
            .collect();
        assert!(
            front_sharpes.contains(&2.0),
            "A (best Sharpe) must be in the front"
        );
        assert!(
            front_sharpes.contains(&0.5),
            "B (best drawdown) must be in the front"
        );
        assert!(
            !front_sharpes.contains(&0.4),
            "C is dominated and must be excluded"
        );
        assert!(
            !front_sharpes.contains(&1.0),
            "D is dominated by A and must be excluded"
        );

        // Sorted by first objective (Sharpe) descending.
        assert_eq!(report.front[0].result.metrics.sharpe_ratio, 2.0);
    }

    #[test]
    fn test_pareto_front_requires_two_objectives() {
        let results = vec![make_opt_result("A", 1.0, 0.1)];
        let err = build_pareto_report(results, &[OptimizeMetric::SharpeRatio]);
        assert!(err.is_err());
    }

    #[test]
    fn test_pareto_front_empty_results_errors() {
        let err = build_pareto_report(
            vec![],
            &[OptimizeMetric::SharpeRatio, OptimizeMetric::MinDrawdown],
        );
        assert!(err.is_err());
    }

    #[test]
    fn test_pareto_front_all_non_finite_scores_errors() {
        // NaN sharpe on every candidate for one objective → no valid candidates.
        let mut r1 = make_opt_result("A", f64::NAN, 0.1);
        r1.result.metrics.sharpe_ratio = f64::NAN;
        let mut r2 = make_opt_result("B", f64::NAN, 0.2);
        r2.result.metrics.sharpe_ratio = f64::NAN;

        let err = build_pareto_report(
            vec![r1, r2],
            &[OptimizeMetric::SharpeRatio, OptimizeMetric::MinDrawdown],
        );
        assert!(err.is_err());
    }
}
