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

    /// Maximum drawdown percentage (0.0-1.0)
    pub max_drawdown_pct: f64,

    /// Maximum drawdown duration in bars
    pub max_drawdown_duration: i64,

    /// Win rate (profitable trades / total trades)
    pub win_rate: f64,

    /// Profit factor (gross profit / gross loss)
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

    /// Number of winning trades
    pub winning_trades: usize,

    /// Number of losing trades
    pub losing_trades: usize,

    /// Largest winning trade P&L
    pub largest_win: f64,

    /// Largest losing trade P&L
    pub largest_loss: f64,

    /// Maximum consecutive wins
    pub max_consecutive_wins: usize,

    /// Maximum consecutive losses
    pub max_consecutive_losses: usize,

    /// Calmar ratio (annualized return / max drawdown)
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
}

impl PerformanceMetrics {
    /// Calculate performance metrics from trades and equity curve
    pub fn calculate(
        trades: &[Trade],
        equity_curve: &[EquityPoint],
        initial_capital: f64,
        total_signals: usize,
        executed_signals: usize,
    ) -> Self {
        let total_trades = trades.len();

        if total_trades == 0 {
            let final_equity = equity_curve
                .last()
                .map(|e| e.equity)
                .unwrap_or(initial_capital);
            let total_return_pct = ((final_equity / initial_capital) - 1.0) * 100.0;

            return Self {
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
            };
        }

        // Basic trade stats
        let winning_trades = trades.iter().filter(|t| t.is_profitable()).count();
        let losing_trades = trades.iter().filter(|t| t.is_loss()).count();
        let long_trades = trades.iter().filter(|t| t.is_long()).count();
        let short_trades = trades.iter().filter(|t| t.is_short()).count();

        let win_rate = winning_trades as f64 / total_trades as f64;

        // P&L calculations
        let gross_profit: f64 = trades.iter().filter(|t| t.pnl > 0.0).map(|t| t.pnl).sum();
        let gross_loss: f64 = trades
            .iter()
            .filter(|t| t.pnl < 0.0)
            .map(|t| t.pnl.abs())
            .sum();

        let profit_factor = if gross_loss > 0.0 {
            gross_profit / gross_loss
        } else if gross_profit > 0.0 {
            f64::INFINITY
        } else {
            0.0
        };

        // Average returns
        let avg_trade_return_pct =
            trades.iter().map(|t| t.return_pct).sum::<f64>() / total_trades as f64;

        let winning_returns: Vec<f64> = trades
            .iter()
            .filter(|t| t.is_profitable())
            .map(|t| t.return_pct)
            .collect();
        let losing_returns: Vec<f64> = trades
            .iter()
            .filter(|t| t.is_loss())
            .map(|t| t.return_pct)
            .collect();

        let avg_win_pct = if !winning_returns.is_empty() {
            winning_returns.iter().sum::<f64>() / winning_returns.len() as f64
        } else {
            0.0
        };

        let avg_loss_pct = if !losing_returns.is_empty() {
            losing_returns.iter().sum::<f64>() / losing_returns.len() as f64
        } else {
            0.0
        };

        // Trade durations
        let total_duration: i64 = trades.iter().map(|t| t.duration_secs()).sum();
        let avg_trade_duration = total_duration as f64 / total_trades as f64;

        // Largest trades
        let largest_win = trades.iter().map(|t| t.pnl).fold(0.0, f64::max);
        let largest_loss = trades.iter().map(|t| t.pnl).fold(0.0, f64::min);

        // Consecutive wins/losses
        let (max_consecutive_wins, max_consecutive_losses) = calculate_consecutive(trades);

        // Total commission
        let total_commission: f64 = trades.iter().map(|t| t.commission).sum();

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

        // Annualized return (assuming daily bars and 252 trading days)
        let num_bars = equity_curve.len();
        let years = num_bars as f64 / 252.0;
        let annualized_return_pct = if years > 0.0 {
            ((final_equity / initial_capital).powf(1.0 / years) - 1.0) * 100.0
        } else {
            0.0
        };

        // Sharpe and Sortino ratios
        let returns: Vec<f64> = calculate_periodic_returns(equity_curve);
        let sharpe_ratio = calculate_sharpe_ratio(&returns);
        let sortino_ratio = calculate_sortino_ratio(&returns);

        // Calmar ratio
        let calmar_ratio = if max_drawdown_pct > 0.0 {
            annualized_return_pct / (max_drawdown_pct * 100.0)
        } else if annualized_return_pct > 0.0 {
            f64::INFINITY
        } else {
            0.0
        };

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
            winning_trades,
            losing_trades,
            largest_win,
            largest_loss,
            max_consecutive_wins,
            max_consecutive_losses,
            calmar_ratio,
            total_commission,
            long_trades,
            short_trades,
            total_signals,
            executed_signals,
        }
    }
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

/// Calculate Sharpe ratio (assuming risk-free rate = 0)
fn calculate_sharpe_ratio(returns: &[f64]) -> f64 {
    if returns.is_empty() {
        return 0.0;
    }

    let mean = returns.iter().sum::<f64>() / returns.len() as f64;
    let variance = returns.iter().map(|r| (r - mean).powi(2)).sum::<f64>() / returns.len() as f64;
    let std_dev = variance.sqrt();

    if std_dev > 0.0 {
        // Annualize: assume daily returns, 252 trading days
        (mean / std_dev) * (252.0_f64).sqrt()
    } else if mean > 0.0 {
        f64::INFINITY
    } else {
        0.0
    }
}

/// Calculate Sortino ratio (downside deviation only)
fn calculate_sortino_ratio(returns: &[f64]) -> f64 {
    if returns.is_empty() {
        return 0.0;
    }

    let mean = returns.iter().sum::<f64>() / returns.len() as f64;

    // Only consider negative returns for downside deviation
    let downside_returns: Vec<f64> = returns.iter().filter(|&&r| r < 0.0).copied().collect();

    if downside_returns.is_empty() {
        return if mean > 0.0 { f64::INFINITY } else { 0.0 };
    }

    let downside_variance =
        downside_returns.iter().map(|r| r.powi(2)).sum::<f64>() / returns.len() as f64;
    let downside_dev = downside_variance.sqrt();

    if downside_dev > 0.0 {
        (mean / downside_dev) * (252.0_f64).sqrt()
    } else if mean > 0.0 {
        f64::INFINITY
    } else {
        0.0
    }
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

        let metrics = PerformanceMetrics::calculate(&[], &equity, 10000.0, 0, 0);

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

        let metrics = PerformanceMetrics::calculate(&trades, &equity, 10000.0, 10, 4);

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
}
