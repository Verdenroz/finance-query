//! Threshold-based conditions for position management.
//!
//! This module provides conditions for stop-loss, take-profit, and trailing stops.

use crate::backtesting::strategy::{PositionExtremes, StrategyContext};
use crate::indicators::Indicator;

use super::Condition;

/// Condition: position P/L is at or below the stop-loss threshold.
///
/// # Execution Model
///
/// This condition evaluates at **bar close**: it fires when the closing price
/// implies a loss ≥ `pct`. The resulting exit signal is deferred to the **next
/// bar's open** (identical to all strategy-signal exits).
///
/// For intrabar detection (fill same bar at `min(open, stop_level)`), use
/// [`BacktestConfig::stop_loss_pct`] instead. A −10% intraday move that closes
/// at −3% will be caught by the config field but missed by this condition.
///
/// # Example
///
/// ```ignore
/// use finance_query::backtesting::condition::*;
///
/// let exit = stop_loss(0.05); // Exit if loss >= 5% at bar close
/// ```
#[derive(Debug, Clone, Copy)]
pub struct StopLoss {
    /// Stop-loss percentage (e.g., 0.05 for 5%)
    pub pct: f64,
}

impl StopLoss {
    /// Create a new stop-loss condition.
    ///
    /// # Arguments
    ///
    /// * `pct` - Stop-loss percentage (e.g., 0.05 for 5%)
    pub fn new(pct: f64) -> Self {
        Self { pct }
    }
}

impl Condition for StopLoss {
    fn evaluate(&self, ctx: &StrategyContext) -> bool {
        if let Some(pos) = ctx.position {
            let pnl_pct = pos.unrealized_return_pct(ctx.close()) / 100.0;
            pnl_pct <= -self.pct
        } else {
            false
        }
    }

    fn required_indicators(&self) -> Vec<(String, Indicator)> {
        vec![]
    }

    fn description(&self) -> String {
        format!("stop loss at {:.1}%", self.pct * 100.0)
    }
}

/// Create a stop-loss condition.
///
/// # Example
///
/// ```ignore
/// use finance_query::backtesting::condition::*;
///
/// let exit = rsi(14).above(70.0).or(stop_loss(0.05));
/// ```
#[inline]
pub fn stop_loss(pct: f64) -> StopLoss {
    StopLoss::new(pct)
}

/// Condition: position P/L is at or above the take-profit threshold.
///
/// # Execution Model
///
/// This condition evaluates at **bar close**: it fires when the closing price
/// implies a gain ≥ `pct`. The resulting exit signal is deferred to the **next
/// bar's open** (identical to all strategy-signal exits).
///
/// For intrabar detection (fill same bar at `max(open, target_level)`), use
/// [`BacktestConfig::take_profit_pct`] instead.
///
/// # Example
///
/// ```ignore
/// use finance_query::backtesting::condition::*;
///
/// let exit = take_profit(0.10); // Exit if gain >= 10% at bar close
/// ```
#[derive(Debug, Clone, Copy)]
pub struct TakeProfit {
    /// Take-profit percentage (e.g., 0.10 for 10%)
    pub pct: f64,
}

impl TakeProfit {
    /// Create a new take-profit condition.
    ///
    /// # Arguments
    ///
    /// * `pct` - Take-profit percentage (e.g., 0.10 for 10%)
    pub fn new(pct: f64) -> Self {
        Self { pct }
    }
}

impl Condition for TakeProfit {
    fn evaluate(&self, ctx: &StrategyContext) -> bool {
        if let Some(pos) = ctx.position {
            let pnl_pct = pos.unrealized_return_pct(ctx.close()) / 100.0;
            pnl_pct >= self.pct
        } else {
            false
        }
    }

    fn required_indicators(&self) -> Vec<(String, Indicator)> {
        vec![]
    }

    fn description(&self) -> String {
        format!("take profit at {:.1}%", self.pct * 100.0)
    }
}

/// Create a take-profit condition.
///
/// # Example
///
/// ```ignore
/// use finance_query::backtesting::condition::*;
///
/// let exit = rsi(14).above(70.0).or(take_profit(0.15));
/// ```
#[inline]
pub fn take_profit(pct: f64) -> TakeProfit {
    TakeProfit::new(pct)
}

/// Condition: check if we have any position.
#[derive(Debug, Clone, Copy)]
pub struct HasPosition;

impl Condition for HasPosition {
    fn evaluate(&self, ctx: &StrategyContext) -> bool {
        ctx.has_position()
    }

    fn required_indicators(&self) -> Vec<(String, Indicator)> {
        vec![]
    }

    fn description(&self) -> String {
        "has position".to_string()
    }
}

/// Create a condition that checks if we have any position.
#[inline]
pub fn has_position() -> HasPosition {
    HasPosition
}

/// Condition: check if we have no position.
#[derive(Debug, Clone, Copy)]
pub struct NoPosition;

impl Condition for NoPosition {
    fn evaluate(&self, ctx: &StrategyContext) -> bool {
        !ctx.has_position()
    }

    fn required_indicators(&self) -> Vec<(String, Indicator)> {
        vec![]
    }

    fn description(&self) -> String {
        "no position".to_string()
    }
}

/// Create a condition that checks if we have no position.
#[inline]
pub fn no_position() -> NoPosition {
    NoPosition
}

/// Condition: check if we have a long position.
#[derive(Debug, Clone, Copy)]
pub struct IsLong;

impl Condition for IsLong {
    fn evaluate(&self, ctx: &StrategyContext) -> bool {
        ctx.is_long()
    }

    fn required_indicators(&self) -> Vec<(String, Indicator)> {
        vec![]
    }

    fn description(&self) -> String {
        "is long".to_string()
    }
}

/// Create a condition that checks if we have a long position.
#[inline]
pub fn is_long() -> IsLong {
    IsLong
}

/// Condition: check if we have a short position.
#[derive(Debug, Clone, Copy)]
pub struct IsShort;

impl Condition for IsShort {
    fn evaluate(&self, ctx: &StrategyContext) -> bool {
        ctx.is_short()
    }

    fn required_indicators(&self) -> Vec<(String, Indicator)> {
        vec![]
    }

    fn description(&self) -> String {
        "is short".to_string()
    }
}

/// Create a condition that checks if we have a short position.
#[inline]
pub fn is_short() -> IsShort {
    IsShort
}

/// Condition: position P/L is positive (in profit).
#[derive(Debug, Clone, Copy)]
pub struct InProfit;

impl Condition for InProfit {
    fn evaluate(&self, ctx: &StrategyContext) -> bool {
        if let Some(pos) = ctx.position {
            pos.unrealized_return_pct(ctx.close()) > 0.0
        } else {
            false
        }
    }

    fn required_indicators(&self) -> Vec<(String, Indicator)> {
        vec![]
    }

    fn description(&self) -> String {
        "in profit".to_string()
    }
}

/// Create a condition that checks if position is profitable.
#[inline]
pub fn in_profit() -> InProfit {
    InProfit
}

/// Condition: position P/L is negative (in loss).
#[derive(Debug, Clone, Copy)]
pub struct InLoss;

impl Condition for InLoss {
    fn evaluate(&self, ctx: &StrategyContext) -> bool {
        if let Some(pos) = ctx.position {
            pos.unrealized_return_pct(ctx.close()) < 0.0
        } else {
            false
        }
    }

    fn required_indicators(&self) -> Vec<(String, Indicator)> {
        vec![]
    }

    fn description(&self) -> String {
        "in loss".to_string()
    }
}

/// Create a condition that checks if position is at a loss.
#[inline]
pub fn in_loss() -> InLoss {
    InLoss
}

/// Condition: position has been held for at least N bars.
#[derive(Debug, Clone, Copy)]
pub struct HeldForBars {
    /// Minimum number of bars the position must be held
    pub min_bars: usize,
}

impl HeldForBars {
    /// Create a new held-for-bars condition.
    pub fn new(min_bars: usize) -> Self {
        Self { min_bars }
    }
}

impl Condition for HeldForBars {
    fn evaluate(&self, ctx: &StrategyContext) -> bool {
        if let Some(pos) = ctx.position {
            // Count bars since entry
            let entry_idx = entry_index(ctx.candles, pos.entry_timestamp);
            let bars_held = ctx.index.saturating_sub(entry_idx);
            bars_held >= self.min_bars
        } else {
            false
        }
    }

    fn required_indicators(&self) -> Vec<(String, Indicator)> {
        vec![]
    }

    fn description(&self) -> String {
        format!("held for {} bars", self.min_bars)
    }
}

/// Create a condition that checks if position has been held for at least N bars.
#[inline]
pub fn held_for_bars(min_bars: usize) -> HeldForBars {
    HeldForBars::new(min_bars)
}

/// Index of the first candle at or after the position's entry.
///
/// Candles are sorted ascending, so this is a binary search; the `== len` arm
/// reproduces the `.unwrap_or(0)` of the linear scan it replaced.
fn entry_index(candles: &[crate::models::chart::Candle], entry_timestamp: i64) -> usize {
    let ep = candles.partition_point(|c| c.timestamp < entry_timestamp);
    if ep == candles.len() { 0 } else { ep }
}

/// Extremes since entry for the open position in `ctx`.
///
/// The engine folds these once per bar and hands them over on the context, so
/// every trailing condition on a position reads one running value rather than
/// rescanning the candle history itself. Contexts built outside the engine's bar
/// loop carry no extremes, so those fall back to a scan from the entry bar.
fn position_extremes(
    ctx: &StrategyContext,
    pos: &crate::backtesting::position::Position,
) -> PositionExtremes {
    ctx.extremes.copied().unwrap_or_else(|| {
        let entry_idx = entry_index(ctx.candles, pos.entry_timestamp);
        PositionExtremes::from_candles(&ctx.candles[entry_idx..=ctx.index])
            .unwrap_or_else(|| PositionExtremes::new(ctx.current_candle()))
    })
}

/// Condition: trailing stop triggered when price retraces from peak/trough.
///
/// For long positions: tracks the highest price since entry and triggers
/// when price falls by `trail_pct` from that high.
///
/// For short positions: tracks the lowest price since entry and triggers
/// when price rises by `trail_pct` from that low.
///
/// # Execution Model
///
/// The peak/trough is computed from bar **highs/lows** since entry, but the
/// trigger test uses the **bar close**. The exit signal is deferred to the
/// **next bar's open** (identical to all strategy-signal exits).
///
/// For intrabar enforcement, use [`BacktestConfig::trailing_stop_pct`] instead,
/// which fills on the same bar when the trailing level is breached intraday.
///
/// # Example
///
/// ```ignore
/// use finance_query::backtesting::condition::*;
///
/// // Exit if price drops 3% from highest point since entry
/// let exit = trailing_stop(0.03);
/// ```
#[derive(Debug, Clone, Copy)]
pub struct TrailingStop {
    /// Trail percentage (e.g., 0.03 for 3%)
    pub trail_pct: f64,
}

impl TrailingStop {
    /// Create a new trailing stop condition.
    ///
    /// # Arguments
    ///
    /// * `trail_pct` - Trail percentage (e.g., 0.03 for 3%)
    pub fn new(trail_pct: f64) -> Self {
        Self { trail_pct }
    }
}

impl Condition for TrailingStop {
    fn evaluate(&self, ctx: &StrategyContext) -> bool {
        if let Some(pos) = ctx.position {
            let current_close = ctx.close();
            let PositionExtremes {
                high: peak,
                low: trough,
                ..
            } = position_extremes(ctx, pos);

            match pos.side {
                crate::backtesting::position::PositionSide::Long => {
                    current_close <= peak * (1.0 - self.trail_pct)
                }
                crate::backtesting::position::PositionSide::Short => {
                    current_close >= trough * (1.0 + self.trail_pct)
                }
            }
        } else {
            false
        }
    }

    fn required_indicators(&self) -> Vec<(String, Indicator)> {
        vec![]
    }

    fn description(&self) -> String {
        format!("trailing stop at {:.1}%", self.trail_pct * 100.0)
    }

    fn tracks_position_extremes(&self) -> bool {
        true
    }
}

/// Create a trailing stop condition.
///
/// The trailing stop tracks the best price (highest for longs, lowest for shorts)
/// since position entry and triggers when price retraces by the specified percentage.
///
/// # Example
///
/// ```ignore
/// use finance_query::backtesting::condition::*;
///
/// // Exit if price drops 3% from the highest point since entry
/// let exit = trailing_stop(0.03);
/// ```
#[inline]
pub fn trailing_stop(trail_pct: f64) -> TrailingStop {
    TrailingStop::new(trail_pct)
}

/// Condition: trailing take-profit triggered when profit retraces from peak.
///
/// For long positions: tracks the highest profit since entry and triggers
/// when profit falls by `trail_pct` from that peak profit.
///
/// For short positions: tracks the highest profit since entry and triggers
/// when profit falls by `trail_pct` from that peak profit.
///
/// This is useful for locking in gains - it only triggers after you've been
/// in profit and then profit starts declining.
///
/// # Example
///
/// ```ignore
/// use finance_query::backtesting::condition::*;
///
/// // Exit if profit drops 2% from highest profit achieved
/// let exit = trailing_take_profit(0.02);
/// ```
#[derive(Debug, Clone, Copy)]
pub struct TrailingTakeProfit {
    /// Trail percentage from peak profit (e.g., 0.02 for 2%)
    pub trail_pct: f64,
}

impl TrailingTakeProfit {
    /// Create a new trailing take-profit condition.
    ///
    /// # Arguments
    ///
    /// * `trail_pct` - Trail percentage from peak profit (e.g., 0.02 for 2%)
    pub fn new(trail_pct: f64) -> Self {
        Self { trail_pct }
    }
}

impl Condition for TrailingTakeProfit {
    fn evaluate(&self, ctx: &StrategyContext) -> bool {
        if let Some(pos) = ctx.position {
            let PositionExtremes {
                close_high,
                close_low,
                ..
            } = position_extremes(ctx, pos);
            // unrealized_return_pct is monotone in price, so the peak profit is
            // reached at the window's extreme close for the position's side.
            let best_close = match pos.side {
                crate::backtesting::position::PositionSide::Long => close_high,
                crate::backtesting::position::PositionSide::Short => close_low,
            };
            let peak_profit_pct = pos.unrealized_return_pct(best_close);

            // Only trigger if we've been in profit and current profit is below peak by trail_pct
            let current_profit_pct = pos.unrealized_return_pct(ctx.close());

            // Convert trail_pct to percentage points (e.g., 0.02 -> 2.0 percentage points)
            let trail_threshold = self.trail_pct * 100.0;

            peak_profit_pct > 0.0 && current_profit_pct <= peak_profit_pct - trail_threshold
        } else {
            false
        }
    }

    fn required_indicators(&self) -> Vec<(String, Indicator)> {
        vec![]
    }

    fn description(&self) -> String {
        format!("trailing take profit at {:.1}%", self.trail_pct * 100.0)
    }

    fn tracks_position_extremes(&self) -> bool {
        true
    }
}

/// Create a trailing take-profit condition.
///
/// This condition tracks the peak profit since entry and triggers when
/// profit drops by the specified percentage from that peak. It only triggers
/// after the position has been in profit.
///
/// # Example
///
/// ```ignore
/// use finance_query::backtesting::condition::*;
///
/// // Exit if profit drops 2% from peak profit
/// let exit = trailing_take_profit(0.02);
/// ```
#[inline]
pub fn trailing_take_profit(trail_pct: f64) -> TrailingTakeProfit {
    TrailingTakeProfit::new(trail_pct)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ramp(n: usize) -> Vec<crate::models::chart::Candle> {
        (0..n)
            .map(|i| {
                let px = 100.0 + i as f64;
                crate::models::chart::Candle {
                    timestamp: 1_600_000_000 + i as i64 * 86400,
                    open: px,
                    high: px * 1.02,
                    low: px * 0.98,
                    close: px,
                    volume: 1_000,
                    ..Default::default()
                }
            })
            .collect()
    }

    #[test]
    fn entry_index_matches_linear_scan() {
        let candles = ramp(200);
        let probes = [
            0i64,
            candles[0].timestamp,
            candles[7].timestamp,
            candles[7].timestamp + 1,
            candles[199].timestamp,
            candles[199].timestamp + 10_000,
        ];
        for entry_ts in probes {
            let linear = candles
                .iter()
                .position(|c| c.timestamp >= entry_ts)
                .unwrap_or(0);
            let ep = candles.partition_point(|c| c.timestamp < entry_ts);
            let binary = if ep == candles.len() { 0 } else { ep };
            assert_eq!(linear, binary, "mismatch for entry_ts={entry_ts}");
        }
    }

    fn peak_then_drop(n: usize) -> Vec<crate::models::chart::Candle> {
        (0..n)
            .map(|i| {
                let half = n / 2;
                let px = if i < half {
                    100.0 + i as f64
                } else {
                    100.0 + half as f64 - (i - half) as f64
                };
                crate::models::chart::Candle {
                    timestamp: 1_600_000_000 + i as i64 * 86400,
                    open: px,
                    high: px * 1.02,
                    low: px * 0.98,
                    close: px,
                    volume: 1_000,
                    ..Default::default()
                }
            })
            .collect()
    }

    #[test]
    fn trailing_stop_exit_bars_are_stable() {
        use crate::backtesting::refs::*;
        use crate::backtesting::{BacktestConfig, BacktestEngine, StrategyBuilder};

        let candles = peak_then_drop(400);
        let strat = StrategyBuilder::new("t")
            .entry(price().above(0.0))
            .exit(trailing_stop(0.03))
            .build();
        let result = BacktestEngine::new(BacktestConfig::default())
            .run("TEST", &candles, strat)
            .unwrap();
        let actual: Vec<(i64, i64)> = result
            .trades
            .iter()
            .map(|t| (t.entry_timestamp, t.exit_timestamp))
            .collect();
        let expected: Vec<(i64, i64)> = vec![
            (1600172800, 1617712000),
            (1617712000, 1618144000),
            (1618144000, 1618576000),
            (1618576000, 1619008000),
            (1619008000, 1619353600),
            (1619353600, 1619699200),
            (1619699200, 1620044800),
            (1620044800, 1620390400),
            (1620390400, 1620736000),
            (1620736000, 1621081600),
            (1621081600, 1621427200),
            (1621427200, 1621772800),
            (1621772800, 1622118400),
            (1622118400, 1622464000),
            (1622464000, 1622809600),
            (1622809600, 1623155200),
            (1623155200, 1623500800),
            (1623500800, 1623846400),
            (1623846400, 1624192000),
            (1624192000, 1624537600),
            (1624537600, 1624883200),
            (1624883200, 1625228800),
            (1625228800, 1625574400),
            (1625574400, 1625920000),
            (1625920000, 1626265600),
            (1626265600, 1626611200),
            (1626611200, 1626956800),
            (1626956800, 1627216000),
            (1627216000, 1627475200),
            (1627475200, 1627734400),
            (1627734400, 1627993600),
            (1627993600, 1628252800),
            (1628252800, 1628512000),
            (1628512000, 1628771200),
            (1628771200, 1629030400),
            (1629030400, 1629289600),
            (1629289600, 1629548800),
            (1629548800, 1629808000),
            (1629808000, 1630067200),
            (1630067200, 1630326400),
            (1630326400, 1630585600),
            (1630585600, 1630844800),
            (1630844800, 1631104000),
            (1631104000, 1631363200),
            (1631363200, 1631622400),
            (1631622400, 1631881600),
            (1631881600, 1632140800),
            (1632140800, 1632400000),
            (1632400000, 1632659200),
            (1632659200, 1632918400),
            (1632918400, 1633177600),
            (1633177600, 1633436800),
            (1633436800, 1633696000),
            (1633696000, 1633955200),
            (1633955200, 1634214400),
            (1634214400, 1634473600),
            (1634473600, 1634473600),
        ];
        assert_eq!(actual, expected);
    }

    #[test]
    fn trailing_conditions_stay_copy() {
        // The peak lives on the position, not the condition, so these carry no
        // state and remain plain `Copy` value types.
        fn assert_copy<T: Copy>(_: &T) {}
        assert_copy(&TrailingStop::new(0.05));
        assert_copy(&TrailingTakeProfit::new(0.05));

        let ts = TrailingStop::new(0.05);
        let a = ts;
        let b = ts;
        assert_eq!(a.trail_pct, b.trail_pct);
    }

    #[test]
    fn only_trailing_strategies_opt_into_extremes_tracking() {
        // The engine folds extremes per bar only when this reports true, so a
        // trailing condition that loses the flag silently falls back to the
        // O(bars²) rescan this PR removed — and a strategy without one would
        // otherwise pay for a value nothing reads.
        use crate::backtesting::refs::*;
        use crate::backtesting::strategy::{Strategy, StrategyBuilder};

        assert!(TrailingStop::new(0.05).tracks_position_extremes());
        assert!(TrailingTakeProfit::new(0.05).tracks_position_extremes());
        assert!(!stop_loss(0.05).tracks_position_extremes());

        // Composites have to carry it through, in either position.
        assert!(
            stop_loss(0.05)
                .or(trailing_stop(0.03))
                .tracks_position_extremes()
        );
        assert!(
            trailing_stop(0.03)
                .and(in_profit())
                .tracks_position_extremes()
        );
        assert!(
            !stop_loss(0.05)
                .or(take_profit(0.1))
                .tracks_position_extremes()
        );

        // ...and so does the strategy built from them.
        let trailing = StrategyBuilder::new("trailing")
            .entry(price().above(0.0))
            .exit(trailing_stop(0.03))
            .build();
        assert!(trailing.tracks_position_extremes());

        let plain = StrategyBuilder::new("plain")
            .entry(price().above(0.0))
            .exit(stop_loss(0.05))
            .build();
        assert!(
            !plain.tracks_position_extremes(),
            "a strategy with no trailing condition must not pay for the tracking"
        );
    }

    #[test]
    fn context_extremes_match_a_scan_from_entry() {
        // Conditions read the engine's running extremes when present and fall
        // back to scanning from the entry bar when they aren't. Both paths must
        // produce the same verdict.
        let candles = ramp(30);
        let entry_idx = 5usize;
        let index = 20usize;
        let position = crate::backtesting::position::Position::new(
            crate::backtesting::position::PositionSide::Long,
            candles[entry_idx].timestamp,
            candles[entry_idx].close,
            10.0,
            0.0,
            crate::backtesting::signal::Signal::long(
                candles[entry_idx].timestamp,
                candles[entry_idx].close,
            ),
        );
        let indicators = std::collections::HashMap::new();
        let scanned = PositionExtremes::from_candles(&candles[entry_idx..=index]).unwrap();

        for pct in [0.001, 0.01, 0.05, 0.5] {
            let cond = TrailingStop::new(pct);
            let tp = TrailingTakeProfit::new(pct);
            let with = StrategyContext {
                candles: &candles[..=index],
                index,
                position: Some(&position),
                equity: 10_000.0,
                indicators: &indicators,
                extremes: Some(&scanned),
            };
            let without = StrategyContext {
                candles: &candles[..=index],
                index,
                position: Some(&position),
                equity: 10_000.0,
                indicators: &indicators,
                extremes: None,
            };
            assert_eq!(
                cond.evaluate(&with),
                cond.evaluate(&without),
                "trailing stop disagreed at {pct}"
            );
            assert_eq!(
                tp.evaluate(&with),
                tp.evaluate(&without),
                "trailing take-profit disagreed at {pct}"
            );
        }
    }

    #[test]
    fn test_stop_loss_description() {
        let sl = stop_loss(0.05);
        assert_eq!(sl.description(), "stop loss at 5.0%");
    }

    #[test]
    fn test_take_profit_description() {
        let tp = take_profit(0.10);
        assert_eq!(tp.description(), "take profit at 10.0%");
    }

    #[test]
    fn test_position_conditions_descriptions() {
        assert_eq!(has_position().description(), "has position");
        assert_eq!(no_position().description(), "no position");
        assert_eq!(is_long().description(), "is long");
        assert_eq!(is_short().description(), "is short");
        assert_eq!(in_profit().description(), "in profit");
        assert_eq!(in_loss().description(), "in loss");
    }

    #[test]
    fn test_held_for_bars_description() {
        let hfb = held_for_bars(5);
        assert_eq!(hfb.description(), "held for 5 bars");
    }

    #[test]
    fn test_trailing_stop_description() {
        let ts = trailing_stop(0.03);
        assert_eq!(ts.description(), "trailing stop at 3.0%");
    }

    #[test]
    fn test_trailing_take_profit_description() {
        let ttp = trailing_take_profit(0.02);
        assert_eq!(ttp.description(), "trailing take profit at 2.0%");
    }

    #[test]
    fn test_no_indicators_required() {
        assert!(stop_loss(0.05).required_indicators().is_empty());
        assert!(take_profit(0.10).required_indicators().is_empty());
        assert!(has_position().required_indicators().is_empty());
        assert!(no_position().required_indicators().is_empty());
        assert!(trailing_stop(0.03).required_indicators().is_empty());
        assert!(trailing_take_profit(0.02).required_indicators().is_empty());
    }

    /// The `TrailingTakeProfit` fast path folds closes into a single extreme and
    /// evaluates `unrealized_return_pct` once, instead of evaluating it per bar
    /// and folding the results. That is only exact because the function is
    /// monotone in price — increasing for longs, decreasing for shorts. Pin it.
    #[test]
    fn peak_profit_from_extreme_close_matches_per_bar_fold() {
        use crate::backtesting::position::{Position, PositionSide};
        use crate::backtesting::signal::Signal;

        let closes: Vec<f64> = (0..300)
            .map(|i| 100.0 + (i as f64 * 0.37).sin() * 25.0 + (i as f64 * 0.011))
            .collect();

        for side in [PositionSide::Long, PositionSide::Short] {
            for entry_price in [1.0_f64, 87.5, 100.0, 133.25] {
                let pos = Position::new(
                    side,
                    1_600_000_000,
                    entry_price,
                    7.0,
                    0.0,
                    Signal::long(1_600_000_000, entry_price),
                );

                let per_bar = closes
                    .iter()
                    .map(|&c| pos.unrealized_return_pct(c))
                    .fold(f64::NEG_INFINITY, f64::max);

                let extreme = match side {
                    PositionSide::Long => closes.iter().copied().fold(f64::NEG_INFINITY, f64::max),
                    PositionSide::Short => closes.iter().copied().fold(f64::INFINITY, f64::min),
                };
                let single = pos.unrealized_return_pct(extreme);

                assert_eq!(
                    per_bar, single,
                    "side={side:?} entry={entry_price}: fold-of-f != f-of-extreme"
                );
            }
        }
    }
}
