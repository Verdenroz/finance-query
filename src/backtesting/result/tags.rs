use super::stats::{infer_bars_per_year, partial_period_adjust};
use super::{BacktestResult, EquityPoint, PerformanceMetrics};
use crate::backtesting::position::Trade;

impl BacktestResult {
    // ─── Trade tagging & subgroup analysis ───────────────────────────────────

    /// Return all trades that carry the given tag.
    ///
    /// Tags are attached to a [`Signal`](crate::backtesting::Signal) at strategy time with `.tag("name")`
    /// and propagated to [`Trade::tags`] when the position closes.
    ///
    /// Tag comparison is **exact and case-sensitive**: `"Breakout"` and
    /// `"breakout"` are distinct tags.  Normalise tag strings at the call
    /// site if case-insensitive matching is required.
    ///
    /// Returns an empty `Vec` when no trades match or no trades have been
    /// tagged at all.
    pub fn trades_by_tag(&self, tag: &str) -> Vec<&Trade> {
        self.trades
            .iter()
            .filter(|t| t.tags.iter().any(|t2| t2 == tag))
            .collect()
    }

    /// Compute `PerformanceMetrics` for the subset of trades that carry `tag`.
    ///
    /// A synthetic equity curve is built by replaying the tagged trades in
    /// sequence starting from `initial_capital`, which gives an accurate
    /// drawdown and return series for that trade subset.
    ///
    /// # Capital base
    ///
    /// All return metrics (`total_return_pct`, `annualized_return_pct`,
    /// `sharpe_ratio`, `calmar_ratio`) are computed relative to
    /// **`initial_capital`** — the full portfolio starting value — *not* the
    /// capital actually deployed into tagged trades.  A tag that fired 2
    /// small trades on a $10,000 portfolio will show a lower `total_return_pct`
    /// than a tag that deployed the same profit using more capital.
    ///
    /// For capital-independent comparisons across tags prefer:
    /// `profit_factor`, `win_rate`, `avg_win_pct`, `avg_loss_pct`.
    ///
    /// # Sharpe / Sortino annualisation
    ///
    /// `bars_per_year` is inferred from the calendar span of the synthetic
    /// equity curve (same technique as [`by_day_of_week`](Self::by_day_of_week))
    /// so that a sparsely-firing tag is not penalised by an inflated
    /// annualisation factor.
    ///
    /// Returns a zero-valued `PerformanceMetrics` (all fields zero except
    /// `total_return_pct`) when no tagged trades exist.
    pub fn metrics_by_tag(&self, tag: &str) -> PerformanceMetrics {
        // Single pass: collect tagged trades and build synthetic equity curve.
        let mut equity = self.initial_capital;
        let mut peak = equity;
        let mut trades_vec: Vec<Trade> = Vec::new();
        let mut equity_curve: Vec<EquityPoint> = Vec::new();

        for trade in &self.trades {
            if !trade.tags.iter().any(|t| t == tag) {
                continue;
            }
            if equity_curve.is_empty() {
                equity_curve.push(EquityPoint {
                    timestamp: trade.entry_timestamp,
                    equity,
                    drawdown_pct: 0.0,
                });
            }
            equity += trade.pnl;
            if equity > peak {
                peak = equity;
            }
            let drawdown_pct = if peak > 0.0 {
                (peak - equity) / peak
            } else {
                0.0
            };
            equity_curve.push(EquityPoint {
                timestamp: trade.exit_timestamp,
                equity,
                drawdown_pct,
            });
            trades_vec.push(trade.clone());
        }

        if trades_vec.is_empty() {
            return PerformanceMetrics::empty(self.initial_capital, &[], 0, 0);
        }

        // H-2: infer effective bars_per_year from the synthetic curve's
        // calendar span — avoids inflating Sharpe for sparsely-firing tags.
        let bpy = infer_bars_per_year(&equity_curve, self.config.bars_per_year);
        let metrics = PerformanceMetrics::calculate(
            &trades_vec,
            &equity_curve,
            self.initial_capital,
            0,
            0,
            self.config.risk_free_rate,
            bpy,
        );
        partial_period_adjust(metrics, equity_curve.len(), bpy)
    }

    /// Return a sorted, deduplicated list of all tags used across all trades.
    ///
    /// Useful for discovering which tags are present in a result before
    /// calling `trades_by_tag` or `metrics_by_tag`.
    pub fn all_tags(&self) -> Vec<&str> {
        let mut tags: std::collections::BTreeSet<&str> = std::collections::BTreeSet::new();
        for trade in &self.trades {
            for tag in &trade.tags {
                tags.insert(tag.as_str());
            }
        }
        tags.into_iter().collect()
    }
}

#[cfg(test)]
mod tests {
    use super::super::fixtures::{equity_point, make_result, make_trade};
    use super::*;
    use crate::backtesting::position::{Position, PositionSide};
    use crate::backtesting::signal::Signal;

    // ─── Trade tagging & subgroup analysis ───────────────────────────────────

    /// Create a tagged trade by going through the real `Position::close` path
    /// so that tag propagation is exercised end-to-end in tests.
    fn make_tagged_trade(pnl: f64, tags: &[&str]) -> Trade {
        let entry_signal = tags
            .iter()
            .fold(Signal::long(0, 100.0), |sig, &t| sig.tag(t));
        // quantity=10, entry_price=100 → entry_value=1000
        // exit_price chosen so that pnl = (exit - 100) * 10
        let exit_price = 100.0 + pnl / 10.0;
        let exit_ts = 86400i64;
        let pos = Position::new(PositionSide::Long, 0, 100.0, 10.0, 0.0, entry_signal);
        pos.close(exit_ts, exit_price, 0.0, Signal::exit(exit_ts, exit_price))
    }

    /// Like `make_tagged_trade` but for a short position.
    fn make_tagged_short_trade(pnl: f64, tags: &[&str]) -> Trade {
        let entry_signal = tags
            .iter()
            .fold(Signal::short(0, 100.0), |sig, &t| sig.tag(t));
        // For a short: pnl = (entry - exit) * qty
        // entry_price=100, qty=10 → pnl = (100 - exit) * 10 → exit = 100 - pnl/10
        let exit_price = 100.0 - pnl / 10.0;
        let exit_ts = 86400i64;
        let pos = Position::new(PositionSide::Short, 0, 100.0, 10.0, 0.0, entry_signal);
        pos.close(exit_ts, exit_price, 0.0, Signal::exit(exit_ts, exit_price))
    }

    // ── Signal::tag builder ───────────────────────────────────────────────────

    #[test]
    fn signal_tag_builder_appends_tag() {
        let sig = Signal::long(0, 100.0).tag("breakout");
        assert_eq!(sig.tags, vec!["breakout"]);
    }

    #[test]
    fn signal_tag_builder_chains_multiple_tags() {
        let sig = Signal::long(0, 100.0).tag("breakout").tag("high_volume");
        assert_eq!(sig.tags, vec!["breakout", "high_volume"]);
    }

    #[test]
    fn signal_tag_builder_preserves_order() {
        let sig = Signal::long(0, 100.0).tag("a").tag("b").tag("c");
        assert_eq!(sig.tags, vec!["a", "b", "c"]);
    }

    #[test]
    fn signal_constructors_start_with_empty_tags() {
        assert!(Signal::long(0, 0.0).tags.is_empty());
        assert!(Signal::short(0, 0.0).tags.is_empty());
        assert!(Signal::exit(0, 0.0).tags.is_empty());
        assert!(Signal::hold().tags.is_empty());
    }

    // ── Tag propagation via Position::close ───────────────────────────────────

    #[test]
    fn position_close_propagates_entry_signal_tags_to_trade() {
        let entry_signal = Signal::long(0, 100.0).tag("breakout").tag("high_volume");
        let pos = Position::new(
            crate::backtesting::position::PositionSide::Long,
            0,
            100.0,
            10.0,
            0.0,
            entry_signal,
        );
        let trade = pos.close(86400, 110.0, 0.0, Signal::exit(86400, 110.0));
        assert_eq!(trade.tags, vec!["breakout", "high_volume"]);
    }

    #[test]
    fn position_close_propagates_empty_tags_when_none_set() {
        let entry_signal = Signal::long(0, 100.0);
        let pos = Position::new(
            crate::backtesting::position::PositionSide::Long,
            0,
            100.0,
            10.0,
            0.0,
            entry_signal,
        );
        let trade = pos.close(86400, 110.0, 0.0, Signal::exit(86400, 110.0));
        assert!(trade.tags.is_empty());
    }

    // ── trades_by_tag ─────────────────────────────────────────────────────────

    #[test]
    fn trades_by_tag_returns_matching_trades() {
        let result = make_result(
            vec![
                make_tagged_trade(100.0, &["breakout"]),
                make_tagged_trade(-50.0, &["reversal"]),
                make_tagged_trade(200.0, &["breakout", "high_volume"]),
            ],
            vec![equity_point(0, 10000.0, 0.0)],
        );
        let tagged = result.trades_by_tag("breakout");
        assert_eq!(tagged.len(), 2);
        assert!((tagged[0].pnl - 100.0).abs() < 1e-9);
        assert!((tagged[1].pnl - 200.0).abs() < 1e-9);
    }

    #[test]
    fn trades_by_tag_returns_empty_for_missing_tag() {
        let result = make_result(
            vec![make_tagged_trade(100.0, &["breakout"])],
            vec![equity_point(0, 10000.0, 0.0)],
        );
        assert!(result.trades_by_tag("nonexistent").is_empty());
    }

    #[test]
    fn trades_by_tag_returns_empty_when_no_trades_tagged() {
        let result = make_result(
            vec![make_trade(100.0, 10.0, true)],
            vec![equity_point(0, 10000.0, 0.0)],
        );
        assert!(result.trades_by_tag("breakout").is_empty());
    }

    #[test]
    fn trades_by_tag_multi_tag_trade_matches_each_tag() {
        let result = make_result(
            vec![make_tagged_trade(100.0, &["a", "b", "c"])],
            vec![equity_point(0, 10000.0, 0.0)],
        );
        assert_eq!(result.trades_by_tag("a").len(), 1);
        assert_eq!(result.trades_by_tag("b").len(), 1);
        assert_eq!(result.trades_by_tag("c").len(), 1);
        assert_eq!(result.trades_by_tag("d").len(), 0);
    }

    // ── all_tags ──────────────────────────────────────────────────────────────

    #[test]
    fn all_tags_returns_sorted_deduped_tags() {
        let result = make_result(
            vec![
                make_tagged_trade(10.0, &["z_tag", "a_tag"]),
                make_tagged_trade(10.0, &["m_tag", "a_tag"]),
            ],
            vec![equity_point(0, 10000.0, 0.0)],
        );
        let tags = result.all_tags();
        assert_eq!(tags, vec!["a_tag", "m_tag", "z_tag"]);
    }

    #[test]
    fn all_tags_returns_empty_when_no_tagged_trades() {
        let result = make_result(
            vec![make_trade(100.0, 10.0, true)],
            vec![equity_point(0, 10000.0, 0.0)],
        );
        assert!(result.all_tags().is_empty());
    }

    #[test]
    fn all_tags_returns_empty_when_no_trades() {
        let result = make_result(vec![], vec![equity_point(0, 10000.0, 0.0)]);
        assert!(result.all_tags().is_empty());
    }

    // ── metrics_by_tag ────────────────────────────────────────────────────────

    #[test]
    fn metrics_by_tag_returns_empty_metrics_for_missing_tag() {
        let result = make_result(
            vec![make_tagged_trade(100.0, &["breakout"])],
            vec![equity_point(0, 10000.0, 0.0)],
        );
        let metrics = result.metrics_by_tag("nonexistent");
        assert_eq!(metrics.total_trades, 0);
        assert_eq!(metrics.win_rate, 0.0);
    }

    #[test]
    fn metrics_by_tag_counts_only_tagged_trades() {
        let result = make_result(
            vec![
                make_tagged_trade(100.0, &["breakout"]),
                make_tagged_trade(200.0, &["breakout"]),
                make_tagged_trade(-50.0, &["reversal"]),
            ],
            vec![equity_point(0, 10000.0, 0.0)],
        );
        let metrics = result.metrics_by_tag("breakout");
        assert_eq!(metrics.total_trades, 2);
        assert_eq!(metrics.long_trades, 2);
    }

    #[test]
    fn metrics_by_tag_win_rate_all_profitable() {
        let result = make_result(
            vec![
                make_tagged_trade(100.0, &["win"]),
                make_tagged_trade(200.0, &["win"]),
            ],
            vec![equity_point(0, 10000.0, 0.0)],
        );
        let metrics = result.metrics_by_tag("win");
        assert!(
            (metrics.win_rate - 1.0).abs() < 1e-9,
            "expected 100% win rate"
        );
    }

    #[test]
    fn metrics_by_tag_win_rate_half_profitable() {
        let result = make_result(
            vec![
                make_tagged_trade(100.0, &["mixed"]),
                make_tagged_trade(-100.0, &["mixed"]),
            ],
            vec![equity_point(0, 10000.0, 0.0)],
        );
        let metrics = result.metrics_by_tag("mixed");
        assert!(
            (metrics.win_rate - 0.5).abs() < 1e-9,
            "expected 50% win rate, got {}",
            metrics.win_rate
        );
    }

    #[test]
    fn metrics_by_tag_total_return_reflects_tagged_pnl() {
        // Two breakout trades: +$100, +$200 → total P&L $300 on $10,000 capital = 3%
        let result = make_result(
            vec![
                make_tagged_trade(100.0, &["breakout"]),
                make_tagged_trade(200.0, &["breakout"]),
                make_tagged_trade(-500.0, &["other"]),
            ],
            vec![equity_point(0, 10000.0, 0.0)],
        );
        let metrics = result.metrics_by_tag("breakout");
        // total_return_pct = (final_equity - initial) / initial * 100
        // = 300 / 10000 * 100 = 3.0%
        assert!(
            (metrics.total_return_pct - 3.0).abs() < 0.01,
            "expected 3%, got {}",
            metrics.total_return_pct
        );
    }

    // L-3: mixed-side trades under the same tag
    #[test]
    fn metrics_by_tag_mixed_long_short_counts_correctly() {
        let long_trade = make_tagged_trade(100.0, &["strategy"]);
        let short_trade = make_tagged_short_trade(50.0, &["strategy"]);
        assert!(long_trade.is_long());
        assert!(short_trade.is_short());

        let result = make_result(
            vec![long_trade, short_trade],
            vec![equity_point(0, 10000.0, 0.0)],
        );
        let metrics = result.metrics_by_tag("strategy");
        assert_eq!(metrics.total_trades, 2);
        assert_eq!(metrics.long_trades, 1);
        assert_eq!(metrics.short_trades, 1);
        assert!(
            (metrics.win_rate - 1.0).abs() < 1e-9,
            "both trades are profitable"
        );
    }

    // L-4: duplicate tags on a single signal are stored as-is; all_tags deduplicates
    #[test]
    fn all_tags_deduplicates_within_single_trade() {
        let sig = Signal::long(0, 100.0).tag("dup").tag("dup");
        let pos = Position::new(PositionSide::Long, 0, 100.0, 10.0, 0.0, sig);
        let trade = pos.close(86400, 110.0, 0.0, Signal::exit(86400, 110.0));
        assert_eq!(trade.tags, vec!["dup", "dup"]); // raw tags preserved on Trade
        let result = make_result(vec![trade], vec![equity_point(0, 10000.0, 0.0)]);
        assert_eq!(result.all_tags(), vec!["dup"]); // all_tags deduplicates
    }

    // L-2: case sensitivity documented behaviour
    #[test]
    fn trades_by_tag_is_case_sensitive() {
        let result = make_result(
            vec![make_tagged_trade(100.0, &["Breakout"])],
            vec![equity_point(0, 10000.0, 0.0)],
        );
        assert_eq!(result.trades_by_tag("Breakout").len(), 1);
        assert_eq!(result.trades_by_tag("breakout").len(), 0);
        assert_eq!(result.trades_by_tag("BREAKOUT").len(), 0);
    }
}
