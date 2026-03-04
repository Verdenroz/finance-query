//! Side-by-side comparison of multiple backtest results.
//!
//! Use [`BacktestComparison`] to rank several [`BacktestResult`]s by a chosen
//! metric and inspect every strategy's key numbers in one place.
//!
//! # Example
//!
//! ```ignore
//! use finance_query::backtesting::{
//!     BacktestComparison, BacktestConfig, SmaCrossover, MacdSignal,
//!     optimizer::OptimizeMetric,
//! };
//! use finance_query::{Ticker, Interval, TimeRange};
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let ticker = Ticker::new("AAPL").await?;
//! let config  = BacktestConfig::default();
//!
//! let result1 = ticker.backtest(SmaCrossover::new(10, 50), Interval::OneDay, TimeRange::OneYear, None).await?;
//! let result2 = ticker.backtest(MacdSignal::default(),     Interval::OneDay, TimeRange::OneYear, None).await?;
//!
//! let report = BacktestComparison::new()
//!     .add("SMA 10/50", result1)
//!     .add("MACD Signal", result2)
//!     .ranked_by(OptimizeMetric::SharpeRatio);
//!
//! println!("Winner: {}", report.winner());
//! for row in report.table() {
//!     println!("{}: sharpe={:.2} return={:.1}%", row.label, row.sharpe_ratio, row.total_return_pct);
//! }
//! # Ok(())
//! # }
//! ```

use serde::{Deserialize, Serialize};

use super::optimizer::OptimizeMetric;
use super::result::BacktestResult;

// ── ComparisonRow ────────────────────────────────────────────────────────────

/// A single row in the comparison table — one strategy's key metrics.
#[non_exhaustive]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComparisonRow {
    /// User-supplied label for the strategy (e.g. `"SMA 10/50"`).
    pub label: String,

    /// Name embedded in the [`BacktestResult`] by the engine.
    pub strategy_name: String,

    /// Symbol that was tested.
    pub symbol: String,

    /// Total return percentage.
    pub total_return_pct: f64,

    /// Annualised return percentage.
    pub annualized_return_pct: f64,

    /// Sharpe ratio.
    pub sharpe_ratio: f64,

    /// Sortino ratio.
    pub sortino_ratio: f64,

    /// Calmar ratio.
    pub calmar_ratio: f64,

    /// Maximum drawdown as a fraction (0.0–1.0).
    ///
    /// Multiply by 100 to get a conventional percentage.
    pub max_drawdown_pct: f64,

    /// Win rate (`winning_trades / total_trades`).
    pub win_rate: f64,

    /// Profit factor (`gross_profit / gross_loss`).
    pub profit_factor: f64,

    /// Total number of completed trades.
    pub total_trades: usize,

    /// Kelly Criterion: optimal capital fraction to risk per trade.
    pub kelly_criterion: f64,

    /// System Quality Number.
    pub sqn: f64,

    /// Expectancy in dollar terms per trade.
    pub expectancy: f64,

    /// Omega Ratio.
    pub omega_ratio: f64,

    /// Time in market as a fraction (0.0–1.0).
    pub time_in_market_pct: f64,

    /// The score on the metric used to rank the comparison.
    pub rank_score: f64,

    /// 1-based rank within the comparison (1 = best).
    pub rank: usize,
}

impl ComparisonRow {
    fn from_result(label: &str, result: &BacktestResult, metric: OptimizeMetric) -> Self {
        let m = &result.metrics;
        let rank_score = metric.score(result);
        ComparisonRow {
            label: label.to_owned(),
            strategy_name: result.strategy_name.clone(),
            symbol: result.symbol.clone(),
            total_return_pct: m.total_return_pct,
            annualized_return_pct: m.annualized_return_pct,
            sharpe_ratio: m.sharpe_ratio,
            sortino_ratio: m.sortino_ratio,
            calmar_ratio: m.calmar_ratio,
            max_drawdown_pct: m.max_drawdown_pct,
            win_rate: m.win_rate,
            profit_factor: m.profit_factor,
            total_trades: m.total_trades,
            kelly_criterion: m.kelly_criterion,
            sqn: m.sqn,
            expectancy: m.expectancy,
            omega_ratio: m.omega_ratio,
            time_in_market_pct: m.time_in_market_pct,
            rank_score,
            // placeholder; assigned after sorting
            rank: 0,
        }
    }
}

// ── BacktestComparison (builder) ──────────────────────────────────────────────

/// Builder that accumulates [`BacktestResult`]s and ranks them.
///
/// # Ordering
///
/// Call [`ranked_by`](BacktestComparison::ranked_by) to produce a
/// [`ComparisonReport`] sorted best-first by the chosen [`OptimizeMetric`].
#[derive(Debug, Default)]
pub struct BacktestComparison {
    entries: Vec<(String, BacktestResult)>,
}

impl BacktestComparison {
    /// Create an empty comparison.
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a labelled backtest result.
    ///
    /// The `label` is an arbitrary human-readable name (e.g. `"SMA 10/50"`).
    /// It does **not** have to match the strategy's internal name.
    pub fn add(mut self, label: impl Into<String>, result: BacktestResult) -> Self {
        self.entries.push((label.into(), result));
        self
    }

    /// Rank all added results by `metric` and return a [`ComparisonReport`].
    ///
    /// Results are sorted **best-first** (highest score wins for all metrics
    /// except [`OptimizeMetric::MinDrawdown`], which is already negated
    /// internally so that a lower drawdown yields a higher score).
    pub fn ranked_by(self, metric: OptimizeMetric) -> ComparisonReport {
        let mut rows: Vec<ComparisonRow> = self
            .entries
            .iter()
            .map(|(label, result)| ComparisonRow::from_result(label, result, metric))
            .collect();

        // Sort best-first; use total_return_pct as a tie-breaker.
        rows.sort_by(|a, b| {
            b.rank_score
                .partial_cmp(&a.rank_score)
                .unwrap_or(std::cmp::Ordering::Equal)
                .then_with(|| {
                    b.total_return_pct
                        .partial_cmp(&a.total_return_pct)
                        .unwrap_or(std::cmp::Ordering::Equal)
                })
        });

        // Assign 1-based ranks.
        for (idx, row) in rows.iter_mut().enumerate() {
            row.rank = idx + 1;
        }

        ComparisonReport { rows, metric }
    }
}

// ── ComparisonReport ──────────────────────────────────────────────────────────

/// Ranked comparison of multiple backtest results produced by
/// [`BacktestComparison::ranked_by`].
#[non_exhaustive]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComparisonReport {
    /// Rows sorted best-first by the chosen metric.
    pub rows: Vec<ComparisonRow>,
    /// The metric used for ranking.
    pub metric: OptimizeMetric,
}

impl ComparisonReport {
    /// Label of the best-performing strategy.
    ///
    /// Returns `""` when the report contains no entries.
    pub fn winner(&self) -> &str {
        self.rows.first().map(|r| r.label.as_str()).unwrap_or("")
    }

    /// All rows sorted best-first (rank 1 = winner).
    pub fn table(&self) -> &[ComparisonRow] {
        &self.rows
    }

    /// Returns the row for the winning strategy, if any.
    pub fn winner_row(&self) -> Option<&ComparisonRow> {
        self.rows.first()
    }

    /// Returns the number of strategies in the comparison.
    pub fn len(&self) -> usize {
        self.rows.len()
    }

    /// Returns `true` when no results were added.
    pub fn is_empty(&self) -> bool {
        self.rows.is_empty()
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::backtesting::{
        BacktestConfig,
        optimizer::OptimizeMetric,
        result::{BacktestResult, EquityPoint, PerformanceMetrics},
    };

    /// Build a minimal `PerformanceMetrics` with only the fields under test set.
    fn metrics_with(
        total_return_pct: f64,
        sharpe_ratio: f64,
        max_drawdown_pct: f64,
    ) -> PerformanceMetrics {
        PerformanceMetrics {
            total_return_pct,
            annualized_return_pct: total_return_pct,
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
        }
    }

    fn make_result(strategy_name: &str, total_return: f64, sharpe: f64, dd: f64) -> BacktestResult {
        BacktestResult {
            symbol: "TEST".to_owned(),
            strategy_name: strategy_name.to_owned(),
            config: BacktestConfig::default(),
            start_timestamp: 0,
            end_timestamp: 1_000_000,
            initial_capital: 10_000.0,
            final_equity: 10_000.0 * (1.0 + total_return / 100.0),
            metrics: metrics_with(total_return, sharpe, dd),
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
        }
    }

    #[test]
    fn empty_comparison() {
        let report = BacktestComparison::new().ranked_by(OptimizeMetric::SharpeRatio);
        assert!(report.is_empty());
        assert_eq!(report.winner(), "");
        assert!(report.winner_row().is_none());
        assert_eq!(report.table().len(), 0);
    }

    #[test]
    fn single_entry_is_winner() {
        let result = make_result("SMA", 10.0, 1.5, 0.05);
        let report = BacktestComparison::new()
            .add("SMA 10/50", result)
            .ranked_by(OptimizeMetric::SharpeRatio);

        assert_eq!(report.winner(), "SMA 10/50");
        assert_eq!(report.len(), 1);
        assert_eq!(report.table()[0].rank, 1);
    }

    #[test]
    fn ranked_by_sharpe() {
        let r1 = make_result("SMA", 10.0, 0.8, 0.10);
        let r2 = make_result("MACD", 15.0, 1.5, 0.12);
        let r3 = make_result("RSI", 5.0, 1.2, 0.08);

        let report = BacktestComparison::new()
            .add("SMA 10/50", r1)
            .add("MACD Signal", r2)
            .add("RSI Mean Rev", r3)
            .ranked_by(OptimizeMetric::SharpeRatio);

        assert_eq!(report.winner(), "MACD Signal");
        let table = report.table();
        assert_eq!(table[0].label, "MACD Signal");
        assert_eq!(table[1].label, "RSI Mean Rev");
        assert_eq!(table[2].label, "SMA 10/50");
        assert_eq!(table[0].rank, 1);
        assert_eq!(table[1].rank, 2);
        assert_eq!(table[2].rank, 3);
    }

    #[test]
    fn ranked_by_total_return() {
        let r1 = make_result("SMA", 10.0, 0.8, 0.10);
        let r2 = make_result("MACD", 25.0, 0.6, 0.20);

        let report = BacktestComparison::new()
            .add("SMA", r1)
            .add("MACD", r2)
            .ranked_by(OptimizeMetric::TotalReturn);

        assert_eq!(report.winner(), "MACD");
    }

    #[test]
    fn ranked_by_min_drawdown() {
        // Lower drawdown should rank higher.
        let r1 = make_result("SMA", 10.0, 0.8, 0.20);
        let r2 = make_result("MACD", 10.0, 0.8, 0.05);

        let report = BacktestComparison::new()
            .add("High DD", r1)
            .add("Low DD", r2)
            .ranked_by(OptimizeMetric::MinDrawdown);

        assert_eq!(report.winner(), "Low DD");
    }

    #[test]
    fn tie_broken_by_total_return() {
        // Both have identical Sharpe; higher return should win.
        let r1 = make_result("A", 20.0, 1.0, 0.10);
        let r2 = make_result("B", 5.0, 1.0, 0.10);

        let report = BacktestComparison::new()
            .add("A", r1)
            .add("B", r2)
            .ranked_by(OptimizeMetric::SharpeRatio);

        assert_eq!(report.winner(), "A");
    }

    #[test]
    fn table_returns_all_rows() {
        let n = 5;
        let mut comparison = BacktestComparison::new();
        for i in 0..n {
            comparison = comparison.add(
                format!("Strategy {i}"),
                make_result(&format!("S{i}"), i as f64 * 2.0, i as f64 * 0.5, 0.1),
            );
        }
        let report = comparison.ranked_by(OptimizeMetric::SharpeRatio);
        assert_eq!(report.table().len(), n);
        assert_eq!(report.len(), n);
    }

    #[test]
    fn row_fields_populated_correctly() {
        let result = make_result("SMA", 12.0, 1.3, 0.07);
        let report = BacktestComparison::new()
            .add("My Strategy", result)
            .ranked_by(OptimizeMetric::SharpeRatio);

        let row = &report.table()[0];
        assert_eq!(row.label, "My Strategy");
        assert_eq!(row.strategy_name, "SMA");
        assert_eq!(row.symbol, "TEST");
        assert!((row.total_return_pct - 12.0).abs() < 1e-10);
        assert!((row.sharpe_ratio - 1.3).abs() < 1e-10);
        assert!((row.max_drawdown_pct - 0.07).abs() < 1e-10);
        assert_eq!(row.rank, 1);
    }
}
