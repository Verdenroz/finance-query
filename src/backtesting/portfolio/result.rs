//! Portfolio backtest results.

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::backtesting::result::{BacktestResult, EquityPoint, PerformanceMetrics};

/// Snapshot of capital allocation at a single point in time.
#[non_exhaustive]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AllocationSnapshot {
    /// Timestamp (Unix seconds)
    pub timestamp: i64,

    /// Uninvested cash at this point
    pub cash: f64,

    /// Market value of each open position (symbol → value)
    pub positions: HashMap<String, f64>,
}

impl AllocationSnapshot {
    /// Total portfolio value (cash + all positions).
    pub fn total_equity(&self) -> f64 {
        self.cash + self.positions.values().sum::<f64>()
    }
}

/// Results of a multi-symbol portfolio backtest.
#[non_exhaustive]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PortfolioResult {
    /// Per-symbol backtest results.
    ///
    /// Each value is the standalone result for that symbol's trades as if the
    /// strategy had been run in isolation (same trades, same P&L, same metrics).
    pub symbols: HashMap<String, BacktestResult>,

    /// Combined portfolio equity curve over the entire backtest period.
    pub portfolio_equity_curve: Vec<EquityPoint>,

    /// Aggregate performance metrics computed from the combined equity curve
    /// and all trades across all symbols.
    pub portfolio_metrics: PerformanceMetrics,

    /// Starting capital shared across all symbols.
    pub initial_capital: f64,

    /// Final portfolio value (cash + open position values).
    pub final_equity: f64,

    /// Capital allocation snapshots recorded at each master timeline step.
    ///
    /// Use this to visualise how capital was distributed over time.
    pub allocation_history: Vec<AllocationSnapshot>,
}
