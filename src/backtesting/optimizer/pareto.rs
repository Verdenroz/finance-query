//! Multi-objective parameter search over completed backtests.
//!
//! [`GridSearch::run_pareto`] and [`BayesianSearch::run_pareto`] score finished
//! evaluations against two or more objectives and return the non-dominated set
//! as a [`ParetoReport`].

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::backtesting::config::BacktestConfig;
use crate::backtesting::error::{BacktestError, Result};
use crate::backtesting::result::BacktestResult;
use crate::backtesting::strategy::Strategy;
use crate::models::chart::Candle;

use super::{BayesianSearch, GridSearch, OptimizationResult, OptimizeMetric, ParamValue};

/// One non-dominated parameter set and its score on each objective.
#[non_exhaustive]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParetoPoint {
    /// Parameter set that produced this result.
    pub params: HashMap<String, ParamValue>,
    /// The completed backtest.
    pub result: BacktestResult,
    /// Score per objective, in the order the objectives were requested. Higher
    /// is better on every entry, matching [`OptimizeMetric`].
    pub scores: Vec<f64>,
}

/// The Pareto front of a multi-objective search.
///
/// `total_evaluated == front.len() + dominated_count + non_finite_count`.
#[non_exhaustive]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParetoReport {
    /// Name of the strategy under test.
    pub strategy_name: String,
    /// Objectives that were optimised, in request order.
    pub objectives: Vec<OptimizeMetric>,
    /// Non-dominated parameter sets, sorted best-first on the first objective.
    pub front: Vec<ParetoPoint>,
    /// Parameter sets that produced a completed backtest.
    pub total_evaluated: usize,
    /// Candidates beaten outright by another candidate.
    pub dominated_count: usize,
    /// Candidates excluded for scoring non-finite on at least one objective.
    pub non_finite_count: usize,
}

/// True when `a` is at least as good as `b` everywhere and strictly better
/// somewhere. Mismatched lengths never dominate.
fn dominates(a: &[f64], b: &[f64]) -> bool {
    if a.len() != b.len() {
        return false;
    }

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

pub(super) fn validate_objectives(objectives: &[OptimizeMetric]) -> Result<()> {
    if objectives.len() < 2 {
        return Err(BacktestError::invalid_param(
            "objectives",
            "Pareto search requires at least 2 objectives",
        ));
    }
    Ok(())
}

/// Reduce completed evaluations to their Pareto front.
pub(super) fn build_pareto_report(
    results: Vec<OptimizationResult>,
    objectives: &[OptimizeMetric],
) -> Result<ParetoReport> {
    validate_objectives(objectives)?;

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
            scores.iter().all(|s| s.is_finite()).then_some((scores, r))
        })
        .collect();

    if candidates.is_empty() {
        return Err(BacktestError::invalid_param(
            "objectives",
            "every evaluation produced a non-finite score on at least one objective",
        ));
    }

    let non_finite_count = total_evaluated - candidates.len();

    let dominated: Vec<bool> = (0..candidates.len())
        .map(|i| {
            candidates
                .iter()
                .enumerate()
                .any(|(j, other)| j != i && dominates(&other.0, &candidates[i].0))
        })
        .collect();

    let mut front: Vec<ParetoPoint> = candidates
        .into_iter()
        .zip(dominated)
        .filter(|(_, is_dominated)| !is_dominated)
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

    let dominated_count = total_evaluated - non_finite_count - front.len();

    Ok(ParetoReport {
        strategy_name,
        objectives: objectives.to_vec(),
        front,
        total_evaluated,
        dominated_count,
        non_finite_count,
    })
}

impl GridSearch {
    /// Run the full grid and return the Pareto front over `objectives`.
    ///
    /// Requires at least two objectives; [`run`](Self::run) covers the single
    /// metric case.
    pub fn run_pareto<S, F>(
        &self,
        symbol: &str,
        candles: &[Candle],
        config: &BacktestConfig,
        objectives: &[OptimizeMetric],
        factory: F,
    ) -> Result<ParetoReport>
    where
        S: Strategy + Send,
        F: Fn(&HashMap<String, ParamValue>) -> S + Send + Sync,
    {
        validate_objectives(objectives)?;
        let (results, _, _) = self.evaluate_all(symbol, candles, config, factory)?;
        build_pareto_report(results, objectives)
    }
}

impl BayesianSearch {
    /// Run the surrogate search and return the Pareto front over `objectives`.
    ///
    /// The search itself is guided by `objectives[0]`; the remaining objectives
    /// filter the completed evaluations. Requires at least two objectives.
    pub fn run_pareto<S, F>(
        &self,
        symbol: &str,
        candles: &[Candle],
        config: &BacktestConfig,
        objectives: &[OptimizeMetric],
        factory: F,
    ) -> Result<ParetoReport>
    where
        S: Strategy,
        F: Fn(&HashMap<String, ParamValue>) -> S,
    {
        validate_objectives(objectives)?;
        let (results, _, _) = self.search(symbol, candles, config, objectives[0], &factory)?;
        build_pareto_report(results, objectives)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::backtesting::result::PerformanceMetrics;
    use crate::backtesting::{ParamRange, SmaCrossover};

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
                provider_id: None,
            })
            .collect()
    }

    fn trending_prices(n: usize) -> Vec<f64> {
        (0..n).map(|i| 100.0 + i as f64 * 0.5).collect()
    }

    fn opt_result(label: &str, sharpe: f64, max_drawdown_pct: f64) -> OptimizationResult {
        let mut result = BacktestResult {
            symbol: "TEST".to_string(),
            strategy_name: "Synthetic".to_string(),
            config: BacktestConfig::default(),
            start_timestamp: 0,
            end_timestamp: 1,
            initial_capital: 10_000.0,
            final_equity: 10_000.0,
            metrics: PerformanceMetrics::calculate(&[], &[], 10_000.0, 0, 0, 0.0, 252.0),
            trades: vec![],
            equity_curve: vec![],
            signals: vec![],
            open_position: None,
            benchmark: None,
            diagnostics: vec![],
        };
        result.metrics.sharpe_ratio = sharpe;
        result.metrics.max_drawdown_pct = max_drawdown_pct;

        let mut params = HashMap::new();
        params.insert("label".to_string(), ParamValue::Int(label.len() as i64));
        OptimizationResult { params, result }
    }

    const TWO: [OptimizeMetric; 2] = [OptimizeMetric::SharpeRatio, OptimizeMetric::MinDrawdown];

    #[test]
    fn test_dominates_requires_no_worse_and_one_better() {
        assert!(dominates(&[2.0, 1.0], &[1.0, 1.0]));
        assert!(dominates(&[2.0, 2.0], &[1.0, 1.0]));
        assert!(!dominates(&[2.0, 0.5], &[1.0, 1.0]));
        assert!(!dominates(&[1.0, 1.0], &[1.0, 1.0]));
    }

    #[test]
    fn test_dominates_is_false_on_length_mismatch() {
        assert!(!dominates(&[1.0, 2.0], &[1.0]));
        assert!(!dominates(&[1.0], &[1.0, 2.0]));
    }

    #[test]
    fn test_front_keeps_only_non_dominated_points() {
        let results = vec![
            opt_result("a", 2.0, 0.30),
            opt_result("b", 0.5, 0.05),
            opt_result("c", 0.4, 0.35),
            opt_result("d", 1.0, 0.30),
        ];
        let report = build_pareto_report(results, &TWO).unwrap();

        assert_eq!(report.front.len(), 2);
        assert_eq!(report.dominated_count, 2);
        assert_eq!(report.non_finite_count, 0);
        assert!((report.front[0].scores[0] - 2.0).abs() < 1e-12);
    }

    #[test]
    fn test_counts_partition_every_evaluation() {
        let mut results = vec![
            opt_result("a", 2.0, 0.30),
            opt_result("b", 0.5, 0.05),
            opt_result("c", 0.4, 0.35),
        ];
        results.push(opt_result("nan", f64::NAN, 0.10));
        let report = build_pareto_report(results, &TWO).unwrap();

        assert_eq!(report.total_evaluated, 4);
        assert_eq!(report.non_finite_count, 1);
        assert_eq!(
            report.total_evaluated,
            report.front.len() + report.dominated_count + report.non_finite_count
        );
    }

    #[test]
    fn test_fewer_than_two_objectives_is_rejected() {
        let results = vec![opt_result("a", 1.0, 0.1)];
        assert!(build_pareto_report(results.clone(), &[]).is_err());
        assert!(build_pareto_report(results, &[OptimizeMetric::SharpeRatio]).is_err());
    }

    #[test]
    fn test_empty_results_are_rejected() {
        assert!(build_pareto_report(vec![], &TWO).is_err());
    }

    #[test]
    fn test_all_non_finite_scores_are_rejected() {
        let results = vec![
            opt_result("a", f64::NAN, 0.1),
            opt_result("b", f64::NAN, 0.2),
        ];
        assert!(build_pareto_report(results, &TWO).is_err());
    }

    #[test]
    fn test_grid_run_pareto_returns_a_non_dominated_front() {
        let candles = make_candles(&trending_prices(120));
        let report = GridSearch::new()
            .param("fast", ParamRange::int_range(3, 9, 3))
            .param("slow", ParamRange::int_range(12, 24, 6))
            .run_pareto("TEST", &candles, &BacktestConfig::default(), &TWO, |p| {
                SmaCrossover::new(p["fast"].as_int() as usize, p["slow"].as_int() as usize)
            })
            .unwrap();

        assert!(!report.front.is_empty());
        assert_eq!(report.objectives, TWO.to_vec());
        for point in &report.front {
            assert_eq!(point.scores.len(), 2);
        }
    }

    #[test]
    fn test_bayesian_run_pareto_returns_a_non_dominated_front() {
        let candles = make_candles(&trending_prices(120));
        let report = BayesianSearch::new()
            .param("fast", ParamRange::int_bounds(3, 10))
            .param("slow", ParamRange::int_bounds(12, 30))
            .max_evaluations(12)
            .initial_points(4)
            .seed(42)
            .run_pareto("TEST", &candles, &BacktestConfig::default(), &TWO, |p| {
                SmaCrossover::new(p["fast"].as_int() as usize, p["slow"].as_int() as usize)
            })
            .unwrap();

        assert!(!report.front.is_empty());
        for point in &report.front {
            assert!(point.scores.iter().all(|s| s.is_finite()));
        }
    }

    #[test]
    fn test_objectives_are_validated_before_the_search_runs() {
        let candles = make_candles(&trending_prices(60));
        let err = GridSearch::new()
            .run_pareto(
                "TEST",
                &candles,
                &BacktestConfig::default(),
                &[OptimizeMetric::SharpeRatio],
                |_| SmaCrossover::new(3, 12),
            )
            .unwrap_err();
        assert!(err.to_string().contains("objectives"));

        let err = BayesianSearch::new()
            .run_pareto(
                "TEST",
                &candles,
                &BacktestConfig::default(),
                &[OptimizeMetric::SharpeRatio],
                |_| SmaCrossover::new(3, 12),
            )
            .unwrap_err();
        assert!(err.to_string().contains("objectives"));
    }
}
