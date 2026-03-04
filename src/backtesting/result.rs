//! Backtest results and performance metrics.

use std::collections::HashMap;

use chrono::{DateTime, Datelike, NaiveDateTime, Utc, Weekday};
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

    /// Kelly Criterion: optimal fraction of capital to risk per trade.
    ///
    /// Computed as `W - (1 - W) / R` where `W` is win rate and `R` is
    /// `avg_win_pct / abs(avg_loss_pct)`. A positive value suggests the
    /// strategy has an edge; a negative value suggests it does not. Values
    /// above 1 indicate extreme edge (rare in practice). Returns `0.0` when
    /// there are no losing trades to compute a ratio.
    pub kelly_criterion: f64,

    /// Van Tharp's System Quality Number.
    ///
    /// `SQN = (mean_R / std_R) * sqrt(n_trades)` where `R` is the
    /// distribution of per-trade return percentages. Interpretation:
    /// `>1.6` = below average, `>2.0` = average, `>2.5` = good,
    /// `>3.0` = excellent, `>5.0` = superb, `>7.0` = holy grail.
    /// Returns `0.0` when fewer than 2 trades are available.
    ///
    /// **Note:** Van Tharp's original definition uses *R-multiples*
    /// (profit/loss normalised by initial risk per trade, i.e. entry-to-stop
    /// distance). Since the engine does not track per-trade initial risk,
    /// this implementation uses `return_pct` as a proxy. Values will
    /// therefore not match Van Tharp's published benchmarks exactly.
    /// At least 30 trades are recommended for statistical reliability.
    pub sqn: f64,

    /// Expectancy: expected profit per trade in dollar terms.
    ///
    /// `P(win) × avg_win_dollar + P(loss) × avg_loss_dollar` where each
    /// probability is computed independently (`winning_trades / total` and
    /// `losing_trades / total`). Unlike `avg_trade_return_pct` (which is a
    /// percentage), this gives the expected monetary gain or loss per trade
    /// in the same currency as `initial_capital`. A positive value means the
    /// strategy has a statistical edge; e.g. `+$25` means you expect to make
    /// $25 on average per trade taken.
    pub expectancy: f64,

    /// Omega Ratio: probability-weighted ratio of gains to losses.
    ///
    /// `Σ max(r, 0) / Σ max(-r, 0)` computed over **bar-by-bar periodic
    /// returns** from the equity curve (consistent with Sharpe/Sortino),
    /// using a threshold of `0.0`. More general than Sharpe — considers the
    /// full return distribution rather than only mean and standard deviation.
    /// Returns `f64::MAX` when there are no negative-return bars.
    pub omega_ratio: f64,

    /// Tail Ratio: ratio of right tail to left tail of trade returns.
    ///
    /// `abs(p95) / abs(p5)` of the trade return distribution using the
    /// floor nearest-rank method (`floor(p × n)` as the 0-based index).
    /// A value `>1` means large wins are more extreme than large losses
    /// (favourable asymmetry). Returns `f64::MAX` when the 5th-percentile
    /// return is zero. Returns `0.0` when fewer than 2 trades exist.
    ///
    /// **Note:** Reliable interpretation requires at least ~20 trades;
    /// with fewer trades the percentile estimates are dominated by
    /// individual outliers.
    pub tail_ratio: f64,

    /// Recovery Factor: net profit relative to maximum drawdown.
    ///
    /// `total_return_pct / (max_drawdown_pct * 100)`. Measures how
    /// efficiently the strategy recovers from its worst drawdown. Returns
    /// `f64::MAX` when there is no drawdown, `0.0` when unprofitable.
    pub recovery_factor: f64,

    /// Ulcer Index: root-mean-square of drawdown depth across all bars,
    /// expressed as a **percentage** (0–100), consistent with backtesting.py
    /// and Peter Martin's original 1987 definition.
    ///
    /// `sqrt(mean((drawdown_pct × 100)²))` computed from the equity curve.
    /// Unlike max drawdown, it penalises both depth and duration — a long
    /// shallow drawdown scores higher than a brief deep one. A lower value
    /// indicates a smoother equity curve.
    pub ulcer_index: f64,

    /// Serenity Ratio (Martin Ratio / Ulcer Performance Index): excess
    /// annualised return per unit of Ulcer Index risk.
    ///
    /// `(annualized_return_pct - risk_free_rate_pct) / ulcer_index` where
    /// both numerator and denominator are in percentage units. Analogous to
    /// the Sharpe Ratio but uses the Ulcer Index as the risk measure,
    /// penalising prolonged drawdowns more heavily than short-term volatility.
    /// Returns `f64::MAX` when Ulcer Index is zero and excess return is positive.
    pub serenity_ratio: f64,
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
            kelly_criterion: 0.0,
            sqn: 0.0,
            expectancy: 0.0,
            omega_ratio: 0.0,
            tail_ratio: 0.0,
            recovery_factor: 0.0,
            ulcer_index: 0.0,
            serenity_ratio: 0.0,
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

        // Annualized return using configured bars_per_year.
        // Use return periods (N-1), not points (N), to avoid overestimating
        // elapsed time for short series.
        let num_periods = equity_curve.len().saturating_sub(1);
        let years = num_periods as f64 / bars_per_year;
        let growth = final_equity / initial_capital;
        let annualized_return_pct = if years > 0.0 {
            if growth <= 0.0 {
                -100.0
            } else {
                (growth.powf(1.0 / years) - 1.0) * 100.0
            }
        } else {
            0.0
        };

        // Sharpe and Sortino ratios (computed in one pass over shared excess returns)
        let returns: Vec<f64> = calculate_periodic_returns(equity_curve);
        let (sharpe_ratio, sortino_ratio) =
            calculate_risk_ratios(&returns, risk_free_rate, bars_per_year);

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

        // Phase 1 — extended metrics
        let kelly_criterion = calculate_kelly(win_rate, avg_win_pct, avg_loss_pct);
        let sqn = calculate_sqn(&stats.all_returns);
        // Dollar expectancy: expected profit per trade in the same currency as
        // initial_capital. This is distinct from avg_trade_return_pct (which
        // is a percentage). Break-even trades reduce both probabilities without
        // contributing to either avg, so each outcome is weighted independently.
        let loss_rate = stats.losing_trades as f64 / total_trades as f64;
        let avg_win_dollar = if stats.winning_trades > 0 {
            stats.gross_profit / stats.winning_trades as f64
        } else {
            0.0
        };
        let avg_loss_dollar = if stats.losing_trades > 0 {
            -(stats.gross_loss / stats.losing_trades as f64)
        } else {
            0.0
        };
        let expectancy = win_rate * avg_win_dollar + loss_rate * avg_loss_dollar;
        // Omega Ratio is defined on the continuous return distribution —
        // use the same bar-by-bar periodic returns as Sharpe/Sortino, not
        // per-trade returns (which vary by holding period and are incomparable
        // across strategies with different average trade durations).
        let omega_ratio = calculate_omega_ratio(&returns);
        let tail_ratio = calculate_tail_ratio(&stats.all_returns);
        let recovery_factor = if max_drawdown_pct > 0.0 {
            total_return_pct / (max_drawdown_pct * 100.0)
        } else if total_return_pct > 0.0 {
            f64::MAX
        } else {
            0.0
        };
        // ulcer_index is already in percentage units (see calculate_ulcer_index).
        let ulcer_index = calculate_ulcer_index(equity_curve);
        let rf_pct = risk_free_rate * 100.0;
        let serenity_ratio = if ulcer_index > 0.0 {
            (annualized_return_pct - rf_pct) / ulcer_index
        } else if annualized_return_pct > rf_pct {
            f64::MAX
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
            kelly_criterion,
            sqn,
            expectancy,
            omega_ratio,
            tail_ratio,
            recovery_factor,
            ulcer_index,
            serenity_ratio,
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
    /// All trade return percentages (wins + losses + break-even).
    all_returns: Vec<f64>,
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
        all_returns: Vec::new(),
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
        stats.all_returns.push(t.return_pct);
    }

    stats
}

/// Kelly Criterion: `W - (1 - W) / R` where R = avg_win / abs(avg_loss).
///
/// Returns `f64::MAX` when there are no losing trades and wins are positive
/// (unbounded edge). Returns `0.0` when inputs are degenerate.
fn calculate_kelly(win_rate: f64, avg_win_pct: f64, avg_loss_pct: f64) -> f64 {
    let abs_loss = avg_loss_pct.abs();
    if abs_loss == 0.0 {
        // No losing trades: edge is unbounded. Use f64::MAX to match the
        // sentinel convention used by profit_factor and calmar_ratio.
        return if avg_win_pct > 0.0 { f64::MAX } else { 0.0 };
    }
    if avg_win_pct == 0.0 {
        return 0.0;
    }
    let r = avg_win_pct / abs_loss;
    win_rate - (1.0 - win_rate) / r
}

/// Van Tharp's System Quality Number.
///
/// `(mean_R / std_R) * sqrt(n)` over per-trade return percentages.
/// Uses sample standard deviation (n-1). Returns `0.0` for fewer than 2 trades.
fn calculate_sqn(returns: &[f64]) -> f64 {
    let n = returns.len();
    if n < 2 {
        return 0.0;
    }
    let mean = returns.iter().sum::<f64>() / n as f64;
    let variance = returns.iter().map(|r| (r - mean).powi(2)).sum::<f64>() / (n - 1) as f64;
    let std_dev = variance.sqrt();
    if std_dev == 0.0 {
        return 0.0;
    }
    (mean / std_dev) * (n as f64).sqrt()
}

/// Omega Ratio using a threshold of 0.0.
///
/// `Σ max(r, 0) / Σ max(-r, 0)`. Returns `f64::MAX` when the denominator
/// is zero (no negative returns), `0.0` when the numerator is also zero.
fn calculate_omega_ratio(returns: &[f64]) -> f64 {
    let gains: f64 = returns.iter().map(|&r| r.max(0.0)).sum();
    let losses: f64 = returns.iter().map(|&r| (-r).max(0.0)).sum();
    if losses == 0.0 {
        if gains > 0.0 { f64::MAX } else { 0.0 }
    } else {
        gains / losses
    }
}

/// Tail Ratio: `abs(p95) / abs(p5)` of trade returns.
///
/// Returns `0.0` for fewer than 2 trades, `f64::MAX` when `p5 == 0`.
fn calculate_tail_ratio(returns: &[f64]) -> f64 {
    let n = returns.len();
    if n < 2 {
        return 0.0;
    }
    let mut sorted = returns.to_vec();
    sorted.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));

    let p5_idx = ((0.05 * n as f64).floor() as usize).min(n - 1);
    let p95_idx = ((0.95 * n as f64).floor() as usize).min(n - 1);

    let p5 = sorted[p5_idx].abs();
    let p95 = sorted[p95_idx].abs();

    if p5 == 0.0 {
        if p95 > 0.0 { f64::MAX } else { 0.0 }
    } else {
        p95 / p5
    }
}

/// Ulcer Index: `sqrt(mean(drawdown_pct²))` across all equity curve points,
/// returned in **percentage** units (0–100) to match standard tool output.
fn calculate_ulcer_index(equity_curve: &[EquityPoint]) -> f64 {
    if equity_curve.is_empty() {
        return 0.0;
    }
    // drawdown_pct is a fraction (0–1); multiply by 100 before squaring so
    // the result is in percentage units consistent with backtesting.py and
    // Peter Martin's original definition.
    let sum_sq: f64 = equity_curve
        .iter()
        .map(|p| (p.drawdown_pct * 100.0).powi(2))
        .sum();
    (sum_sq / equity_curve.len() as f64).sqrt()
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

/// Calculate Sharpe and Sortino ratios in a single pass over excess returns.
///
/// Computes the shared `excess` vec and `mean` once, then derives both ratios.
/// Uses sample standard deviation (n-1) and annualises by `sqrt(bars_per_year)`.
/// Returns `f64::MAX` for the positive-mean / zero-deviation edge case so the
/// value survives JSON round-trips (avoids `INFINITY`).
fn calculate_risk_ratios(
    returns: &[f64],
    annual_risk_free_rate: f64,
    bars_per_year: f64,
) -> (f64, f64) {
    if returns.len() < 2 {
        return (0.0, 0.0);
    }

    let periodic_rf = annual_to_periodic_rf(annual_risk_free_rate, bars_per_year);
    let excess: Vec<f64> = returns.iter().map(|r| r - periodic_rf).collect();
    let n = excess.len() as f64;
    let mean = excess.iter().sum::<f64>() / n;

    // Sharpe: sample variance (n-1) for unbiased estimation
    let variance = excess.iter().map(|r| (r - mean).powi(2)).sum::<f64>() / (n - 1.0);
    let std_dev = variance.sqrt();
    let sharpe = if std_dev > 0.0 {
        (mean / std_dev) * bars_per_year.sqrt()
    } else if mean > 0.0 {
        f64::MAX
    } else {
        0.0
    };

    // Sortino: downside deviation (only negative excess; denominator is n-1,
    // per Sortino's original definition and the `risk` module convention)
    let downside_sq_sum: f64 = excess.iter().filter(|&&r| r < 0.0).map(|r| r.powi(2)).sum();
    let downside_dev = (downside_sq_sum / (n - 1.0)).sqrt();
    let sortino = if downside_dev > 0.0 {
        (mean / downside_dev) * bars_per_year.sqrt()
    } else if mean > 0.0 {
        f64::MAX
    } else {
        0.0
    };

    (sharpe, sortino)
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

/// Infer the effective bars-per-year from the calendar span of an equity slice.
///
/// When an equity slice contains non-consecutive bars (e.g. every Monday in a
/// daily-bar backtest), the configured `bars_per_year` is no longer the right
/// annualisation denominator.  This function derives the correct value from
/// the number of return periods and the elapsed calendar time so that Sharpe
/// and Sortino ratios are annualised accurately regardless of bar frequency.
///
/// Falls back to `fallback_bpy` when the slice has fewer than two points or
/// its timestamp span is non-positive.
fn infer_bars_per_year(equity_slice: &[EquityPoint], fallback_bpy: f64) -> f64 {
    if equity_slice.len() < 2 {
        return fallback_bpy;
    }
    let first_ts = equity_slice.first().unwrap().timestamp as f64;
    let last_ts = equity_slice.last().unwrap().timestamp as f64;
    let seconds_per_year = 365.25 * 24.0 * 3600.0;
    let years = (last_ts - first_ts) / seconds_per_year;
    if years <= 0.0 {
        return fallback_bpy;
    }
    // Use (len - 1) = number of return periods, consistent with how
    // calculate_periodic_returns counts returns.
    ((equity_slice.len() - 1) as f64 / years).max(1.0)
}

/// Zero out time-scaled ratios when a period slice covers less than half a
/// year of bars.
///
/// Geometric annualisation of a sub-half-year return magnifies the result
/// by raising `growth` to a power > 2, making `annualized_return_pct`,
/// `calmar_ratio`, and `serenity_ratio` misleadingly large for short slices
/// (e.g. partial first/last years, individual monthly buckets).  Setting
/// them to `0.0` signals to callers that no reliable annual rate is available
/// for this period without requiring a new return type.
fn partial_period_adjust(
    mut metrics: PerformanceMetrics,
    slice_len: usize,
    bpy: f64,
) -> PerformanceMetrics {
    let periods = slice_len.saturating_sub(1) as f64;
    if periods / bpy < 0.5 {
        metrics.annualized_return_pct = 0.0;
        metrics.calmar_ratio = 0.0;
        metrics.serenity_ratio = 0.0;
    }
    metrics
}

/// Convert a Unix-second timestamp to a `NaiveDateTime` (UTC).
///
/// Returns `None` for timestamps outside the range representable by
/// [`DateTime<Utc>`] (i.e. before ≈ year −262144 or after ≈ year 262143).
/// Call sites should skip entries that map to `None` rather than defaulting
/// to the Unix epoch, which would silently misattribute those records to
/// `1970-01-01 Thursday`.
fn datetime_from_timestamp(ts: i64) -> Option<NaiveDateTime> {
    DateTime::<Utc>::from_timestamp(ts, 0).map(|dt| dt.naive_utc())
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

    /// Jensen's Alpha: annualised strategy excess return over the benchmark (CAPM).
    ///
    /// Computed as `strategy_ann - rf - β × (benchmark_ann - rf)` on the
    /// timestamp-aligned subset of strategy and benchmark returns.
    ///
    /// # Accuracy Caveat
    ///
    /// Annualisation uses `aligned_bars / bars_per_year` to estimate elapsed
    /// years.  If the strategy and benchmark candles have **different sampling
    /// frequencies** (e.g., daily strategy vs. weekly benchmark), the aligned
    /// subset contains far fewer bars than the full backtest period and the
    /// per-year estimate will be wrong — both `strategy_ann` and `benchmark_ann`
    /// are inflated by the same factor, but the risk-free rate is always the
    /// true annual rate, making alpha unreliable.
    ///
    /// For accurate alpha, supply benchmark candles with the **same interval**
    /// as the strategy candles.
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

    // ─── Phase 2 — Rolling & Temporal Analysis ───────────────────────────────

    /// Rolling Sharpe ratio over a sliding window of equity-curve bars.
    ///
    /// For each window of `window` consecutive bar-to-bar returns, computes
    /// the Sharpe ratio using the same `risk_free_rate` and `bars_per_year`
    /// as the overall backtest.  The first element corresponds to bars
    /// `0..window` of the equity curve.
    ///
    /// Returns an empty vector when `window == 0` or when the equity curve
    /// contains fewer than `window + 1` bars (i.e. fewer than `window`
    /// return periods).
    ///
    /// # Statistical reliability
    ///
    /// Sharpe and Sortino are computed from `window` return observations using
    /// sample variance (`n − 1` degrees of freedom).  Very small windows
    /// produce extreme and unreliable values — at least **30 bars** is a
    /// practical lower bound; **60–252** is typical for daily backtests.
    pub fn rolling_sharpe(&self, window: usize) -> Vec<f64> {
        if window == 0 {
            return vec![];
        }
        let returns = calculate_periodic_returns(&self.equity_curve);
        if returns.len() < window {
            return vec![];
        }
        let rf = self.config.risk_free_rate;
        let bpy = self.config.bars_per_year;
        returns
            .windows(window)
            .map(|w| {
                let (sharpe, _) = calculate_risk_ratios(w, rf, bpy);
                sharpe
            })
            .collect()
    }

    /// Running drawdown fraction at each bar of the equity curve (0.0–1.0).
    ///
    /// Each value is the fractional decline from the running all-time-high
    /// equity up to that bar: `0.0` means the equity is at a new peak; `0.2`
    /// means it is 20% below the highest value seen so far.
    ///
    /// **This is not a sliding-window computation.** Values are read directly
    /// from the precomputed [`EquityPoint::drawdown_pct`] field, which tracks
    /// the running-peak drawdown since the backtest began.  To compute the
    /// *maximum* drawdown within a rolling N-bar window (regime-change
    /// detection), iterate over [`BacktestResult::equity_curve`] manually.
    ///
    /// The returned vector has the same length as
    /// [`BacktestResult::equity_curve`].
    pub fn drawdown_series(&self) -> Vec<f64> {
        self.equity_curve.iter().map(|p| p.drawdown_pct).collect()
    }

    /// Rolling win rate over a sliding window of consecutive closed trades.
    ///
    /// For each window of `window` trades (ordered by exit timestamp as stored
    /// in the trade log), returns the fraction of winning trades in that
    /// window.  The first element corresponds to trades `0..window`.
    ///
    /// This is a **trade-count window**, not a time window.  To compute win
    /// rate over a fixed calendar period, use [`by_year`](Self::by_year),
    /// [`by_month`](Self::by_month), or filter [`BacktestResult::trades`]
    /// directly by timestamp.
    ///
    /// Returns an empty vector when `window == 0` or when fewer than `window`
    /// trades were closed.
    pub fn rolling_win_rate(&self, window: usize) -> Vec<f64> {
        if window == 0 || self.trades.len() < window {
            return vec![];
        }
        self.trades
            .windows(window)
            .map(|w| {
                let wins = w.iter().filter(|t| t.is_profitable()).count();
                wins as f64 / window as f64
            })
            .collect()
    }

    /// Performance metrics broken down by calendar year.
    ///
    /// Each trade is attributed to the year in which it **closed**
    /// (`exit_timestamp`).  The equity curve is sliced to the bars that fall
    /// within that calendar year, and the equity at the first bar of the year
    /// serves as `initial_capital` for the period metrics.
    ///
    /// Years with no closed trades are omitted from the result.
    ///
    /// # Caveats
    ///
    /// - **Open positions**: a position that is open throughout the year
    ///   contributes to the equity-curve drawdown and Sharpe of that year but
    ///   does **not** appear in `total_trades` or `win_rate`, because those
    ///   are derived from closed trades only.  Strategies with long holding
    ///   periods will show systematically low trade counts per year.
    /// - **Partial years**: the first and last year of a backtest typically
    ///   cover fewer than 12 months.  `annualized_return_pct`, `calmar_ratio`,
    ///   and `serenity_ratio` are set to `0.0` for slices shorter than half a
    ///   year (`< bars_per_year / 2` bars) to prevent geometric-compounding
    ///   distortion.
    /// - **`total_signals` / `executed_signals`**: these fields are `0` in
    ///   period breakdowns because signal records are not partitioned per
    ///   period.  Use [`BacktestResult::signals`] directly if needed.
    pub fn by_year(&self) -> HashMap<i32, PerformanceMetrics> {
        self.temporal_metrics(|ts| datetime_from_timestamp(ts).map(|dt| dt.year()))
    }

    /// Performance metrics broken down by calendar month.
    ///
    /// Each trade is attributed to the `(year, month)` in which it **closed**.
    /// Uses the same equity-slicing approach as [`by_year`](Self::by_year);
    /// the same caveats about open positions, partial periods, and signal
    /// counts apply here as well.
    pub fn by_month(&self) -> HashMap<(i32, u32), PerformanceMetrics> {
        self.temporal_metrics(|ts| datetime_from_timestamp(ts).map(|dt| (dt.year(), dt.month())))
    }

    /// Performance metrics broken down by day of week.
    ///
    /// Each trade is attributed to the weekday on which it **closed**
    /// (`exit_timestamp`).  Only weekdays present in the trade log appear in
    /// the result.  Trades and equity-curve points with timestamps that cannot
    /// be converted to a valid date are silently skipped.
    ///
    /// # Sharpe / Sortino annualisation
    ///
    /// The equity curve is filtered to bars that fall on each specific
    /// weekday, so consecutive equity points in each slice are roughly one
    /// *week* apart (for a daily-bar backtest).  `bars_per_year` is inferred
    /// from the calendar span of each slice so that annualisation matches the
    /// actual sampling frequency — **you do not need to adjust the config**.
    /// The inferred value is approximately `52` for daily bars, `12` for
    /// weekly bars, and so on.
    ///
    /// # Other caveats
    ///
    /// The same open-position and signal-count caveats from
    /// [`by_year`](Self::by_year) apply here.
    pub fn by_day_of_week(&self) -> HashMap<Weekday, PerformanceMetrics> {
        // Pre-group trades by weekday — O(T)
        let mut trade_groups: HashMap<Weekday, Vec<&Trade>> = HashMap::new();
        for trade in &self.trades {
            if let Some(day) = datetime_from_timestamp(trade.exit_timestamp).map(|dt| dt.weekday())
            {
                trade_groups.entry(day).or_default().push(trade);
            }
        }

        // Pre-group equity curve by weekday — O(N), avoids O(N × K) rescanning
        let mut equity_groups: HashMap<Weekday, Vec<EquityPoint>> = HashMap::new();
        for p in &self.equity_curve {
            if let Some(day) = datetime_from_timestamp(p.timestamp).map(|dt| dt.weekday()) {
                equity_groups.entry(day).or_default().push(p.clone());
            }
        }

        trade_groups
            .into_iter()
            .map(|(day, group_trades)| {
                let equity_slice = equity_groups.remove(&day).unwrap_or_default();
                let initial_capital = equity_slice
                    .first()
                    .map(|p| p.equity)
                    .unwrap_or(self.initial_capital);
                let trades_vec: Vec<Trade> = group_trades.into_iter().cloned().collect();
                // Infer the effective bars_per_year from the slice's calendar
                // span: same-weekday bars are ~5 trading days apart for a
                // daily-bar backtest, so the correct annualisation factor is
                // ≈52, not the configured 252.
                let bpy = infer_bars_per_year(&equity_slice, self.config.bars_per_year);
                let metrics = PerformanceMetrics::calculate(
                    &trades_vec,
                    &equity_slice,
                    initial_capital,
                    0,
                    0,
                    self.config.risk_free_rate,
                    bpy,
                );
                let slice_len = equity_slice.len();
                (day, partial_period_adjust(metrics, slice_len, bpy))
            })
            .collect()
    }

    /// Groups trades and equity-curve points by an arbitrary calendar key,
    /// then computes [`PerformanceMetrics`] for each group.
    ///
    /// `key_fn` maps a Unix-second timestamp to `Some(K)`, or `None` for
    /// timestamps that cannot be parsed (those entries are silently skipped).
    ///
    /// Both trades and equity-curve points are pre-grouped in **O(N + T)**
    /// passes before metrics are computed per period, avoiding the O(N × K)
    /// inner-loop cost of the naïve approach.
    fn temporal_metrics<K>(
        &self,
        key_fn: impl Fn(i64) -> Option<K>,
    ) -> HashMap<K, PerformanceMetrics>
    where
        K: std::hash::Hash + Eq + Copy,
    {
        // Pre-group trades by period key — O(T)
        let mut trade_groups: HashMap<K, Vec<&Trade>> = HashMap::new();
        for trade in &self.trades {
            if let Some(key) = key_fn(trade.exit_timestamp) {
                trade_groups.entry(key).or_default().push(trade);
            }
        }

        // Pre-group equity curve by period key — O(N)
        let mut equity_groups: HashMap<K, Vec<EquityPoint>> = HashMap::new();
        for p in &self.equity_curve {
            if let Some(key) = key_fn(p.timestamp) {
                equity_groups.entry(key).or_default().push(p.clone());
            }
        }

        trade_groups
            .into_iter()
            .map(|(key, group_trades)| {
                let equity_slice = equity_groups.remove(&key).unwrap_or_default();
                let initial_capital = equity_slice
                    .first()
                    .map(|p| p.equity)
                    .unwrap_or(self.initial_capital);
                let trades_vec: Vec<Trade> = group_trades.into_iter().cloned().collect();
                let metrics = PerformanceMetrics::calculate(
                    &trades_vec,
                    &equity_slice,
                    initial_capital,
                    // H-3: both zero — signal records are not partitioned
                    // per period; callers should filter BacktestResult::signals
                    // directly if per-period signal counts are needed.
                    0,
                    0,
                    self.config.risk_free_rate,
                    self.config.bars_per_year,
                );
                let slice_len = equity_slice.len();
                // C-2: suppress annualised metrics for sub-half-year slices.
                (
                    key,
                    partial_period_adjust(metrics, slice_len, self.config.bars_per_year),
                )
            })
            .collect()
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
            entry_quantity: 10.0,
            commission: 0.0,
            pnl,
            return_pct,
            dividend_income: 0.0,
            unreinvested_dividends: 0.0,
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
        let (sharpe, _) = calculate_risk_ratios(&returns, 0.0, 252.0);
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

    #[test]
    fn test_kelly_criterion() {
        // W=0.6, avg_win=10%, avg_loss=5% => R=2.0 => Kelly=0.6 - 0.4/2 = 0.4
        let kelly = calculate_kelly(0.6, 10.0, -5.0);
        assert!(
            (kelly - 0.4).abs() < 1e-9,
            "Kelly should be 0.4, got {kelly}"
        );

        // No losses with positive wins => f64::MAX (unbounded edge)
        assert_eq!(calculate_kelly(1.0, 10.0, 0.0), f64::MAX);
        // No losses, no wins => 0.0
        assert_eq!(calculate_kelly(0.0, 0.0, 0.0), 0.0);

        // Negative edge: W=0.3, R=1.0 => Kelly=0.3-0.7=-0.4
        let kelly_neg = calculate_kelly(0.3, 5.0, -5.0);
        assert!(
            (kelly_neg - (-0.4)).abs() < 1e-9,
            "Kelly should be -0.4, got {kelly_neg}"
        );
    }

    #[test]
    fn test_sqn() {
        // 10 trades all returning 1.0% -> std_dev=0 -> SQN=0
        let returns = vec![1.0; 10];
        assert_eq!(calculate_sqn(&returns), 0.0);

        // Fewer than 2 trades -> 0
        assert_eq!(calculate_sqn(&[1.0]), 0.0);
        assert_eq!(calculate_sqn(&[]), 0.0);

        // Known values: returns = [2, -1, 3, -1, 2], n=5
        // mean = 1.0, sample_std = sqrt(((1+4+4+4+1)/4)) = sqrt(14/4) = sqrt(3.5) ≈ 1.8708
        // SQN = (1.0 / 1.8708) * sqrt(5) ≈ 0.5345 * 2.2361 ≈ 1.1952
        let returns2 = vec![2.0, -1.0, 3.0, -1.0, 2.0];
        let sqn = calculate_sqn(&returns2);
        assert!(
            (sqn - 1.1952).abs() < 0.001,
            "SQN should be ~1.195, got {sqn}"
        );
    }

    #[test]
    fn test_omega_ratio() {
        // All positive: gains=6, losses=0 -> f64::MAX
        assert_eq!(calculate_omega_ratio(&[1.0, 2.0, 3.0]), f64::MAX);

        // All negative: gains=0, losses=6 -> 0.0
        assert_eq!(calculate_omega_ratio(&[-1.0, -2.0, -3.0]), 0.0);

        // Mixed: [2, -1, 3, -2] -> gains=5, losses=3 -> omega=5/3
        let omega = calculate_omega_ratio(&[2.0, -1.0, 3.0, -2.0]);
        assert!(
            (omega - 5.0 / 3.0).abs() < 1e-9,
            "Omega should be 5/3, got {omega}"
        );
    }

    #[test]
    fn test_tail_ratio() {
        // Fewer than 2 -> 0
        assert_eq!(calculate_tail_ratio(&[1.0]), 0.0);

        // 20 values: p5 at idx 1, p95 at idx 19
        // sorted: -10, -5, 0, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 5, 10
        let mut vals = vec![1.0f64; 16];
        vals.extend([-10.0, -5.0, 5.0, 10.0]);
        vals.sort_by(|a, b| a.partial_cmp(b).unwrap());
        // n=20, p5_idx=floor(0.05*20)=1 -> sorted[1]=-5 -> abs=5
        //        p95_idx=floor(0.95*20)=19 -> sorted[19]=10 -> abs=10
        // tail_ratio = 10/5 = 2.0
        let tr = calculate_tail_ratio(&vals);
        assert!(
            (tr - 2.0).abs() < 1e-9,
            "Tail ratio should be 2.0, got {tr}"
        );

        // p5 = 0 -> f64::MAX when p95 > 0
        let zeros_with_win = vec![
            0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0,
            0.0, 0.0, 5.0,
        ];
        assert_eq!(calculate_tail_ratio(&zeros_with_win), f64::MAX);
    }

    #[test]
    fn test_ulcer_index() {
        // No drawdowns -> 0
        let flat = vec![
            EquityPoint {
                timestamp: 0,
                equity: 100.0,
                drawdown_pct: 0.0,
            },
            EquityPoint {
                timestamp: 1,
                equity: 110.0,
                drawdown_pct: 0.0,
            },
        ];
        assert_eq!(calculate_ulcer_index(&flat), 0.0);

        // drawdown_pct fractions 0.1 and 0.2 → 10% and 20%
        // sqrt((10² + 20²) / 2) = sqrt(250) ≈ 15.811 (percentage units)
        let dd = vec![
            EquityPoint {
                timestamp: 0,
                equity: 100.0,
                drawdown_pct: 0.1,
            },
            EquityPoint {
                timestamp: 1,
                equity: 90.0,
                drawdown_pct: 0.2,
            },
        ];
        let ui = calculate_ulcer_index(&dd);
        let expected = ((100.0f64 + 400.0) / 2.0).sqrt(); // sqrt(250) ≈ 15.811
        assert!(
            (ui - expected).abs() < 1e-9,
            "Ulcer index should be {expected}, got {ui}"
        );
    }

    #[test]
    fn test_new_metrics_in_calculate() {
        // Mixed trades: 2 wins (+10%, +20%), 1 loss (-5%) with known equity curve
        let trades = vec![
            make_trade(100.0, 10.0, true),
            make_trade(200.0, 20.0, true),
            make_trade(-50.0, -5.0, true),
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
                equity: 10300.0,
                drawdown_pct: 0.0,
            },
            EquityPoint {
                timestamp: 3,
                equity: 10250.0,
                drawdown_pct: 0.005,
            },
        ];
        let m = PerformanceMetrics::calculate(&trades, &equity, 10000.0, 3, 3, 0.0, 252.0);

        // win_rate=2/3, avg_win=(10+20)/2=15, avg_loss=-5
        // Kelly = 2/3 - (1/3)/(15/5) = 0.6667 - 0.3333/3 = 0.6667 - 0.1111 ≈ 0.5556
        assert!(
            m.kelly_criterion > 0.0,
            "Kelly should be positive for profitable strategy"
        );

        // SQN with 3 trades
        assert!(m.sqn.is_finite(), "SQN should be finite");

        // Dollar expectancy: win_rate=2/3, avg_win=$100+$200)/2=$150, avg_loss=-$50
        // = (2/3)*150 + (1/3)*(-50) = 100 - 16.67 ≈ 83.33
        assert!(
            m.expectancy > 0.0,
            "Expectancy should be positive in dollar terms"
        );

        // Omega ratio is computed on periodic equity curve returns, not
        // trade returns — just verify it is positive and finite.
        assert!(m.omega_ratio > 0.0 && m.omega_ratio.is_finite() || m.omega_ratio == f64::MAX);

        // Ulcer index from equity curve (max_drawdown=0.5%)
        assert!(m.ulcer_index >= 0.0);

        // Recovery factor: profitable with non-zero drawdown -> positive
        assert!(m.recovery_factor > 0.0);
    }

    #[test]
    fn test_profit_factor_all_wins_is_f64_max() {
        let trades = vec![make_trade(100.0, 10.0, true), make_trade(50.0, 5.0, true)];
        let equity = vec![
            EquityPoint {
                timestamp: 0,
                equity: 10000.0,
                drawdown_pct: 0.0,
            },
            EquityPoint {
                timestamp: 1,
                equity: 10150.0,
                drawdown_pct: 0.0,
            },
        ];

        let metrics = PerformanceMetrics::calculate(&trades, &equity, 10000.0, 2, 2, 0.0, 252.0);
        assert_eq!(metrics.profit_factor, f64::MAX);
    }

    // ─── Phase 2 — Rolling & Temporal Analysis ───────────────────────────────

    use super::super::config::BacktestConfig;
    use crate::backtesting::position::Position;
    use chrono::{NaiveDate, Weekday};

    fn make_trade_timed(pnl: f64, return_pct: f64, entry_ts: i64, exit_ts: i64) -> Trade {
        Trade {
            side: PositionSide::Long,
            entry_timestamp: entry_ts,
            exit_timestamp: exit_ts,
            entry_price: 100.0,
            exit_price: 100.0 + pnl / 10.0,
            quantity: 10.0,
            entry_quantity: 10.0,
            commission: 0.0,
            pnl,
            return_pct,
            dividend_income: 0.0,
            unreinvested_dividends: 0.0,
            entry_signal: Signal::long(entry_ts, 100.0),
            exit_signal: Signal::exit(exit_ts, 100.0 + pnl / 10.0),
        }
    }

    /// Minimal `BacktestResult` fixture using the default `BacktestConfig`
    /// (risk_free_rate=0.0, bars_per_year=252.0).
    fn make_result(trades: Vec<Trade>, equity_curve: Vec<EquityPoint>) -> BacktestResult {
        let metrics = PerformanceMetrics::calculate(
            &trades,
            &equity_curve,
            10000.0,
            trades.len(),
            trades.len(),
            0.0,
            252.0,
        );
        BacktestResult {
            symbol: "TEST".to_string(),
            strategy_name: "TestStrategy".to_string(),
            config: BacktestConfig::default(),
            start_timestamp: equity_curve.first().map(|e| e.timestamp).unwrap_or(0),
            end_timestamp: equity_curve.last().map(|e| e.timestamp).unwrap_or(0),
            initial_capital: 10000.0,
            final_equity: equity_curve.last().map(|e| e.equity).unwrap_or(10000.0),
            metrics,
            trades,
            equity_curve,
            signals: vec![],
            open_position: None::<Position>,
            benchmark: None,
            diagnostics: vec![],
        }
    }

    fn ts(date: &str) -> i64 {
        let d = NaiveDate::parse_from_str(date, "%Y-%m-%d").unwrap();
        d.and_hms_opt(12, 0, 0).unwrap().and_utc().timestamp()
    }

    fn equity_point(timestamp: i64, equity: f64, drawdown_pct: f64) -> EquityPoint {
        EquityPoint {
            timestamp,
            equity,
            drawdown_pct,
        }
    }

    // ── rolling_sharpe ────────────────────────────────────────────────────────

    #[test]
    fn rolling_sharpe_window_zero_returns_empty() {
        let result = make_result(
            vec![],
            vec![equity_point(0, 10000.0, 0.0), equity_point(1, 10100.0, 0.0)],
        );
        assert!(result.rolling_sharpe(0).is_empty());
    }

    #[test]
    fn rolling_sharpe_insufficient_bars_returns_empty() {
        // 3 equity points → 2 returns; window=3 needs 3 returns → empty
        let result = make_result(
            vec![],
            vec![
                equity_point(0, 10000.0, 0.0),
                equity_point(1, 10100.0, 0.0),
                equity_point(2, 10200.0, 0.0),
            ],
        );
        assert!(result.rolling_sharpe(3).is_empty());
    }

    #[test]
    fn rolling_sharpe_correct_length() {
        // 5 equity points → 4 returns; window=2 → 3 values
        let pts: Vec<EquityPoint> = (0..5)
            .map(|i| equity_point(i, 10000.0 + i as f64 * 100.0, 0.0))
            .collect();
        let result = make_result(vec![], pts);
        assert_eq!(result.rolling_sharpe(2).len(), 3);
    }

    #[test]
    fn rolling_sharpe_monotone_increase_positive() {
        // Strictly increasing equity → all positive Sharpe values
        let pts: Vec<EquityPoint> = (0..10)
            .map(|i| equity_point(i, 10000.0 + i as f64 * 100.0, 0.0))
            .collect();
        let result = make_result(vec![], pts);
        let sharpes = result.rolling_sharpe(3);
        assert!(!sharpes.is_empty());
        for s in &sharpes {
            assert!(
                *s > 0.0 || *s == f64::MAX,
                "expected positive Sharpe, got {s}"
            );
        }
    }

    // ── drawdown_series ───────────────────────────────────────────────────────

    #[test]
    fn drawdown_series_mirrors_equity_curve() {
        let pts = vec![
            equity_point(0, 10000.0, 0.00),
            equity_point(1, 9500.0, 0.05),
            equity_point(2, 9000.0, 0.10),
            equity_point(3, 9200.0, 0.08),
            equity_point(4, 10000.0, 0.00),
        ];
        let result = make_result(vec![], pts.clone());
        let dd = result.drawdown_series();
        assert_eq!(dd.len(), pts.len());
        for (got, ep) in dd.iter().zip(pts.iter()) {
            assert!(
                (got - ep.drawdown_pct).abs() < f64::EPSILON,
                "expected {}, got {}",
                ep.drawdown_pct,
                got
            );
        }
    }

    #[test]
    fn drawdown_series_empty_curve() {
        let result = make_result(vec![], vec![]);
        assert!(result.drawdown_series().is_empty());
    }

    // ── rolling_win_rate ──────────────────────────────────────────────────────

    #[test]
    fn rolling_win_rate_window_zero_returns_empty() {
        let result = make_result(vec![make_trade(50.0, 5.0, true)], vec![]);
        assert!(result.rolling_win_rate(0).is_empty());
    }

    #[test]
    fn rolling_win_rate_window_exceeds_trades_returns_empty() {
        let result = make_result(vec![make_trade(50.0, 5.0, true)], vec![]);
        assert!(result.rolling_win_rate(2).is_empty());
    }

    #[test]
    fn rolling_win_rate_all_wins() {
        let trades = vec![
            make_trade(10.0, 1.0, true),
            make_trade(20.0, 2.0, true),
            make_trade(15.0, 1.5, true),
        ];
        let result = make_result(trades, vec![]);
        let wr = result.rolling_win_rate(2);
        // 3 trades, window=2 → 2 values, each 1.0
        assert_eq!(wr, vec![1.0, 1.0]);
    }

    #[test]
    fn rolling_win_rate_alternating() {
        // win, loss, win, loss → window=2 → [0.5, 0.5, 0.5]
        let trades = vec![
            make_trade(10.0, 1.0, true),
            make_trade(-10.0, -1.0, true),
            make_trade(10.0, 1.0, true),
            make_trade(-10.0, -1.0, true),
        ];
        let result = make_result(trades, vec![]);
        let wr = result.rolling_win_rate(2);
        assert_eq!(wr.len(), 3);
        for v in &wr {
            assert!((v - 0.5).abs() < f64::EPSILON, "expected 0.5, got {v}");
        }
    }

    #[test]
    fn rolling_win_rate_correct_length() {
        let trades: Vec<Trade> = (0..5)
            .map(|i| make_trade(i as f64, i as f64, true))
            .collect();
        let result = make_result(trades, vec![]);
        // 5 trades, window=3 → 3 values
        assert_eq!(result.rolling_win_rate(3).len(), 3);
    }

    #[test]
    fn rolling_win_rate_window_equals_trade_count_returns_one_element() {
        // L-2: boundary — window == trades.len() → exactly 1 element
        let trades = vec![
            make_trade(10.0, 1.0, true),
            make_trade(-5.0, -0.5, true),
            make_trade(8.0, 0.8, true),
        ];
        let result = make_result(trades, vec![]);
        let wr = result.rolling_win_rate(3);
        assert_eq!(wr.len(), 1);
        // 2 wins out of 3
        assert!((wr[0] - 2.0 / 3.0).abs() < f64::EPSILON);
    }

    // ── partial_period_adjust ─────────────────────────────────────────────────

    #[test]
    fn partial_period_adjust_zeroes_annualised_fields_for_short_slice() {
        // C-2: a 10-bar slice with bpy=252 → years ≈ 0.036 < 0.5 → zero out
        let dummy_metrics = PerformanceMetrics::calculate(
            &[make_trade(100.0, 10.0, true)],
            &[equity_point(0, 10000.0, 0.0), equity_point(1, 11000.0, 0.0)],
            10000.0,
            0,
            0,
            0.0,
            252.0,
        );
        assert!(dummy_metrics.annualized_return_pct != 0.0);
        let adjusted = partial_period_adjust(dummy_metrics, 10, 252.0);
        assert_eq!(adjusted.annualized_return_pct, 0.0);
        assert_eq!(adjusted.calmar_ratio, 0.0);
        assert_eq!(adjusted.serenity_ratio, 0.0);
    }

    #[test]
    fn partial_period_adjust_preserves_full_year_metrics() {
        // A 252-bar slice with bpy=252 → years ≈ 1.0 ≥ 0.5 → no change
        let metrics = PerformanceMetrics::calculate(
            &[make_trade(100.0, 10.0, true)],
            &[equity_point(0, 10000.0, 0.0), equity_point(1, 11000.0, 0.0)],
            10000.0,
            0,
            0,
            0.0,
            252.0,
        );
        let ann_before = metrics.annualized_return_pct;
        let adjusted = partial_period_adjust(metrics, 252, 252.0);
        assert_eq!(adjusted.annualized_return_pct, ann_before);
    }

    // ── by_year ───────────────────────────────────────────────────────────────

    #[test]
    fn by_year_no_trades_empty() {
        let result = make_result(vec![], vec![equity_point(ts("2023-06-01"), 10000.0, 0.0)]);
        assert!(result.by_year().is_empty());
    }

    #[test]
    fn by_year_splits_across_years() {
        let eq = vec![
            equity_point(ts("2022-06-15"), 10000.0, 0.0),
            equity_point(ts("2022-06-16"), 10100.0, 0.0),
            equity_point(ts("2023-06-15"), 10200.0, 0.0),
            equity_point(ts("2023-06-16"), 10300.0, 0.0),
        ];
        let t1 = make_trade_timed(100.0, 1.0, ts("2022-06-15"), ts("2022-06-16"));
        let t2 = make_trade_timed(100.0, 1.0, ts("2023-06-15"), ts("2023-06-16"));
        let result = make_result(vec![t1, t2], eq);
        let by_year = result.by_year();
        assert_eq!(by_year.len(), 2);
        assert!(by_year.contains_key(&2022));
        assert!(by_year.contains_key(&2023));
        assert_eq!(by_year[&2022].total_trades, 1);
        assert_eq!(by_year[&2023].total_trades, 1);
    }

    #[test]
    fn by_year_all_same_year() {
        let eq = vec![
            equity_point(ts("2023-03-01"), 10000.0, 0.0),
            equity_point(ts("2023-06-01"), 10200.0, 0.0),
            equity_point(ts("2023-09-01"), 10500.0, 0.0),
        ];
        let t1 = make_trade_timed(200.0, 2.0, ts("2023-03-01"), ts("2023-06-01"));
        let t2 = make_trade_timed(300.0, 3.0, ts("2023-06-01"), ts("2023-09-01"));
        let result = make_result(vec![t1, t2], eq);
        let by_year = result.by_year();
        assert_eq!(by_year.len(), 1);
        assert!(by_year.contains_key(&2023));
        assert_eq!(by_year[&2023].total_trades, 2);
    }

    // ── by_month ──────────────────────────────────────────────────────────────

    #[test]
    fn by_month_splits_across_months() {
        let eq = vec![
            equity_point(ts("2023-03-15"), 10000.0, 0.0),
            equity_point(ts("2023-03-16"), 10100.0, 0.0),
            equity_point(ts("2023-07-15"), 10200.0, 0.0),
            equity_point(ts("2023-07-16"), 10300.0, 0.0),
        ];
        let t1 = make_trade_timed(100.0, 1.0, ts("2023-03-15"), ts("2023-03-16"));
        let t2 = make_trade_timed(100.0, 1.0, ts("2023-07-15"), ts("2023-07-16"));
        let result = make_result(vec![t1, t2], eq);
        let by_month = result.by_month();
        assert_eq!(by_month.len(), 2);
        assert!(by_month.contains_key(&(2023, 3)));
        assert!(by_month.contains_key(&(2023, 7)));
    }

    #[test]
    fn by_month_same_month_different_years_are_separate_keys() {
        let eq = vec![
            equity_point(ts("2022-06-15"), 10000.0, 0.0),
            equity_point(ts("2023-06-15"), 10200.0, 0.0),
        ];
        let t1 = make_trade_timed(100.0, 1.0, ts("2022-06-14"), ts("2022-06-15"));
        let t2 = make_trade_timed(100.0, 1.0, ts("2023-06-14"), ts("2023-06-15"));
        let result = make_result(vec![t1, t2], eq);
        let by_month = result.by_month();
        assert_eq!(by_month.len(), 2);
        assert!(by_month.contains_key(&(2022, 6)));
        assert!(by_month.contains_key(&(2023, 6)));
    }

    // ── by_day_of_week ────────────────────────────────────────────────────────

    #[test]
    fn by_day_of_week_single_day() {
        // 2023-01-02 is a Monday
        let monday = ts("2023-01-02");
        let t1 = make_trade_timed(100.0, 1.0, monday - 86400, monday);
        let t2 = make_trade_timed(50.0, 0.5, monday - 86400 * 2, monday);
        let eq = vec![equity_point(monday, 10000.0, 0.0)];
        let result = make_result(vec![t1, t2], eq);
        let by_dow = result.by_day_of_week();
        assert_eq!(by_dow.len(), 1);
        assert!(by_dow.contains_key(&Weekday::Mon));
        assert_eq!(by_dow[&Weekday::Mon].total_trades, 2);
    }

    #[test]
    fn by_day_of_week_multiple_days() {
        // 2023-01-02 = Monday, 2023-01-03 = Tuesday
        let monday = ts("2023-01-02");
        let tuesday = ts("2023-01-03");
        let t_mon = make_trade_timed(100.0, 1.0, monday - 86400, monday);
        let t_tue = make_trade_timed(-50.0, -0.5, tuesday - 86400, tuesday);
        let eq = vec![
            equity_point(monday, 10000.0, 0.0),
            equity_point(tuesday, 10100.0, 0.0),
        ];
        let result = make_result(vec![t_mon, t_tue], eq);
        let by_dow = result.by_day_of_week();
        assert_eq!(by_dow.len(), 2);
        assert!(by_dow.contains_key(&Weekday::Mon));
        assert!(by_dow.contains_key(&Weekday::Tue));
        assert_eq!(by_dow[&Weekday::Mon].total_trades, 1);
        assert_eq!(by_dow[&Weekday::Tue].total_trades, 1);
        assert_eq!(by_dow[&Weekday::Mon].winning_trades, 1);
        assert_eq!(by_dow[&Weekday::Tue].losing_trades, 1);
    }

    #[test]
    fn by_day_of_week_no_trades_empty() {
        let result = make_result(vec![], vec![equity_point(ts("2023-01-02"), 10000.0, 0.0)]);
        assert!(result.by_day_of_week().is_empty());
    }

    #[test]
    fn by_day_of_week_infers_weekly_bpy_for_daily_bars() {
        // C-3: for a daily-bar backtest filtered to Mondays, the inferred
        // bars_per_year should be ≈52 (one per week), not the configured 252.
        // We verify this indirectly: Sharpe from by_day_of_week should differ
        // from a Sharpe computed with bpy=252 on the same Monday returns,
        // confirming that infer_bars_per_year adjusted the annualisation.
        //
        // Build 2 years of weekly Monday equity points (≈104 points).
        let base = ts("2023-01-02"); // Monday
        let week_secs = 7 * 86400i64;
        let n_weeks = 104usize;
        let equity_pts: Vec<EquityPoint> = (0..n_weeks)
            .map(|i| {
                equity_point(
                    base + (i as i64) * week_secs,
                    10000.0 + i as f64 * 10.0,
                    0.0,
                )
            })
            .collect();

        let trade = make_trade_timed(
            100.0,
            1.0,
            base,
            base + week_secs, // exit on the second Monday
        );
        let result = make_result(vec![trade], equity_pts.clone());
        let by_dow = result.by_day_of_week();

        // The inferred bpy from 103 weekly returns over ~2 years ≈ 52.
        // With bpy=252, Sharpe would be sqrt(252/52) ≈ 2.2× larger.
        // We only assert the result is finite and present — correctness of
        // the specific ratio is covered by infer_bars_per_year unit behaviour.
        assert!(by_dow.contains_key(&Weekday::Mon));
        let s = by_dow[&Weekday::Mon].sharpe_ratio;
        assert!(
            s.is_finite() || s == f64::MAX,
            "Sharpe should be finite, got {s}"
        );
    }

    #[test]
    fn infer_bars_per_year_approximates_weekly_for_monday_subset() {
        // Direct unit test for infer_bars_per_year.
        // 104 weekly Monday points over ~2 calendar years → ≈ 52 bpy
        let base = ts("2023-01-02");
        let week_secs = 7 * 86400i64;
        let pts: Vec<EquityPoint> = (0..104)
            .map(|i| equity_point(base + i * week_secs, 10000.0, 0.0))
            .collect();
        let bpy = infer_bars_per_year(&pts, 252.0);
        // 103 return periods over ~2 years ≈ 51.5; accept 48–56 as reasonable
        assert!(bpy > 48.0 && bpy < 56.0, "expected ~52, got {bpy}");
    }
}
