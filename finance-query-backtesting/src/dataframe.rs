//! Polars conversions for portfolio, simulation and search reports.

use std::collections::HashMap;

use polars::prelude::{Column, DataFrame, NamedFrom, PolarsResult, Series};

use crate::comparison::{ComparisonReport, ComparisonRow};
use crate::monte_carlo::MonteCarloResult;
use crate::optimizer::{OptimizationReport, ParamValue, ParetoReport};
use crate::portfolio::PortfolioResult;
use crate::result::dataframe::datetimes;
use crate::result::{EquityPoint, PerformanceMetrics};
use crate::walk_forward::WalkForwardReport;

fn column<U>(name: &str, values: Vec<U>) -> Column
where
    Series: NamedFrom<Vec<U>, [U]>,
{
    Series::new(name.into(), values).into()
}

fn sorted<V>(map: &HashMap<String, V>) -> Vec<(&String, &V)> {
    let mut entries: Vec<_> = map.iter().collect();
    entries.sort_by(|a, b| a.0.cmp(b.0));
    entries
}

/// Stacks each metrics row under `leading` columns, which must hold one value per row.
fn metrics_rows(leading: Vec<Column>, metrics: &[&PerformanceMetrics]) -> PolarsResult<DataFrame> {
    let mut frame = PerformanceMetrics::vec_to_dataframe(&[])?;
    for m in metrics {
        frame = frame.vstack(&m.to_dataframe()?)?;
    }
    for (at, column) in leading.into_iter().enumerate() {
        frame.insert_column(at, column)?;
    }
    frame.rechunk_mut();
    Ok(frame)
}

/// One row per parameter of each keyed set, in key order then parameter name.
fn param_rows<'a>(
    key_column: &str,
    sets: impl Iterator<Item = (u64, &'a HashMap<String, ParamValue>)>,
) -> PolarsResult<DataFrame> {
    let (mut keys, mut params, mut values) = (Vec::new(), Vec::new(), Vec::new());
    for (key, set) in sets {
        for (name, value) in sorted(set) {
            keys.push(key);
            params.push(name.as_str());
            values.push(value.as_float());
        }
    }
    DataFrame::new(
        keys.len(),
        vec![
            column(key_column, keys),
            column("param", params),
            column("value", values),
        ],
    )
}

impl ComparisonReport {
    /// Converts the ranked rows to a polars DataFrame, one row per strategy, best first.
    ///
    /// Columns follow [`ComparisonRow`]: `label`, `strategy_name`, `symbol`, the metric
    /// columns, then `rank_score` (f64) and `rank` (u64, 1-based).
    pub fn to_dataframe(&self) -> PolarsResult<DataFrame> {
        ComparisonRow::vec_to_dataframe(&self.rows)
    }
}

impl MonteCarloResult {
    /// Converts the metric distributions to a polars DataFrame, one row per metric.
    ///
    /// Columns are `metric` (string: `total_return_dist`, `max_drawdown`, `sharpe_ratio`,
    /// `profit_factor`) and `p5`, `p25`, `p50`, `p75`, `p95`, `mean` (f64).
    pub fn to_dataframe(&self) -> PolarsResult<DataFrame> {
        let stats = [
            ("total_return_dist", &self.total_return_dist),
            ("max_drawdown", &self.max_drawdown),
            ("sharpe_ratio", &self.sharpe_ratio),
            ("profit_factor", &self.profit_factor),
        ];
        let stat = |get: fn(&crate::monte_carlo::PercentileStats) -> f64| {
            stats.iter().map(|(_, s)| get(s)).collect::<Vec<_>>()
        };
        DataFrame::new(
            stats.len(),
            vec![
                column("metric", stats.iter().map(|(name, _)| *name).collect()),
                column("p5", stat(|s| s.p5)),
                column("p25", stat(|s| s.p25)),
                column("p50", stat(|s| s.p50)),
                column("p75", stat(|s| s.p75)),
                column("p95", stat(|s| s.p95)),
                column("mean", stat(|s| s.mean)),
            ],
        )
    }
}

impl PortfolioResult {
    /// Converts the per-symbol metrics to a polars DataFrame, one row per symbol.
    ///
    /// Columns are `symbol` (string, sorted ascending) followed by the
    /// [`PerformanceMetrics`] columns. An empty portfolio keeps the schema.
    pub fn to_dataframe(&self) -> PolarsResult<DataFrame> {
        let symbols = sorted(&self.symbols);
        let metrics: Vec<_> = symbols.iter().map(|(_, r)| &r.metrics).collect();
        let names = symbols.iter().map(|(s, _)| s.as_str()).collect();
        metrics_rows(vec![column("symbol", names)], &metrics)
    }

    /// Converts the combined equity curve to a polars DataFrame, one row per bar.
    ///
    /// Columns are `timestamp` (datetime, UTC), `equity` (f64) and `drawdown` (f64, a fraction).
    pub fn equity_dataframe(&self) -> PolarsResult<DataFrame> {
        EquityPoint::vec_to_dataframe(&self.portfolio_equity_curve)
    }

    /// Converts the allocation history to a long-format polars DataFrame.
    ///
    /// Columns are `timestamp` (datetime, UTC), `asset` (string, `cash` or a symbol) and `value`
    /// (f64). Each snapshot emits `cash` first, then its positions sorted by symbol.
    pub fn allocation_dataframe(&self) -> PolarsResult<DataFrame> {
        let (mut timestamps, mut assets, mut values) = (Vec::new(), Vec::new(), Vec::new());
        for snapshot in &self.allocation_history {
            timestamps.push(snapshot.timestamp);
            assets.push("cash");
            values.push(snapshot.cash);
            for (symbol, value) in sorted(&snapshot.positions) {
                timestamps.push(snapshot.timestamp);
                assets.push(symbol.as_str());
                values.push(*value);
            }
        }
        DataFrame::new(
            values.len(),
            vec![
                datetimes("timestamp", &timestamps, |timestamp| *timestamp),
                column("asset", assets),
                column("value", values),
            ],
        )
    }
}

impl OptimizationReport {
    /// Converts the results to a polars DataFrame, one row per parameter set, best first.
    ///
    /// Columns are `rank` (u64, 1-based) followed by the [`PerformanceMetrics`] columns.
    pub fn to_dataframe(&self) -> PolarsResult<DataFrame> {
        let metrics: Vec<_> = self.results.iter().map(|r| &r.result.metrics).collect();
        let ranks = (1..=metrics.len() as u64).collect();
        metrics_rows(vec![column("rank", ranks)], &metrics)
    }

    /// Converts the parameter sets to a long-format polars DataFrame.
    ///
    /// Columns are `rank` (u64, matching [`to_dataframe`](Self::to_dataframe)), `param`
    /// (string) and `value` (f64). Each rank lists its parameters sorted by name.
    pub fn params_dataframe(&self) -> PolarsResult<DataFrame> {
        param_rows(
            "rank",
            self.results
                .iter()
                .zip(1u64..)
                .map(|(r, rank)| (rank, &r.params)),
        )
    }

    /// Converts the convergence trace to a polars DataFrame, one row per evaluation.
    ///
    /// Columns are `evaluation` (u64, 1-based) and `best_metric` (f64). Empty for grid search.
    pub fn convergence_dataframe(&self) -> PolarsResult<DataFrame> {
        let n = self.convergence_curve.len();
        DataFrame::new(
            n,
            vec![
                column("evaluation", (1..=n as u64).collect()),
                column("best_metric", self.convergence_curve.clone()),
            ],
        )
    }
}

impl WalkForwardReport {
    /// Converts the window results to a polars DataFrame, two rows per window.
    ///
    /// Columns are `window` (u64), `sample` (string, `in_sample` then `out_of_sample`)
    /// followed by the [`PerformanceMetrics`] columns.
    pub fn to_dataframe(&self) -> PolarsResult<DataFrame> {
        let mut windows = Vec::new();
        let mut samples = Vec::new();
        let mut metrics = Vec::new();
        for w in &self.windows {
            for (sample, result) in [
                ("in_sample", &w.in_sample),
                ("out_of_sample", &w.out_of_sample),
            ] {
                windows.push(w.window as u64);
                samples.push(sample);
                metrics.push(&result.metrics);
            }
        }
        metrics_rows(
            vec![column("window", windows), column("sample", samples)],
            &metrics,
        )
    }

    /// Converts the optimised parameters to a long-format polars DataFrame.
    ///
    /// Columns are `window` (u64), `param` (string) and `value` (f64), parameters sorted by name.
    pub fn params_dataframe(&self) -> PolarsResult<DataFrame> {
        param_rows(
            "window",
            self.windows
                .iter()
                .map(|w| (w.window as u64, &w.optimized_params)),
        )
    }
}

impl ParetoReport {
    /// Converts the front's scores to a long-format polars DataFrame.
    ///
    /// Columns are `point` (u64, 1-based position in the front), `objective` (string, the
    /// `OptimizeMetric` variant name) and `score` (f64), one row per point and objective.
    /// Scores are the optimizer's maximised values, so `MinDrawdown` is the negated drawdown.
    pub fn to_dataframe(&self) -> PolarsResult<DataFrame> {
        let (mut points, mut objectives, mut scores) = (Vec::new(), Vec::new(), Vec::new());
        for (point, p) in (1u64..).zip(&self.front) {
            for (objective, score) in self.objectives.iter().zip(&p.scores) {
                points.push(point);
                objectives.push(format!("{objective:?}"));
                scores.push(*score);
            }
        }
        DataFrame::new(
            scores.len(),
            vec![
                column("point", points),
                column("objective", objectives),
                column("score", scores),
            ],
        )
    }

    /// Converts the front's parameter sets to a long-format polars DataFrame.
    ///
    /// Columns are `point` (u64, matching [`to_dataframe`](Self::to_dataframe)), `param`
    /// (string) and `value` (f64), parameters sorted by name.
    pub fn params_dataframe(&self) -> PolarsResult<DataFrame> {
        param_rows(
            "point",
            self.front
                .iter()
                .zip(1u64..)
                .map(|(p, point)| (point, &p.params)),
        )
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use polars::prelude::DataType;

    use super::*;
    use crate::monte_carlo::{MonteCarloMethod, PercentileStats};
    use crate::optimizer::{OptimizationResult, OptimizeMetric, ParetoPoint};
    use crate::portfolio::AllocationSnapshot;
    use crate::result::fixtures::{equity_point, make_result, make_trade};
    use crate::walk_forward::WindowResult;

    fn names(df: &DataFrame) -> Vec<String> {
        df.get_column_names()
            .iter()
            .map(|n| n.to_string())
            .collect()
    }

    fn dtypes(df: &DataFrame) -> Vec<DataType> {
        df.dtypes()
    }

    fn params(pairs: &[(&str, ParamValue)]) -> HashMap<String, ParamValue> {
        pairs
            .iter()
            .map(|(k, v)| (k.to_string(), v.clone()))
            .collect()
    }

    fn result() -> crate::BacktestResult {
        make_result(
            vec![make_trade(50.0, 5.0, true)],
            vec![equity_point(1, 100.0, 0.0)],
        )
    }

    fn stats(base: f64) -> PercentileStats {
        PercentileStats {
            p5: base,
            p25: base + 1.0,
            p50: base + 2.0,
            p75: base + 3.0,
            p95: base + 4.0,
            mean: base + 2.0,
        }
    }

    fn optimization() -> OptimizationReport {
        let best = OptimizationResult {
            params: params(&[
                ("slow", ParamValue::Int(50)),
                ("fast", ParamValue::Float(0.5)),
            ]),
            result: result(),
        };
        OptimizationReport {
            strategy_name: "S".into(),
            total_combinations: 2,
            results: vec![best.clone(), best.clone()],
            best,
            skipped_errors: 0,
            convergence_curve: vec![1.0, 2.0],
            n_evaluations: 2,
        }
    }

    #[test]
    fn portfolio_frames() {
        let report = PortfolioResult {
            symbols: HashMap::from([("B".to_string(), result()), ("A".to_string(), result())]),
            portfolio_equity_curve: vec![equity_point(1, 100.0, 0.0), equity_point(2, 90.0, 0.1)],
            portfolio_metrics: result().metrics,
            initial_capital: 100.0,
            final_equity: 90.0,
            allocation_history: vec![AllocationSnapshot {
                timestamp: 1,
                cash: 10.0,
                positions: HashMap::from([("B".to_string(), 2.0), ("A".to_string(), 1.0)]),
            }],
        };

        let df = report.to_dataframe().unwrap();
        assert_eq!(df.height(), 2);
        assert_eq!(df.width(), 40);
        assert_eq!(names(&df)[..2], ["symbol", "total_return"]);
        assert_eq!(
            df.column("symbol").unwrap().str().unwrap().get(0),
            Some("A")
        );
        assert_eq!(report.equity_dataframe().unwrap().height(), 2);

        let allocation = report.allocation_dataframe().unwrap();
        assert_eq!(names(&allocation), ["timestamp", "asset", "value"]);
        assert_eq!(
            dtypes(&allocation),
            [
                finance_query_types::__dataframe::datetime_dtype(),
                DataType::String,
                DataType::Float64
            ]
        );
        let micros = allocation
            .column("timestamp")
            .unwrap()
            .cast(&DataType::Int64)
            .unwrap();
        assert_eq!(micros.i64().unwrap().get(0), Some(1_000_000));
        let assets: Vec<_> = allocation
            .column("asset")
            .unwrap()
            .str()
            .unwrap()
            .into_no_null_iter()
            .collect();
        assert_eq!(assets, ["cash", "A", "B"]);
    }

    #[test]
    fn empty_portfolio_keeps_schema() {
        let mut report = PortfolioResult {
            symbols: HashMap::new(),
            portfolio_equity_curve: vec![],
            portfolio_metrics: result().metrics,
            initial_capital: 0.0,
            final_equity: 0.0,
            allocation_history: vec![],
        };
        report.symbols.insert("A".into(), result());
        let filled = report.to_dataframe().unwrap();
        report.symbols.clear();
        let empty = report.to_dataframe().unwrap();
        assert_eq!(empty.height(), 0);
        assert_eq!(empty.schema(), filled.schema());
        assert_eq!(report.allocation_dataframe().unwrap().width(), 3);
        assert_eq!(report.equity_dataframe().unwrap().width(), 3);
    }

    #[test]
    fn monte_carlo_frame_has_one_row_per_metric() {
        let report = MonteCarloResult {
            num_simulations: 10,
            method: MonteCarloMethod::default(),
            total_return_dist: stats(1.0),
            max_drawdown: stats(2.0),
            sharpe_ratio: stats(3.0),
            profit_factor: stats(4.0),
        };
        let df = report.to_dataframe().unwrap();

        assert_eq!(
            names(&df),
            ["metric", "p5", "p25", "p50", "p75", "p95", "mean"]
        );
        assert_eq!(dtypes(&df)[0], DataType::String);
        assert_eq!(dtypes(&df)[1], DataType::Float64);
        assert_eq!(df.height(), 4);
        assert_eq!(df.column("p50").unwrap().f64().unwrap().get(2), Some(5.0));
    }

    #[test]
    fn optimization_frames() {
        let report = optimization();

        let df = report.to_dataframe().unwrap();
        assert_eq!(df.height(), 2);
        assert_eq!(df.width(), 40);
        assert_eq!(df.column("rank").unwrap().dtype(), &DataType::UInt64);

        let params = report.params_dataframe().unwrap();
        assert_eq!(names(&params), ["rank", "param", "value"]);
        assert_eq!(
            dtypes(&params),
            [DataType::UInt64, DataType::String, DataType::Float64]
        );
        assert_eq!(params.height(), 4);
        assert_eq!(
            params.column("param").unwrap().str().unwrap().get(0),
            Some("fast")
        );

        let curve = report.convergence_dataframe().unwrap();
        assert_eq!(names(&curve), ["evaluation", "best_metric"]);
        assert_eq!(curve.height(), 2);
    }

    #[test]
    fn empty_optimization_keeps_schema() {
        let mut report = optimization();
        let filled = report.to_dataframe().unwrap();
        report.results.clear();
        report.convergence_curve.clear();

        let empty = report.to_dataframe().unwrap();
        assert_eq!(empty.height(), 0);
        assert_eq!(empty.schema(), filled.schema());
        assert_eq!(report.params_dataframe().unwrap().width(), 3);
        assert_eq!(report.convergence_dataframe().unwrap().width(), 2);
    }

    #[test]
    fn walk_forward_frames() {
        let window = |n| WindowResult {
            window: n,
            optimized_params: params(&[("fast", ParamValue::Int(5))]),
            in_sample: result(),
            out_of_sample: result(),
        };
        let mut report = WalkForwardReport {
            strategy_name: "S".into(),
            windows: vec![window(0), window(1)],
            aggregate_metrics: result().metrics,
            consistency_ratio: 1.0,
            optimization_reports: vec![],
        };

        let df = report.to_dataframe().unwrap();
        assert_eq!(df.height(), 4);
        assert_eq!(names(&df)[..3], ["window", "sample", "total_return"]);
        let samples: Vec<_> = df
            .column("sample")
            .unwrap()
            .str()
            .unwrap()
            .into_no_null_iter()
            .collect();
        assert_eq!(
            samples,
            ["in_sample", "out_of_sample", "in_sample", "out_of_sample"]
        );

        let params = report.params_dataframe().unwrap();
        assert_eq!(names(&params), ["window", "param", "value"]);
        assert_eq!(params.height(), 2);

        report.windows.clear();
        assert_eq!(report.to_dataframe().unwrap().schema(), df.schema());
        assert_eq!(report.params_dataframe().unwrap().height(), 0);
    }

    #[test]
    fn pareto_frames() {
        let point = |score| ParetoPoint {
            params: params(&[("fast", ParamValue::Int(5))]),
            result: result(),
            scores: vec![score, score + 1.0],
        };
        let mut report = ParetoReport {
            strategy_name: "S".into(),
            objectives: vec![OptimizeMetric::TotalReturn, OptimizeMetric::SharpeRatio],
            front: vec![point(1.0), point(2.0)],
            total_evaluated: 2,
            dominated_count: 0,
            non_finite_count: 0,
        };

        let df = report.to_dataframe().unwrap();
        assert_eq!(names(&df), ["point", "objective", "score"]);
        assert_eq!(
            dtypes(&df),
            [DataType::UInt64, DataType::String, DataType::Float64]
        );
        assert_eq!(df.height(), 4);
        assert_eq!(
            df.column("objective").unwrap().str().unwrap().get(1),
            Some("SharpeRatio")
        );
        assert_eq!(report.params_dataframe().unwrap().height(), 2);

        report.front.clear();
        assert_eq!(report.to_dataframe().unwrap().schema(), df.schema());
    }

    #[test]
    fn comparison_frame_keeps_schema_when_empty() {
        let report = crate::comparison::BacktestComparison::new()
            .add("a", result())
            .ranked_by(OptimizeMetric::TotalReturn);
        let df = report.to_dataframe().unwrap();
        assert_eq!(df.height(), 1);
        assert_eq!(names(&df)[..3], ["label", "strategy_name", "symbol"]);
        assert_eq!(df.column("rank").unwrap().dtype(), &DataType::UInt64);

        let empty = crate::comparison::BacktestComparison::new()
            .ranked_by(OptimizeMetric::TotalReturn)
            .to_dataframe()
            .unwrap();
        assert_eq!(empty.height(), 0);
        assert_eq!(empty.schema(), df.schema());
    }
}
