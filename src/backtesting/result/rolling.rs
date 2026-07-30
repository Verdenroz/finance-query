use super::BacktestResult;
use super::stats::{calculate_periodic_returns, calculate_risk_ratios};

impl BacktestResult {
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
}

#[cfg(test)]
mod tests {
    use super::super::EquityPoint;
    use super::super::stats::fixtures::{equity_point, make_result, make_trade};
    use crate::backtesting::position::Trade;

    // ─── Phase 2 — Rolling & Temporal Analysis ───────────────────────────────

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
}
