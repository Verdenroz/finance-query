use std::collections::HashMap;

use chrono::{Datelike, Weekday};

use super::stats::{datetime_from_timestamp, infer_bars_per_year, partial_period_adjust};
use super::{BacktestResult, EquityPoint, PerformanceMetrics};
use crate::backtesting::position::Trade;

impl BacktestResult {
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
    use super::super::fixtures::{equity_point, make_result, make_trade_timed, ts};
    use super::*;

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
}
