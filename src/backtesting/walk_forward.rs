//! Walk-forward parameter optimisation for backtesting strategies.
//!
//! Walk-forward testing prevents overfitting by splitting historical data into
//! rolling in-sample (training) and out-of-sample (test) windows. For each
//! window, the best parameters are discovered on the in-sample slice via grid
//! search, then validated on the subsequent out-of-sample slice.
//!
//! # How it works
//!
//! ```text
//! |--- in-sample (IS) ---|--- out-of-sample (OOS) ---|
//!            |-- step --|--- IS ---|--- OOS ---|
//!                                  |-- step --|--- IS ---|--- OOS ---|
//! ```
//!
//! Aggregate metrics from all OOS windows provide an unbiased estimate of
//! real-world strategy performance.
//!
//! # Example
//!
//! ```ignore
//! use finance_query::backtesting::{
//!     BacktestConfig, SmaCrossover,
//!     optimizer::{GridSearch, OptimizeMetric, ParamRange},
//!     walk_forward::WalkForwardConfig,
//! };
//!
//! # fn example(candles: &[finance_query::models::chart::Candle]) {
//! let grid = GridSearch::new()
//!     .param("fast", ParamRange::int_range(5, 30, 5))
//!     .param("slow", ParamRange::int_range(20, 100, 10))
//!     .optimize_for(OptimizeMetric::SharpeRatio);
//!
//! let wf = WalkForwardConfig::new(grid, BacktestConfig::default())
//!     .in_sample_bars(252)
//!     .out_of_sample_bars(63);
//!
//! let report = wf
//!     .run("AAPL", candles, |params| SmaCrossover::new(
//!         params["fast"].as_int() as usize,
//!         params["slow"].as_int() as usize,
//!     ))
//!     .unwrap();
//!
//! println!("OOS consistency: {:.1}%", report.consistency_ratio * 100.0);
//! # }
//! ```

use std::collections::HashMap;

use rayon::prelude::*;
use serde::{Deserialize, Serialize};

use crate::models::chart::Candle;

use super::config::BacktestConfig;
use super::error::{BacktestError, Result};
use super::optimizer::{GridSearch, OptimizationReport, ParamValue};
use super::result::{BacktestResult, PerformanceMetrics};
use super::strategy::Strategy;

// ── Result types ─────────────────────────────────────────────────────────────

/// Backtest results for a single walk-forward window pair.
#[non_exhaustive]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WindowResult {
    /// Zero-based window index
    pub window: usize,
    /// Parameter values selected as best on the in-sample data
    pub optimized_params: HashMap<String, ParamValue>,
    /// In-sample backtest result (using the best parameters)
    pub in_sample: BacktestResult,
    /// Out-of-sample backtest result (using the same best parameters)
    pub out_of_sample: BacktestResult,
}

/// Aggregate walk-forward report across all windows.
#[non_exhaustive]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WalkForwardReport {
    /// Strategy name
    pub strategy_name: String,
    /// Per-window results
    pub windows: Vec<WindowResult>,
    /// Aggregate performance metrics computed from the concatenated OOS equity curves
    pub aggregate_metrics: PerformanceMetrics,
    /// Fraction of OOS windows that were profitable (0.0 – 1.0)
    pub consistency_ratio: f64,
    /// Full grid-search optimisation reports, one per window
    pub optimization_reports: Vec<OptimizationReport>,
}

// ── WalkForwardConfig ─────────────────────────────────────────────────────────

/// Configuration for a walk-forward parameter optimisation test.
///
/// Build with [`WalkForwardConfig::new`], configure window sizes with the
/// builder methods, then call [`WalkForwardConfig::run`].
#[non_exhaustive]
#[derive(Debug, Clone)]
pub struct WalkForwardConfig {
    /// Grid search to use for optimising in-sample windows
    pub grid: GridSearch,
    /// Base backtest configuration (capital, commission, slippage, …)
    pub config: BacktestConfig,
    /// Number of bars in each in-sample (training) window
    pub in_sample_bars: usize,
    /// Number of bars in each out-of-sample (test) window
    pub out_of_sample_bars: usize,
    /// Number of bars to advance the window each step.
    ///
    /// Defaults to `out_of_sample_bars` (non-overlapping OOS windows).
    pub step_bars: Option<usize>,
}

impl WalkForwardConfig {
    /// Create a new walk-forward config.
    ///
    /// Defaults: `in_sample_bars = 252`, `out_of_sample_bars = 63`, `step_bars = None`.
    pub fn new(grid: GridSearch, config: BacktestConfig) -> Self {
        Self {
            grid,
            config,
            in_sample_bars: 252,
            out_of_sample_bars: 63,
            step_bars: None,
        }
    }

    /// Set the number of bars for each in-sample (training) window.
    pub fn in_sample_bars(mut self, bars: usize) -> Self {
        self.in_sample_bars = bars;
        self
    }

    /// Set the number of bars for each out-of-sample (test) window.
    pub fn out_of_sample_bars(mut self, bars: usize) -> Self {
        self.out_of_sample_bars = bars;
        self
    }

    /// Set the step size (bars to advance between windows).
    ///
    /// Defaults to `out_of_sample_bars` for non-overlapping OOS windows.
    pub fn step_bars(mut self, bars: usize) -> Self {
        self.step_bars = Some(bars);
        self
    }

    /// Run the walk-forward test.
    ///
    /// `symbol` is used only for labelling. `factory` receives the parameter
    /// map selected by each in-sample optimisation and must return a fresh
    /// strategy instance.
    ///
    /// Returns an error if there is not enough data for at least one complete
    /// window pair, or if the grid search or the out-of-sample simulation
    /// fails on any window (fail-fast — a partial result is never returned).
    pub fn run<S, F>(
        &self,
        symbol: &str,
        candles: &[Candle],
        factory: F,
    ) -> Result<WalkForwardReport>
    where
        S: Strategy + Clone + Send,
        F: Fn(&HashMap<String, ParamValue>) -> S,
        F: Send + Sync,
    {
        self.validate(candles.len())?;
        // Checked once for the whole series; every window is a slice of it.
        crate::backtesting::engine::validate_series_order(candles, &[])?;

        let step = self.step_bars.unwrap_or(self.out_of_sample_bars);
        let total_bars = self.in_sample_bars + self.out_of_sample_bars;

        // Slide the window through the candle series
        let starts: Vec<usize> = {
            let mut v = Vec::new();
            let mut start = 0usize;
            while start + total_bars <= candles.len() {
                v.push(start);
                start += step;
            }
            v
        };

        let mut windows: Vec<WindowResult> = Vec::with_capacity(starts.len());
        let mut opt_reports: Vec<OptimizationReport> = Vec::with_capacity(starts.len());

        // Collect every result rather than short-circuiting: rayon does not
        // define which error wins a fallible collect, and callers rely on the
        // lowest-index window's failure being the one reported.
        let results: Vec<Result<(WindowResult, OptimizationReport)>> = starts
            .par_iter()
            .enumerate()
            .map(|(idx, &start)| self.run_one_window(idx, start, symbol, candles, &factory))
            .collect();

        for r in results {
            let (w, o) = r?;
            windows.push(w);
            opt_reports.push(o);
        }

        let strategy_name = windows[0].in_sample.strategy_name.clone();
        let consistency_ratio = calculate_consistency_ratio(&windows);
        let aggregate_metrics = aggregate_oos_metrics(
            &windows,
            self.config.risk_free_rate,
            self.config.bars_per_year,
        );

        Ok(WalkForwardReport {
            strategy_name,
            windows,
            aggregate_metrics,
            consistency_ratio,
            optimization_reports: opt_reports,
        })
    }

    /// Run the optimisation and out-of-sample test for a single window.
    fn run_one_window<S, F>(
        &self,
        window_idx: usize,
        start: usize,
        symbol: &str,
        candles: &[Candle],
        factory: &F,
    ) -> Result<(WindowResult, OptimizationReport)>
    where
        S: Strategy + Clone + Send,
        F: Fn(&HashMap<String, ParamValue>) -> S,
        F: Send + Sync,
    {
        let is_end = start + self.in_sample_bars;
        let oos_end = is_end + self.out_of_sample_bars;

        let is_candles = &candles[start..is_end];
        let oos_candles = &candles[is_end..oos_end];

        // Optimise on the in-sample slice
        let opt_report = self
            .grid
            .run(symbol, is_candles, &self.config, factory)
            .map_err(|e| {
                BacktestError::invalid_param(
                    "walk_forward",
                    format!("window {window_idx} optimisation failed: {e}"),
                )
            })?;

        let best_params = opt_report.best.params.clone();
        let is_result = opt_report.best.result.clone();

        // Test on the out-of-sample slice using the best parameters
        let oos_strategy = factory(&best_params);
        let oos_result = crate::backtesting::BacktestEngine::new(self.config.clone())
            .simulate(symbol, oos_candles, oos_strategy, &[])
            .map_err(|e| {
                BacktestError::invalid_param(
                    "walk_forward",
                    format!("window {window_idx} OOS run failed: {e}"),
                )
            })?;

        Ok((
            WindowResult {
                window: window_idx,
                optimized_params: best_params,
                in_sample: is_result,
                out_of_sample: oos_result,
            },
            opt_report,
        ))
    }

    /// Validate the configuration before running.
    fn validate(&self, num_candles: usize) -> Result<()> {
        if self.in_sample_bars == 0 {
            return Err(BacktestError::invalid_param(
                "in_sample_bars",
                "must be greater than zero",
            ));
        }
        if self.out_of_sample_bars == 0 {
            return Err(BacktestError::invalid_param(
                "out_of_sample_bars",
                "must be greater than zero",
            ));
        }
        if self.step_bars == Some(0) {
            return Err(BacktestError::invalid_param(
                "step_bars",
                "must be greater than zero",
            ));
        }
        let total_bars = self.in_sample_bars + self.out_of_sample_bars;
        if num_candles < total_bars {
            return Err(BacktestError::insufficient_data(total_bars, num_candles));
        }
        Ok(())
    }
}

// ── Internal helpers ──────────────────────────────────────────────────────────

/// Fraction of OOS windows that had a positive total P&L.
fn calculate_consistency_ratio(windows: &[WindowResult]) -> f64 {
    if windows.is_empty() {
        return 0.0;
    }
    let profitable = windows
        .iter()
        .filter(|w| w.out_of_sample.is_profitable())
        .count();
    profitable as f64 / windows.len() as f64
}

/// Compute aggregate `PerformanceMetrics` over all OOS trade lists and equity curves.
///
/// Concatenates trades and stitches OOS equity curves so each window starts
/// from the previous window's ending equity.
fn aggregate_oos_metrics(
    windows: &[WindowResult],
    risk_free_rate: f64,
    bars_per_year: f64,
) -> PerformanceMetrics {
    use crate::backtesting::result::EquityPoint;

    let all_trades: Vec<_> = windows
        .iter()
        .flat_map(|w| w.out_of_sample.trades.iter().cloned())
        .collect();

    // Stitch per-window equity into one continuous compounded series.
    // Each OOS window internally resets to its own initial capital; to avoid
    // synthetic drawdowns between windows, scale each window by the running
    // equity level from the previous window.
    let mut combined_equity: Vec<EquityPoint> = Vec::new();
    // `windows` is guaranteed non-empty by the validation above; index directly.
    let mut running_equity = windows[0].out_of_sample.initial_capital;

    for (window_idx, window) in windows.iter().enumerate() {
        let window_initial = window.out_of_sample.initial_capital;
        if window_initial <= 0.0 {
            continue;
        }

        for (point_idx, point) in window.out_of_sample.equity_curve.iter().enumerate() {
            if window_idx > 0 && point_idx == 0 {
                continue;
            }

            let scaled_equity = running_equity * (point.equity / window_initial);
            combined_equity.push(EquityPoint {
                timestamp: point.timestamp,
                equity: scaled_equity,
                drawdown_pct: 0.0,
            });
        }

        if let Some(last) = combined_equity.last() {
            running_equity = last.equity;
        }
    }

    // Recompute drawdowns on the stitched curve.
    let mut peak = f64::NEG_INFINITY;
    for point in &mut combined_equity {
        peak = peak.max(point.equity);
        point.drawdown_pct = if peak > 0.0 {
            (peak - point.equity) / peak
        } else {
            0.0
        };
    }

    // Aggregate metrics use the initial capital of the first OOS window.
    let initial_capital = windows
        .first()
        .map(|w| w.out_of_sample.initial_capital)
        .unwrap_or(10_000.0);

    let total_signals: usize = windows.iter().map(|w| w.out_of_sample.signals.len()).sum();
    let executed_signals: usize = windows
        .iter()
        .map(|w| {
            w.out_of_sample
                .signals
                .iter()
                .filter(|s| s.executed)
                .count()
        })
        .sum();

    PerformanceMetrics::calculate(
        &all_trades,
        &combined_equity,
        initial_capital,
        total_signals,
        executed_signals,
        risk_free_rate,
        bars_per_year,
    )
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
#[path = "walk_forward_tests.rs"]
mod tests;
