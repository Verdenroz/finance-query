use serde::{Deserialize, Serialize};

use super::EquityPoint;
use super::stats::{
    analyze_trades, calculate_consecutive, calculate_kelly, calculate_max_drawdown_duration,
    calculate_max_idle_period, calculate_omega_ratio, calculate_periodic_returns,
    calculate_risk_ratios, calculate_sqn, calculate_tail_ratio, calculate_time_in_market,
    calculate_ulcer_index, calculate_win_loss_durations,
};
use crate::backtesting::position::Trade;

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

    /// Average trade duration in seconds
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

    /// Total cost of borrowed capital over the run: short borrow fees and
    /// margin interest. Already subtracted from each trade's P&L, and includes
    /// what a still-open position has accrued so far.
    #[serde(default)]
    pub total_financing_cost: f64,

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
    /// Computed as `W - (1 - W) / R` where `R` is `avg_win_pct /
    /// abs(avg_loss_pct)` and `W` is the win rate over decisive
    /// (non-break-even) trades, unlike [`win_rate`](Self::win_rate) which is
    /// diluted by break-even trades. A positive value suggests the strategy
    /// has an edge; a negative value suggests it does not. Values above 1
    /// indicate extreme edge (rare in practice). Returns `0.0` when there are
    /// no losing trades to compute a ratio.
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
    pub(super) fn empty(
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
            total_financing_cost: 0.0,
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
        let total_trades = trades.len();
        let stats = analyze_trades(trades);

        // Curve-derived metrics come before the zero-trade early return: a
        // position left open the whole run (close_at_end = false) has no
        // closed trades but still carries real drawdown/risk.

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

        // Omega uses the same bar-by-bar returns as Sharpe/Sortino; per-trade
        // returns vary by holding period and are incomparable across strategies.
        let omega_ratio = calculate_omega_ratio(&returns);
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

        if total_trades == 0 {
            return Self {
                total_return_pct,
                annualized_return_pct,
                sharpe_ratio,
                sortino_ratio,
                max_drawdown_pct,
                max_drawdown_duration,
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
                calmar_ratio,
                total_commission: 0.0,
                total_financing_cost: 0.0,
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
                omega_ratio,
                tail_ratio: 0.0,
                recovery_factor,
                ulcer_index,
                serenity_ratio,
            };
        }

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

        // Trade duration analysis
        let (avg_win_duration, avg_loss_duration) = calculate_win_loss_durations(trades);
        let time_in_market_pct = calculate_time_in_market(trades, equity_curve);
        let max_idle_period = calculate_max_idle_period(trades);

        // Phase 1 — extended metrics
        let decisive_trades = stats.winning_trades + stats.losing_trades;
        let kelly_win_rate = if decisive_trades > 0 {
            stats.winning_trades as f64 / decisive_trades as f64
        } else {
            0.0
        };
        let kelly_criterion = calculate_kelly(kelly_win_rate, avg_win_pct, avg_loss_pct);
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
        let tail_ratio = calculate_tail_ratio(&stats.all_returns);

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
            total_financing_cost: stats.total_financing_cost,
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

#[cfg(test)]
mod tests {
    use super::super::fixtures::make_trade;
    use super::*;

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

    #[test]
    fn test_kelly_uses_decisive_win_rate_not_diluted_win_rate() {
        // 1 win (+10%), 2 break-even, 1 loss (-5%): diluted win_rate=0.25
        // would give a negative Kelly, but the decisive win_rate (1 win of
        // 2 decisive trades = 0.5) gives Kelly = 0.5 - 0.5/2 = 0.25.
        let trades = vec![
            make_trade(10.0, 10.0, true),
            make_trade(0.0, 0.0, true),
            make_trade(0.0, 0.0, true),
            make_trade(-5.0, -5.0, true),
        ];
        let equity = vec![
            EquityPoint {
                timestamp: 0,
                equity: 10000.0,
                drawdown_pct: 0.0,
            },
            EquityPoint {
                timestamp: 1,
                equity: 10005.0,
                drawdown_pct: 0.0,
            },
        ];

        let metrics = PerformanceMetrics::calculate(&trades, &equity, 10000.0, 4, 4, 0.0, 252.0);

        assert!((metrics.win_rate - 0.25).abs() < 1e-9);
        assert!(
            (metrics.kelly_criterion - 0.25).abs() < 1e-9,
            "expected 0.25, got {}",
            metrics.kelly_criterion
        );
    }

    #[test]
    fn test_metrics_no_trades_reports_curve_drawdown_for_open_position() {
        // close_at_end = false leaves an open position with no closed trades,
        // but the equity curve still marks the position to market every bar.
        let equity = vec![
            EquityPoint {
                timestamp: 0,
                equity: 10000.0,
                drawdown_pct: 0.0,
            },
            EquityPoint {
                timestamp: 1,
                equity: 7000.0,
                drawdown_pct: 0.3,
            },
            EquityPoint {
                timestamp: 2,
                equity: 10500.0,
                drawdown_pct: 0.0,
            },
        ];

        let metrics = PerformanceMetrics::calculate(&[], &equity, 10000.0, 0, 0, 0.0, 252.0);

        assert_eq!(metrics.total_trades, 0);
        assert!((metrics.total_return_pct - 5.0).abs() < 0.01);
        assert!(
            (metrics.max_drawdown_pct - 0.3).abs() < 1e-9,
            "expected 0.3, got {}",
            metrics.max_drawdown_pct
        );
        assert!(metrics.ulcer_index > 0.0);
    }
}
