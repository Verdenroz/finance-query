use chrono::{DateTime, NaiveDateTime, Utc};

use super::{EquityPoint, PerformanceMetrics};
use crate::backtesting::position::Trade;

/// Aggregated trade statistics collected in a single pass over the trade log.
pub(super) struct TradeStats {
    pub(super) winning_trades: usize,
    pub(super) losing_trades: usize,
    pub(super) long_trades: usize,
    pub(super) short_trades: usize,
    pub(super) gross_profit: f64,
    pub(super) gross_loss: f64,
    pub(super) total_return_sum: f64,
    pub(super) total_duration: i64,
    pub(super) largest_win: f64,
    pub(super) largest_loss: f64,
    pub(super) total_commission: f64,
    pub(super) total_financing_cost: f64,
    pub(super) total_dividend_income: f64,
    pub(super) winning_returns: Vec<f64>,
    pub(super) losing_returns: Vec<f64>,
    /// All trade return percentages (wins + losses + break-even).
    pub(super) all_returns: Vec<f64>,
}

/// Single-pass accumulation of all per-trade statistics.
pub(super) fn analyze_trades(trades: &[Trade]) -> TradeStats {
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
        total_financing_cost: 0.0,
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
        stats.total_financing_cost += t.financing_cost;
        stats.total_dividend_income += t.dividend_income;
        stats.all_returns.push(t.return_pct);
    }

    stats
}

/// Kelly Criterion: `W - (1 - W) / R` where R = avg_win / abs(avg_loss).
///
/// Returns `f64::MAX` when there are no losing trades and wins are positive
/// (unbounded edge). Returns `0.0` when inputs are degenerate.
///
/// Delegates to `crate::perf_metrics::kelly_criterion`, shared with the
/// standalone `risk` module so both compute the same formula without either
/// feature depending on the other (see that module's doc comment).
pub(super) fn calculate_kelly(win_rate: f64, avg_win_pct: f64, avg_loss_pct: f64) -> f64 {
    crate::perf_metrics::kelly_criterion(win_rate, avg_win_pct, avg_loss_pct)
}

/// Van Tharp's System Quality Number.
///
/// `(mean_R / std_R) * sqrt(n)` over per-trade return percentages.
/// Uses sample standard deviation (n-1). Returns `0.0` for fewer than 2 trades.
pub(super) fn calculate_sqn(returns: &[f64]) -> f64 {
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
///
/// Delegates to `crate::perf_metrics::omega_ratio` (see that module's doc
/// comment for why the shared formula lives outside both `risk` and
/// `backtesting`).
pub(super) fn calculate_omega_ratio(returns: &[f64]) -> f64 {
    crate::perf_metrics::omega_ratio(returns)
}

/// Tail Ratio: `abs(p95) / abs(p5)` of trade returns.
///
/// Returns `0.0` for fewer than 2 trades, `f64::MAX` when `p5 == 0`.
pub(super) fn calculate_tail_ratio(returns: &[f64]) -> f64 {
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
///
/// Delegates to `crate::perf_metrics::ulcer_index`, which takes plain
/// drawdown fractions rather than `EquityPoint` so it stays usable from the
/// standalone `risk` module too.
pub(super) fn calculate_ulcer_index(equity_curve: &[EquityPoint]) -> f64 {
    let drawdowns: Vec<f64> = equity_curve.iter().map(|p| p.drawdown_pct).collect();
    crate::perf_metrics::ulcer_index(&drawdowns)
}

/// Calculate maximum consecutive wins and losses
pub(super) fn calculate_consecutive(trades: &[Trade]) -> (usize, usize) {
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
pub(super) fn calculate_max_drawdown_duration(equity_curve: &[EquityPoint]) -> i64 {
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
pub(super) fn calculate_periodic_returns(equity_curve: &[EquityPoint]) -> Vec<f64> {
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
pub(super) fn calculate_risk_ratios(
    returns: &[f64],
    annual_risk_free_rate: f64,
    bars_per_year: f64,
) -> (f64, f64) {
    if returns.len() < 2 {
        return (0.0, 0.0);
    }

    let periodic_rf = annual_to_periodic_rf(annual_risk_free_rate, bars_per_year);
    let n = returns.len() as f64;

    // Pass 1: mean of excess returns (no allocation)
    let mean = returns.iter().map(|r| r - periodic_rf).sum::<f64>() / n;

    // Pass 2: variance and downside sum in one loop (no allocation)
    let (var_sum, downside_sq_sum) = returns.iter().fold((0.0_f64, 0.0_f64), |(v, d), &r| {
        let e = r - periodic_rf;
        let delta = e - mean;
        (v + delta * delta, if e < 0.0 { d + e * e } else { d })
    });

    // Sharpe: sample variance (n-1) for unbiased estimation
    let std_dev = (var_sum / (n - 1.0)).sqrt();
    let sharpe = if std_dev > 0.0 {
        (mean / std_dev) * bars_per_year.sqrt()
    } else if mean > 0.0 {
        f64::MAX
    } else {
        0.0
    };

    // Sortino: downside deviation (only negative excess; denominator is n-1,
    // per Sortino's original definition and the `risk` module convention)
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
pub(super) fn calculate_win_loss_durations(trades: &[Trade]) -> (f64, f64) {
    let (win_sum, win_count, loss_sum, loss_count) =
        trades
            .iter()
            .fold((0i64, 0usize, 0i64, 0usize), |(ws, wc, ls, lc), t| {
                if t.is_profitable() {
                    (ws + t.duration_secs(), wc + 1, ls, lc)
                } else if t.is_loss() {
                    (ws, wc, ls + t.duration_secs(), lc + 1)
                } else {
                    (ws, wc, ls, lc)
                }
            });

    let avg_win = if win_count == 0 {
        0.0
    } else {
        win_sum as f64 / win_count as f64
    };
    let avg_loss = if loss_count == 0 {
        0.0
    } else {
        loss_sum as f64 / loss_count as f64
    };

    (avg_win, avg_loss)
}

/// Calculate fraction of backtest time spent in a position.
///
/// Uses the ratio of total trade duration to the total backtest duration
/// derived from the equity curve timestamps.
pub(super) fn calculate_time_in_market(trades: &[Trade], equity_curve: &[EquityPoint]) -> f64 {
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
pub(super) fn calculate_max_idle_period(trades: &[Trade]) -> i64 {
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
pub(super) fn infer_bars_per_year(equity_slice: &[EquityPoint], fallback_bpy: f64) -> f64 {
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
pub(super) fn partial_period_adjust(
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
pub(super) fn datetime_from_timestamp(ts: i64) -> Option<NaiveDateTime> {
    DateTime::<Utc>::from_timestamp(ts, 0).map(|dt| dt.naive_utc())
}

#[cfg(test)]
mod tests {
    use super::super::fixtures::{equity_point, make_trade, ts};
    use super::*;

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
