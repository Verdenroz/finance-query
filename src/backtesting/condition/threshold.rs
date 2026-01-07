//! Threshold-based conditions for position management.
//!
//! This module provides conditions for stop-loss, take-profit, and trailing stops.

use crate::backtesting::strategy::StrategyContext;
use crate::indicators::Indicator;

use super::Condition;

/// Condition: position P/L is at or below the stop-loss threshold.
///
/// # Example
///
/// ```ignore
/// use finance_query::backtesting::condition::*;
///
/// let exit = stop_loss(0.05); // Exit if loss >= 5%
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
/// # Example
///
/// ```ignore
/// use finance_query::backtesting::condition::*;
///
/// let exit = take_profit(0.10); // Exit if gain >= 10%
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
            let entry_idx = ctx
                .candles
                .iter()
                .position(|c| c.timestamp >= pos.entry_timestamp)
                .unwrap_or(0);
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

/// Condition: trailing stop triggered when price retraces from peak/trough.
///
/// For long positions: tracks the highest price since entry and triggers
/// when price falls by `trail_pct` from that high.
///
/// For short positions: tracks the lowest price since entry and triggers
/// when price rises by `trail_pct` from that low.
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
            // Find the entry index
            let entry_idx = ctx
                .candles
                .iter()
                .position(|c| c.timestamp >= pos.entry_timestamp)
                .unwrap_or(0);

            // Compute peak/trough from entry to current candle (inclusive)
            let current_close = ctx.close();

            match pos.side {
                crate::backtesting::position::PositionSide::Long => {
                    // For long: find highest high since entry
                    let peak = ctx.candles[entry_idx..=ctx.index]
                        .iter()
                        .map(|c| c.high)
                        .fold(f64::NEG_INFINITY, f64::max);

                    // Trigger if current price is trail_pct below peak
                    current_close <= peak * (1.0 - self.trail_pct)
                }
                crate::backtesting::position::PositionSide::Short => {
                    // For short: find lowest low since entry
                    let trough = ctx.candles[entry_idx..=ctx.index]
                        .iter()
                        .map(|c| c.low)
                        .fold(f64::INFINITY, f64::min);

                    // Trigger if current price is trail_pct above trough
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
            // Find the entry index
            let entry_idx = ctx
                .candles
                .iter()
                .position(|c| c.timestamp >= pos.entry_timestamp)
                .unwrap_or(0);

            // Compute peak profit from entry to current
            let peak_profit_pct = ctx.candles[entry_idx..=ctx.index]
                .iter()
                .map(|c| pos.unrealized_return_pct(c.close))
                .fold(f64::NEG_INFINITY, f64::max);

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
}
