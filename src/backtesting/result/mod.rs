//! Backtest results and performance metrics.

mod benchmark;
#[cfg(test)]
mod fixtures;
mod metrics;
mod periods;
mod rolling;
mod stats;
mod tags;

pub use benchmark::BenchmarkMetrics;
pub use metrics::PerformanceMetrics;

use serde::{Deserialize, Serialize};

use super::config::BacktestConfig;
use super::position::{Position, Trade};
use super::signal::SignalDirection;

/// Point on the equity curve
#[non_exhaustive]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EquityPoint {
    /// Timestamp
    pub timestamp: i64,
    /// Portfolio equity at this point
    pub equity: f64,
    /// Current drawdown from peak as a **fraction** (0.0–1.0, not a percentage).
    ///
    /// `0.0` = equity is at its running all-time high; `0.2` = 20% below peak.
    /// Multiply by 100 to convert to a conventional percentage.
    pub drawdown_pct: f64,
}

/// Record of a generated signal (for analysis)
#[non_exhaustive]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignalRecord {
    /// Timestamp when signal was generated
    pub timestamp: i64,
    /// Price at signal time
    pub price: f64,
    /// Signal direction
    pub direction: SignalDirection,
    /// Signal strength (0.0-1.0)
    pub strength: f64,
    /// Signal reason/description
    pub reason: Option<String>,
    /// Whether the signal was executed
    pub executed: bool,
    /// Tags copied from the originating [`Signal`](crate::backtesting::Signal).
    ///
    /// Enables `BacktestResult::signals` to be filtered by tag so callers
    /// can compare total generated vs. executed signal counts per tag.
    #[serde(default)]
    pub tags: Vec<String>,
}

/// Complete backtest result
#[non_exhaustive]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BacktestResult {
    /// Symbol that was backtested
    pub symbol: String,

    /// Strategy name
    pub strategy_name: String,

    /// Configuration used
    pub config: BacktestConfig,

    /// Start timestamp
    pub start_timestamp: i64,

    /// End timestamp
    pub end_timestamp: i64,

    /// Initial capital
    pub initial_capital: f64,

    /// Final equity
    pub final_equity: f64,

    /// Performance metrics
    pub metrics: PerformanceMetrics,

    /// Complete trade log
    pub trades: Vec<Trade>,

    /// Equity curve (portfolio value at each bar)
    pub equity_curve: Vec<EquityPoint>,

    /// All signals generated (including non-executed)
    pub signals: Vec<SignalRecord>,

    /// Current open position (if any at end)
    pub open_position: Option<Position>,

    /// Benchmark comparison metrics (set when a benchmark is provided)
    pub benchmark: Option<BenchmarkMetrics>,

    /// Diagnostic messages (e.g. why zero trades were produced).
    ///
    /// Empty when the backtest ran without issues. Populated with actionable
    /// hints when the engine detects likely misconfiguration.
    #[serde(default)]
    pub diagnostics: Vec<String>,

    /// Highest gross exposure divided by equity reached on any bar.
    ///
    /// `0.0` for a run that never held a position, `1.0` for one that never
    /// borrowed. Compare against `BacktestConfig::max_leverage` to see how much
    /// of the allowance a strategy actually used.
    #[serde(default)]
    pub max_leverage_used: f64,
}

impl BacktestResult {
    /// Get a formatted summary string
    pub fn summary(&self) -> String {
        format!(
            "Backtest: {} on {}\n\
             Period: {} bars\n\
             Initial: ${:.2} -> Final: ${:.2}\n\
             Return: {:.2}% | Sharpe: {:.2} | Max DD: {:.2}%\n\
             Trades: {} | Win Rate: {:.1}% | Profit Factor: {:.2}",
            self.strategy_name,
            self.symbol,
            self.equity_curve.len(),
            self.initial_capital,
            self.final_equity,
            self.metrics.total_return_pct,
            self.metrics.sharpe_ratio,
            self.metrics.max_drawdown_pct * 100.0,
            self.metrics.total_trades,
            self.metrics.win_rate * 100.0,
            self.metrics.profit_factor,
        )
    }

    /// Check if the backtest was profitable
    pub fn is_profitable(&self) -> bool {
        self.final_equity > self.initial_capital
    }

    /// Get total P&L
    pub fn total_pnl(&self) -> f64 {
        self.final_equity - self.initial_capital
    }

    /// Get the number of bars in the backtest
    pub fn num_bars(&self) -> usize {
        self.equity_curve.len()
    }
}
