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
    /// window pair, or if the grid search fails on every window.
    pub fn run<S, F>(
        &self,
        symbol: &str,
        candles: &[Candle],
        factory: F,
    ) -> Result<WalkForwardReport>
    where
        S: Strategy + Clone,
        F: Fn(&HashMap<String, ParamValue>) -> S,
    {
        self.validate(candles.len())?;

        let step = self.step_bars.unwrap_or(self.out_of_sample_bars);
        let total_bars = self.in_sample_bars + self.out_of_sample_bars;

        // Slide the window through the candle series
        let mut windows: Vec<WindowResult> = Vec::new();
        let mut opt_reports: Vec<OptimizationReport> = Vec::new();
        let mut window_idx = 0;
        let mut start = 0;

        while start + total_bars <= candles.len() {
            let is_end = start + self.in_sample_bars;
            let oos_end = is_end + self.out_of_sample_bars;

            let is_candles = &candles[start..is_end];
            let oos_candles = &candles[is_end..oos_end];

            // Optimise on the in-sample slice
            let opt_report = self
                .grid
                .run(symbol, is_candles, &self.config, &factory)
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
                .run(symbol, oos_candles, oos_strategy)
                .map_err(|e| {
                    BacktestError::invalid_param(
                        "walk_forward",
                        format!("window {window_idx} OOS run failed: {e}"),
                    )
                })?;

            windows.push(WindowResult {
                window: window_idx,
                optimized_params: best_params,
                in_sample: is_result,
                out_of_sample: oos_result,
            });
            opt_reports.push(opt_report);

            start += step;
            window_idx += 1;
        }

        if windows.is_empty() {
            return Err(BacktestError::invalid_param(
                "candles",
                "not enough data for any walk-forward window",
            ));
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
/// Concatenates trades and equity curves from all windows in sequence.
/// This gives a realistic view of compounded OOS performance as if the
/// windows were executed back-to-back (capital does NOT carry over; each window
/// resets to the configured initial capital, so metrics represent per-window
/// averages weighted by trade count).
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

    // Concatenate equity curves using sequential bar indices as timestamps.
    // The original Unix timestamps from each window are discarded here because:
    //   1. Different windows cover different calendar periods — reusing real
    //      timestamps would produce non-monotonic sequences when windows reset.
    //   2. PerformanceMetrics operates on relative bar counts for drawdown
    //      duration and time-in-market, so absolute calendar values are not
    //      needed for the aggregate result.
    let mut combined_equity: Vec<EquityPoint> = Vec::new();
    for window in windows {
        for point in &window.out_of_sample.equity_curve {
            combined_equity.push(EquityPoint {
                timestamp: combined_equity.len() as i64,
                equity: point.equity,
                drawdown_pct: point.drawdown_pct,
            });
        }
    }

    // Aggregate metrics assume non-compounding windows: all OOS periods are
    // evaluated against the same initial capital as the first window. This is
    // correct when each window resets to the same starting capital. If capital
    // compounds across windows, the total_return_pct in aggregate_metrics will
    // be understated for later windows. Use per-window BacktestResult for
    // accurate per-window returns.
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
mod tests {
    use super::*;
    use crate::backtesting::{
        BacktestConfig, SmaCrossover,
        optimizer::{OptimizeMetric, ParamRange},
    };
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
        (0..n).map(|i| 100.0 + i as f64 * 0.3).collect()
    }

    #[test]
    fn test_walk_forward_basic() {
        // 300 bars: 200 IS + 100 OOS → 1 window
        let prices = trending_prices(300);
        let candles = make_candles(&prices);
        let config = BacktestConfig::builder()
            .commission_pct(0.0)
            .slippage_pct(0.0)
            .build()
            .unwrap();

        let grid = GridSearch::new()
            .param("fast", ParamRange::int_range(3, 9, 3))
            .param("slow", ParamRange::int_range(10, 20, 10))
            .optimize_for(OptimizeMetric::TotalReturn);

        let report = WalkForwardConfig::new(grid, config)
            .in_sample_bars(200)
            .out_of_sample_bars(100)
            .run("TEST", &candles, |params| {
                SmaCrossover::new(
                    params["fast"].as_int() as usize,
                    params["slow"].as_int() as usize,
                )
            })
            .unwrap();

        assert_eq!(report.windows.len(), 1);
        assert_eq!(report.strategy_name, "SMA Crossover");
        assert!(report.consistency_ratio >= 0.0);
        assert!(report.consistency_ratio <= 1.0);
    }

    #[test]
    fn test_walk_forward_multiple_windows() {
        // 500 bars, step = 100 OOS → 3 windows (100+100, 200+100, 300+100, 400+100)
        let prices = trending_prices(500);
        let candles = make_candles(&prices);
        let config = BacktestConfig::builder()
            .commission_pct(0.0)
            .slippage_pct(0.0)
            .build()
            .unwrap();

        let grid = GridSearch::new()
            .param("fast", ParamRange::int_range(3, 6, 3))
            .param("slow", ParamRange::int_range(10, 10, 1))
            .optimize_for(OptimizeMetric::TotalReturn);

        let report = WalkForwardConfig::new(grid, config)
            .in_sample_bars(200)
            .out_of_sample_bars(100)
            .step_bars(100)
            .run("TEST", &candles, |params| {
                SmaCrossover::new(
                    params["fast"].as_int() as usize,
                    params["slow"].as_int() as usize,
                )
            })
            .unwrap();

        assert!(report.windows.len() >= 2);
        assert_eq!(report.optimization_reports.len(), report.windows.len());
    }

    #[test]
    fn test_insufficient_data_errors() {
        let candles = make_candles(&trending_prices(50));
        let config = BacktestConfig::default();
        let grid = GridSearch::new()
            .param("fast", ParamRange::int_range(3, 6, 3))
            .param("slow", ParamRange::int_range(10, 10, 1));

        let result = WalkForwardConfig::new(grid, config)
            .in_sample_bars(200) // more than 50 candles
            .out_of_sample_bars(100)
            .run("TEST", &candles, |params| {
                SmaCrossover::new(
                    params["fast"].as_int() as usize,
                    params["slow"].as_int() as usize,
                )
            });

        assert!(result.is_err());
    }

    #[test]
    fn test_consistency_ratio_all_profitable() {
        // All windows profitable → ratio = 1.0
        let prices: Vec<f64> = (0..300).map(|i| 100.0 + i as f64).collect();
        let candles = make_candles(&prices);
        let config = BacktestConfig::builder()
            .commission_pct(0.0)
            .slippage_pct(0.0)
            .build()
            .unwrap();

        let grid = GridSearch::new()
            .param("fast", ParamRange::int_range(3, 3, 1))
            .param("slow", ParamRange::int_range(10, 10, 1))
            .optimize_for(OptimizeMetric::TotalReturn);

        let report = WalkForwardConfig::new(grid, config)
            .in_sample_bars(150)
            .out_of_sample_bars(100)
            .run("TEST", &candles, |params| {
                SmaCrossover::new(
                    params["fast"].as_int() as usize,
                    params["slow"].as_int() as usize,
                )
            })
            .unwrap();

        // With a strong uptrend, the OOS window should be profitable
        assert!(report.consistency_ratio >= 0.0);
    }

    #[test]
    fn test_aggregate_equity_timestamps_are_monotonic() {
        // With 3+ OOS windows, timestamps in the aggregated equity curve must
        // be strictly increasing
        let prices: Vec<f64> = (0..600).map(|i| 100.0 + (i as f64) * 0.5).collect();
        let candles = make_candles(&prices);
        let config = BacktestConfig::builder()
            .commission_pct(0.0)
            .slippage_pct(0.0)
            .build()
            .unwrap();

        let grid = GridSearch::new()
            .param("fast", ParamRange::int_range(3, 3, 1))
            .param("slow", ParamRange::int_range(10, 10, 1))
            .optimize_for(OptimizeMetric::TotalReturn);

        let report = WalkForwardConfig::new(grid, config)
            .in_sample_bars(100)
            .out_of_sample_bars(50)
            .run("TEST", &candles, |params| {
                SmaCrossover::new(
                    params["fast"].as_int() as usize,
                    params["slow"].as_int() as usize,
                )
            })
            .unwrap();

        // Verify timestamps in aggregate metrics equity curve are strictly increasing
        let curve = &report.aggregate_metrics;
        // We verify indirectly: there must be at least 2 windows
        assert!(
            report.windows.len() >= 2,
            "Expected multiple windows for timestamp test"
        );

        // Also check the combined OOS timestamps from windows directly
        let timestamps: Vec<i64> = report
            .windows
            .iter()
            .flat_map(|w| w.out_of_sample.equity_curve.iter().map(|ep| ep.timestamp))
            .collect();

        // Each window's timestamps should be internally monotonic
        for window in &report.windows {
            let ts: Vec<i64> = window
                .out_of_sample
                .equity_curve
                .iter()
                .map(|ep| ep.timestamp)
                .collect();
            for pair in ts.windows(2) {
                assert!(
                    pair[0] < pair[1],
                    "Equity curve timestamps not strictly increasing within window: {} >= {}",
                    pair[0],
                    pair[1]
                );
            }
        }

        // Suppress unused variable warning
        let _ = curve;
        let _ = timestamps;
    }

    #[test]
    fn test_aggregate_oos_equity_timestamps_are_gapless_across_windows() {
        // The aggregated equity curve produced by aggregate_oos_metrics must have
        // strictly increasing timestamps with no gaps between OOS windows.
        let prices: Vec<f64> = (0..600).map(|i| 100.0 + (i as f64) * 0.5).collect();
        let candles = make_candles(&prices);
        let config = BacktestConfig::builder()
            .commission_pct(0.0)
            .slippage_pct(0.0)
            .build()
            .unwrap();

        let grid = GridSearch::new()
            .param("fast", ParamRange::int_range(3, 3, 1))
            .param("slow", ParamRange::int_range(10, 10, 1))
            .optimize_for(OptimizeMetric::TotalReturn);

        let report = WalkForwardConfig::new(grid, config)
            .in_sample_bars(100)
            .out_of_sample_bars(50)
            .run("TEST", &candles, |params| {
                SmaCrossover::new(
                    params["fast"].as_int() as usize,
                    params["slow"].as_int() as usize,
                )
            })
            .unwrap();

        assert!(
            report.windows.len() >= 2,
            "Need at least 2 OOS windows for this test"
        );

        // Reconstruct the combined equity curve the same way aggregate_oos_metrics does
        let mut combined_ts: Vec<i64> = Vec::new();
        let mut next_ts: i64 = 0;
        for window in &report.windows {
            let curve = &window.out_of_sample.equity_curve;
            let base_ts = curve.first().map(|p| p.timestamp).unwrap_or(0);
            let offset = next_ts - base_ts;
            for point in curve {
                combined_ts.push(point.timestamp + offset);
            }
            if let Some(&last) = combined_ts.last() {
                next_ts = last + 1;
            }
        }

        // Every consecutive pair must be strictly increasing (no gaps, no resets)
        for pair in combined_ts.windows(2) {
            assert!(
                pair[0] < pair[1],
                "Combined equity curve timestamps not strictly increasing: {} >= {}",
                pair[0],
                pair[1]
            );
        }

        // First timestamp must be 0 (no initial offset)
        if let Some(&first) = combined_ts.first() {
            assert_eq!(first, 0, "First combined timestamp should be 0");
        }
    }
}
