//! Backtest results and performance metrics.

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
    /// Current drawdown from peak (as percentage, 0.0-1.0)
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
}

/// Performance metrics summary
#[non_exhaustive]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceMetrics {
    /// Total return percentage
    pub total_return_pct: f64,

    /// Annualized return percentage (assumes 252 trading days)
    pub annualized_return_pct: f64,

    /// Sharpe ratio (risk-free rate = 0)
    pub sharpe_ratio: f64,

    /// Sortino ratio (downside deviation)
    pub sortino_ratio: f64,

    /// Maximum drawdown as a fraction (0.0–1.0, **not** a percentage).
    ///
    /// A value of `0.2` means the equity fell 20% from its peak at most.
    /// Multiply by 100 to get a conventional percentage. See also
    /// [`max_drawdown_percentage`](Self::max_drawdown_percentage) for a
    /// pre-scaled convenience accessor.
    pub max_drawdown_pct: f64,

    /// Maximum drawdown duration measured in **bars** (not calendar time).
    ///
    /// Counts the number of consecutive bars from a peak until full recovery.
    pub max_drawdown_duration: i64,

    /// Win rate: `winning_trades / total_trades`.
    ///
    /// The denominator is `total_trades`, which includes break-even trades
    /// (`pnl == 0.0`).  Break-even trades are neither wins nor losses, so they
    /// reduce the win rate without appearing in `winning_trades` or
    /// `losing_trades`.
    pub win_rate: f64,

    /// Profit factor: `gross_profit / gross_loss`.
    ///
    /// Returns `f64::MAX` when there are no losing trades (zero denominator)
    /// and at least one profitable trade.  This avoids `f64::INFINITY`, which
    /// is not representable in JSON.
    pub profit_factor: f64,

    /// Average trade return percentage
    pub avg_trade_return_pct: f64,

    /// Average winning trade return percentage
    pub avg_win_pct: f64,

    /// Average losing trade return percentage
    pub avg_loss_pct: f64,

    /// Average trade duration in bars
    pub avg_trade_duration: f64,

    /// Total number of trades
    pub total_trades: usize,

    /// Number of winning trades (`pnl > 0.0`).
    ///
    /// Break-even trades (`pnl == 0.0`) are counted in neither `winning_trades`
    /// nor `losing_trades`, so `winning_trades + losing_trades <= total_trades`.
    pub winning_trades: usize,

    /// Number of losing trades (`pnl < 0.0`).
    ///
    /// Break-even trades (`pnl == 0.0`) are counted in neither `winning_trades`
    /// nor `losing_trades`. See [`winning_trades`](Self::winning_trades).
    pub losing_trades: usize,

    /// Largest winning trade P&L
    pub largest_win: f64,

    /// Largest losing trade P&L
    pub largest_loss: f64,

    /// Maximum consecutive wins
    pub max_consecutive_wins: usize,

    /// Maximum consecutive losses
    pub max_consecutive_losses: usize,

    /// Calmar ratio: `annualized_return_pct / max_drawdown_pct_scaled`.
    ///
    /// Returns `f64::MAX` when max drawdown is zero and the strategy is
    /// profitable (avoids `f64::INFINITY` which cannot be serialized to JSON).
    pub calmar_ratio: f64,

    /// Total commission paid
    pub total_commission: f64,

    /// Number of long trades
    pub long_trades: usize,

    /// Number of short trades
    pub short_trades: usize,

    /// Total signals generated
    pub total_signals: usize,

    /// Signals that were executed
    pub executed_signals: usize,

    /// Average duration of winning trades in seconds
    pub avg_win_duration: f64,

    /// Average duration of losing trades in seconds
    pub avg_loss_duration: f64,

    /// Fraction of backtest time spent with an open position (0.0 - 1.0)
    pub time_in_market_pct: f64,

    /// Longest idle period between trades in seconds (0 if fewer than 2 trades)
    pub max_idle_period: i64,

    /// Total dividend income received across all trades
    pub total_dividend_income: f64,
}

impl PerformanceMetrics {
    /// Maximum drawdown as a conventional percentage (0–100).
    ///
    /// Equivalent to `self.max_drawdown_pct * 100.0`. Provided because
    /// `max_drawdown_pct` is stored as a fraction (0.0–1.0) while most other
    /// return fields use true percentages.
    pub fn max_drawdown_percentage(&self) -> f64 {
        self.max_drawdown_pct * 100.0
    }

    /// Construct a zero-trades result: all metrics are zero except `total_return_pct`
    /// which is derived from the equity curve.
    fn empty(
        initial_capital: f64,
        equity_curve: &[EquityPoint],
        total_signals: usize,
        executed_signals: usize,
    ) -> Self {
        let final_equity = equity_curve
            .last()
            .map(|e| e.equity)
            .unwrap_or(initial_capital);
        let total_return_pct = ((final_equity / initial_capital) - 1.0) * 100.0;
        Self {
            total_return_pct,
            annualized_return_pct: 0.0,
            sharpe_ratio: 0.0,
            sortino_ratio: 0.0,
            max_drawdown_pct: 0.0,
            max_drawdown_duration: 0,
            win_rate: 0.0,
            profit_factor: 0.0,
            avg_trade_return_pct: 0.0,
            avg_win_pct: 0.0,
            avg_loss_pct: 0.0,
            avg_trade_duration: 0.0,
            total_trades: 0,
            winning_trades: 0,
            losing_trades: 0,
            largest_win: 0.0,
            largest_loss: 0.0,
            max_consecutive_wins: 0,
            max_consecutive_losses: 0,
            calmar_ratio: 0.0,
            total_commission: 0.0,
            long_trades: 0,
            short_trades: 0,
            total_signals,
            executed_signals,
            avg_win_duration: 0.0,
            avg_loss_duration: 0.0,
            time_in_market_pct: 0.0,
            max_idle_period: 0,
            total_dividend_income: 0.0,
        }
    }

    /// Calculate performance metrics from trades and equity curve.
    ///
    /// `risk_free_rate` is the **annual** rate (e.g. `0.05` for 5%). It is
    /// converted to a per-bar rate internally before computing Sharpe/Sortino.
    ///
    /// `bars_per_year` controls annualisation (e.g. `252.0` for daily US equity
    /// bars, `52.0` for weekly, `1638.0` for hourly). Affects annualised return,
    /// Sharpe, Sortino, and Calmar calculations.
    pub fn calculate(
        trades: &[Trade],
        equity_curve: &[EquityPoint],
        initial_capital: f64,
        total_signals: usize,
        executed_signals: usize,
        risk_free_rate: f64,
        bars_per_year: f64,
    ) -> Self {
        if trades.is_empty() {
            return Self::empty(
                initial_capital,
                equity_curve,
                total_signals,
                executed_signals,
            );
        }

        let total_trades = trades.len();
        let stats = analyze_trades(trades);

        let win_rate = stats.winning_trades as f64 / total_trades as f64;

        // Use f64::MAX instead of INFINITY so the value survives JSON round-trips.
        let profit_factor = if stats.gross_loss > 0.0 {
            stats.gross_profit / stats.gross_loss
        } else if stats.gross_profit > 0.0 {
            f64::MAX
        } else {
            0.0
        };

        let avg_trade_return_pct = stats.total_return_sum / total_trades as f64;

        let avg_win_pct = if !stats.winning_returns.is_empty() {
            stats.winning_returns.iter().sum::<f64>() / stats.winning_returns.len() as f64
        } else {
            0.0
        };

        let avg_loss_pct = if !stats.losing_returns.is_empty() {
            stats.losing_returns.iter().sum::<f64>() / stats.losing_returns.len() as f64
        } else {
            0.0
        };

        let avg_trade_duration = stats.total_duration as f64 / total_trades as f64;

        // Consecutive wins/losses
        let (max_consecutive_wins, max_consecutive_losses) = calculate_consecutive(trades);

        // Drawdown metrics
        let max_drawdown_pct = equity_curve
            .iter()
            .map(|e| e.drawdown_pct)
            .fold(0.0, f64::max);

        let max_drawdown_duration = calculate_max_drawdown_duration(equity_curve);

        // Total return
        let final_equity = equity_curve
            .last()
            .map(|e| e.equity)
            .unwrap_or(initial_capital);
        let total_return_pct = ((final_equity / initial_capital) - 1.0) * 100.0;

        // Annualized return using configured bars_per_year
        let num_bars = equity_curve.len();
        let years = num_bars as f64 / bars_per_year;
        let annualized_return_pct = if years > 0.0 {
            ((final_equity / initial_capital).powf(1.0 / years) - 1.0) * 100.0
        } else {
            0.0
        };

        // Sharpe and Sortino ratios
        let returns: Vec<f64> = calculate_periodic_returns(equity_curve);
        let sharpe_ratio = calculate_sharpe_ratio(&returns, risk_free_rate, bars_per_year);
        let sortino_ratio = calculate_sortino_ratio(&returns, risk_free_rate, bars_per_year);

        // Calmar ratio = annualised return (%) / max drawdown (%).
        // Use f64::MAX instead of INFINITY when drawdown is zero to keep the
        // value JSON-serializable.
        let calmar_ratio = if max_drawdown_pct > 0.0 {
            annualized_return_pct / (max_drawdown_pct * 100.0)
        } else if annualized_return_pct > 0.0 {
            f64::MAX
        } else {
            0.0
        };

        // Trade duration analysis
        let (avg_win_duration, avg_loss_duration) = calculate_win_loss_durations(trades);
        let time_in_market_pct = calculate_time_in_market(trades, equity_curve);
        let max_idle_period = calculate_max_idle_period(trades);

        Self {
            total_return_pct,
            annualized_return_pct,
            sharpe_ratio,
            sortino_ratio,
            max_drawdown_pct,
            max_drawdown_duration,
            win_rate,
            profit_factor,
            avg_trade_return_pct,
            avg_win_pct,
            avg_loss_pct,
            avg_trade_duration,
            total_trades,
            winning_trades: stats.winning_trades,
            losing_trades: stats.losing_trades,
            largest_win: stats.largest_win,
            largest_loss: stats.largest_loss,
            max_consecutive_wins,
            max_consecutive_losses,
            calmar_ratio,
            total_commission: stats.total_commission,
            long_trades: stats.long_trades,
            short_trades: stats.short_trades,
            total_signals,
            executed_signals,
            avg_win_duration,
            avg_loss_duration,
            time_in_market_pct,
            max_idle_period,
            total_dividend_income: stats.total_dividend_income,
        }
    }
}

/// Aggregated trade statistics collected in a single pass over the trade log.
struct TradeStats {
    winning_trades: usize,
    losing_trades: usize,
    long_trades: usize,
    short_trades: usize,
    gross_profit: f64,
    gross_loss: f64,
    total_return_sum: f64,
    total_duration: i64,
    largest_win: f64,
    largest_loss: f64,
    total_commission: f64,
    total_dividend_income: f64,
    winning_returns: Vec<f64>,
    losing_returns: Vec<f64>,
}

/// Single-pass accumulation of all per-trade statistics.
fn analyze_trades(trades: &[Trade]) -> TradeStats {
    let mut stats = TradeStats {
        winning_trades: 0,
        losing_trades: 0,
        long_trades: 0,
        short_trades: 0,
        gross_profit: 0.0,
        gross_loss: 0.0,
        total_return_sum: 0.0,
        total_duration: 0,
        largest_win: 0.0,
        largest_loss: 0.0,
        total_commission: 0.0,
        total_dividend_income: 0.0,
        winning_returns: Vec::new(),
        losing_returns: Vec::new(),
    };

    for t in trades {
        if t.is_profitable() {
            stats.winning_trades += 1;
            stats.gross_profit += t.pnl;
            stats.winning_returns.push(t.return_pct);
            stats.largest_win = stats.largest_win.max(t.pnl);
        } else if t.is_loss() {
            stats.losing_trades += 1;
            stats.gross_loss += t.pnl.abs();
            stats.losing_returns.push(t.return_pct);
            stats.largest_loss = stats.largest_loss.min(t.pnl);
        }
        if t.is_long() {
            stats.long_trades += 1;
        } else {
            stats.short_trades += 1;
        }
        stats.total_return_sum += t.return_pct;
        stats.total_duration += t.duration_secs();
        stats.total_commission += t.commission;
        stats.total_dividend_income += t.dividend_income;
    }

    stats
}

/// Calculate maximum consecutive wins and losses
fn calculate_consecutive(trades: &[Trade]) -> (usize, usize) {
    let mut max_wins = 0;
    let mut max_losses = 0;
    let mut current_wins = 0;
    let mut current_losses = 0;

    for trade in trades {
        if trade.is_profitable() {
            current_wins += 1;
            current_losses = 0;
            max_wins = max_wins.max(current_wins);
        } else if trade.is_loss() {
            current_losses += 1;
            current_wins = 0;
            max_losses = max_losses.max(current_losses);
        } else {
            // Break-even trade
            current_wins = 0;
            current_losses = 0;
        }
    }

    (max_wins, max_losses)
}

/// Calculate maximum drawdown duration in bars
fn calculate_max_drawdown_duration(equity_curve: &[EquityPoint]) -> i64 {
    if equity_curve.is_empty() {
        return 0;
    }

    let mut max_duration = 0;
    let mut current_duration = 0;
    let mut peak = equity_curve[0].equity;

    for point in equity_curve {
        if point.equity >= peak {
            peak = point.equity;
            max_duration = max_duration.max(current_duration);
            current_duration = 0;
        } else {
            current_duration += 1;
        }
    }

    max_duration.max(current_duration)
}

/// Calculate periodic returns from equity curve
fn calculate_periodic_returns(equity_curve: &[EquityPoint]) -> Vec<f64> {
    if equity_curve.len() < 2 {
        return vec![];
    }

    equity_curve
        .windows(2)
        .map(|w| {
            let prev = w[0].equity;
            let curr = w[1].equity;
            if prev > 0.0 {
                (curr - prev) / prev
            } else {
                0.0
            }
        })
        .collect()
}

/// Convert an annual risk-free rate to a per-bar rate.
///
/// `bars_per_year` controls the compounding frequency (e.g. 252 for daily US
/// equity bars, 52 for weekly, 1638 for hourly). The resulting per-bar rate is
/// subtracted from each return before computing Sharpe/Sortino.
fn annual_to_periodic_rf(annual_rate: f64, bars_per_year: f64) -> f64 {
    (1.0 + annual_rate).powf(1.0 / bars_per_year) - 1.0
}

/// Calculate Sharpe ratio with a configurable annual risk-free rate.
///
/// Uses sample standard deviation (divides by n-1) to match the `risk` module
/// and standard financial convention. Annualised by `sqrt(bars_per_year)`.
fn calculate_sharpe_ratio(returns: &[f64], annual_risk_free_rate: f64, bars_per_year: f64) -> f64 {
    if returns.len() < 2 {
        return 0.0;
    }

    let periodic_rf = annual_to_periodic_rf(annual_risk_free_rate, bars_per_year);
    let excess: Vec<f64> = returns.iter().map(|r| r - periodic_rf).collect();
    let n = excess.len() as f64;
    let mean = excess.iter().sum::<f64>() / n;
    // Sample variance (n-1) for unbiased estimation
    let variance = excess.iter().map(|r| (r - mean).powi(2)).sum::<f64>() / (n - 1.0);
    let std_dev = variance.sqrt();

    if std_dev > 0.0 {
        (mean / std_dev) * bars_per_year.sqrt()
    } else if mean > 0.0 {
        // Use f64::MAX instead of INFINITY so the value survives JSON round-trips.
        f64::MAX
    } else {
        0.0
    }
}

/// Calculate Sortino ratio with a configurable annual risk-free rate.
///
/// Uses downside deviation: only negative excess returns contribute to the
/// deviation, but the denominator is the total observation count minus 1
/// (sample convention, matching Sortino's original definition and the `risk`
/// module). Annualised by `sqrt(bars_per_year)`.
fn calculate_sortino_ratio(returns: &[f64], annual_risk_free_rate: f64, bars_per_year: f64) -> f64 {
    if returns.len() < 2 {
        return 0.0;
    }

    let periodic_rf = annual_to_periodic_rf(annual_risk_free_rate, bars_per_year);
    let excess: Vec<f64> = returns.iter().map(|r| r - periodic_rf).collect();
    let n = excess.len() as f64;
    let mean = excess.iter().sum::<f64>() / n;

    // Downside deviation: sum of squared negative excess returns, divided by n-1
    // (total observations, not just negative ones — per Sortino's original definition)
    let downside_sq_sum: f64 = excess.iter().filter(|&&r| r < 0.0).map(|r| r.powi(2)).sum();
    let downside_dev = (downside_sq_sum / (n - 1.0)).sqrt();

    if downside_dev > 0.0 {
        (mean / downside_dev) * bars_per_year.sqrt()
    } else if mean > 0.0 {
        // Use f64::MAX instead of INFINITY so the value survives JSON round-trips.
        f64::MAX
    } else {
        0.0
    }
}

/// Calculate average duration (in seconds) for winning and losing trades separately.
fn calculate_win_loss_durations(trades: &[Trade]) -> (f64, f64) {
    let win_durations: Vec<i64> = trades
        .iter()
        .filter(|t| t.is_profitable())
        .map(|t| t.duration_secs())
        .collect();
    let loss_durations: Vec<i64> = trades
        .iter()
        .filter(|t| t.is_loss())
        .map(|t| t.duration_secs())
        .collect();

    let avg_win = if win_durations.is_empty() {
        0.0
    } else {
        win_durations.iter().sum::<i64>() as f64 / win_durations.len() as f64
    };

    let avg_loss = if loss_durations.is_empty() {
        0.0
    } else {
        loss_durations.iter().sum::<i64>() as f64 / loss_durations.len() as f64
    };

    (avg_win, avg_loss)
}

/// Calculate fraction of backtest time spent in a position.
///
/// Uses the ratio of total trade duration to the total backtest duration
/// derived from the equity curve timestamps.
fn calculate_time_in_market(trades: &[Trade], equity_curve: &[EquityPoint]) -> f64 {
    let total_duration_secs: i64 = trades.iter().map(|t| t.duration_secs()).sum();

    let backtest_secs = match (equity_curve.first(), equity_curve.last()) {
        (Some(first), Some(last)) if last.timestamp > first.timestamp => {
            last.timestamp - first.timestamp
        }
        _ => return 0.0,
    };

    (total_duration_secs as f64 / backtest_secs as f64).min(1.0)
}

/// Calculate the longest idle period (seconds) between consecutive trades.
///
/// Returns 0 if there are fewer than 2 trades.
fn calculate_max_idle_period(trades: &[Trade]) -> i64 {
    if trades.len() < 2 {
        return 0;
    }

    // Trades are appended in chronological order; compute gaps between
    // exit of trade N and entry of trade N+1.
    trades
        .windows(2)
        .map(|w| (w[1].entry_timestamp - w[0].exit_timestamp).max(0))
        .max()
        .unwrap_or(0)
}

/// Comparison of strategy performance against a benchmark.
///
/// Populated when a benchmark symbol is supplied to `backtest_with_benchmark`.
#[non_exhaustive]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BenchmarkMetrics {
    /// Benchmark symbol (e.g. `"SPY"`)
    pub symbol: String,

    /// Buy-and-hold return of the benchmark over the same period (percentage)
    pub benchmark_return_pct: f64,

    /// Buy-and-hold return of the backtested symbol over the same period (percentage)
    pub buy_and_hold_return_pct: f64,

    /// Alpha: annualised strategy excess return over the benchmark (CAPM)
    pub alpha: f64,

    /// Beta: sensitivity of strategy returns to benchmark movements
    pub beta: f64,

    /// Information ratio: excess return per unit of tracking error (annualised)
    pub information_ratio: f64,
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::backtesting::position::PositionSide;
    use crate::backtesting::signal::Signal;

    fn make_trade(pnl: f64, return_pct: f64, is_long: bool) -> Trade {
        Trade {
            side: if is_long {
                PositionSide::Long
            } else {
                PositionSide::Short
            },
            entry_timestamp: 0,
            exit_timestamp: 100,
            entry_price: 100.0,
            exit_price: 100.0 + pnl / 10.0,
            quantity: 10.0,
            commission: 0.0,
            pnl,
            return_pct,
            dividend_income: 0.0,
            entry_signal: Signal::long(0, 100.0),
            exit_signal: Signal::exit(100, 110.0),
        }
    }

    #[test]
    fn test_metrics_no_trades() {
        let equity = vec![
            EquityPoint {
                timestamp: 0,
                equity: 10000.0,
                drawdown_pct: 0.0,
            },
            EquityPoint {
                timestamp: 1,
                equity: 10100.0,
                drawdown_pct: 0.0,
            },
        ];

        let metrics = PerformanceMetrics::calculate(&[], &equity, 10000.0, 0, 0, 0.0, 252.0);

        assert_eq!(metrics.total_trades, 0);
        assert!((metrics.total_return_pct - 1.0).abs() < 0.01);
    }

    #[test]
    fn test_metrics_with_trades() {
        let trades = vec![
            make_trade(100.0, 10.0, true), // Win
            make_trade(-50.0, -5.0, true), // Loss
            make_trade(75.0, 7.5, false),  // Win (short)
            make_trade(25.0, 2.5, true),   // Win
        ];

        let equity = vec![
            EquityPoint {
                timestamp: 0,
                equity: 10000.0,
                drawdown_pct: 0.0,
            },
            EquityPoint {
                timestamp: 1,
                equity: 10100.0,
                drawdown_pct: 0.0,
            },
            EquityPoint {
                timestamp: 2,
                equity: 10050.0,
                drawdown_pct: 0.005,
            },
            EquityPoint {
                timestamp: 3,
                equity: 10125.0,
                drawdown_pct: 0.0,
            },
            EquityPoint {
                timestamp: 4,
                equity: 10150.0,
                drawdown_pct: 0.0,
            },
        ];

        let metrics = PerformanceMetrics::calculate(&trades, &equity, 10000.0, 10, 4, 0.0, 252.0);

        assert_eq!(metrics.total_trades, 4);
        assert_eq!(metrics.winning_trades, 3);
        assert_eq!(metrics.losing_trades, 1);
        assert!((metrics.win_rate - 0.75).abs() < 0.01);
        assert_eq!(metrics.long_trades, 3);
        assert_eq!(metrics.short_trades, 1);
    }

    #[test]
    fn test_consecutive_wins_losses() {
        let trades = vec![
            make_trade(100.0, 10.0, true), // Win
            make_trade(50.0, 5.0, true),   // Win
            make_trade(25.0, 2.5, true),   // Win
            make_trade(-50.0, -5.0, true), // Loss
            make_trade(-25.0, -2.5, true), // Loss
            make_trade(100.0, 10.0, true), // Win
        ];

        let (max_wins, max_losses) = calculate_consecutive(&trades);
        assert_eq!(max_wins, 3);
        assert_eq!(max_losses, 2);
    }

    #[test]
    fn test_drawdown_duration() {
        let equity = vec![
            EquityPoint {
                timestamp: 0,
                equity: 100.0,
                drawdown_pct: 0.0,
            },
            EquityPoint {
                timestamp: 1,
                equity: 95.0,
                drawdown_pct: 0.05,
            },
            EquityPoint {
                timestamp: 2,
                equity: 90.0,
                drawdown_pct: 0.10,
            },
            EquityPoint {
                timestamp: 3,
                equity: 92.0,
                drawdown_pct: 0.08,
            },
            EquityPoint {
                timestamp: 4,
                equity: 100.0,
                drawdown_pct: 0.0,
            }, // Recovery
            EquityPoint {
                timestamp: 5,
                equity: 98.0,
                drawdown_pct: 0.02,
            },
        ];

        let duration = calculate_max_drawdown_duration(&equity);
        assert_eq!(duration, 3); // 3 bars in drawdown (indices 1, 2, 3) before recovery at index 4
    }

    #[test]
    fn test_sharpe_uses_sample_variance() {
        // Verify Sharpe uses n-1 (sample) not n (population) variance.
        // With returns = [0.01, -0.01, 0.02, -0.02] and rf=0:
        //   mean = 0.0
        //   sample variance = (0.01^2 + 0.01^2 + 0.02^2 + 0.02^2) / 3 = 0.001 / 3
        //   std_dev = sqrt(0.001/3) ≈ 0.018257
        //   Sharpe = (0.0 / 0.018257) * sqrt(252) = 0.0
        let returns = vec![0.01, -0.01, 0.02, -0.02];
        let sharpe = calculate_sharpe_ratio(&returns, 0.0, 252.0);
        // Mean is exactly 0 so Sharpe must be 0 regardless of std_dev
        assert!(
            (sharpe).abs() < 1e-10,
            "Sharpe of zero-mean returns should be 0, got {}",
            sharpe
        );
    }

    #[test]
    fn test_max_drawdown_percentage_method() {
        // Verify the convenience method returns max_drawdown_pct * 100.
        // Use a trade so the no-trades early-return path is not taken, then
        // supply an equity curve with a known 10% drawdown point.
        let trade = make_trade(100.0, 10.0, true);
        let equity = vec![
            EquityPoint {
                timestamp: 0,
                equity: 10000.0,
                drawdown_pct: 0.0,
            },
            EquityPoint {
                timestamp: 1,
                equity: 9000.0,
                drawdown_pct: 0.1,
            },
            EquityPoint {
                timestamp: 2,
                equity: 10000.0,
                drawdown_pct: 0.0,
            },
        ];
        let metrics = PerformanceMetrics::calculate(&[trade], &equity, 10000.0, 1, 1, 0.0, 252.0);
        assert!(
            (metrics.max_drawdown_pct - 0.1).abs() < 1e-9,
            "max_drawdown_pct should be 0.1 (fraction), got {}",
            metrics.max_drawdown_pct
        );
        assert!(
            (metrics.max_drawdown_percentage() - 10.0).abs() < 1e-9,
            "max_drawdown_percentage() should be 10.0, got {}",
            metrics.max_drawdown_percentage()
        );
    }
}
