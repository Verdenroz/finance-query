//! Strategy trait and context for building trading strategies.
//!
//! This module provides the core `Strategy` trait and `StrategyContext` for
//! implementing custom trading strategies, as well as pre-built strategies
//! and a fluent builder API.
//!
//! # Building Custom Strategies
//!
//! Use the `StrategyBuilder` for creating strategies with conditions:
//!
//! ```ignore
//! use finance_query::backtesting::strategy::StrategyBuilder;
//! use finance_query::backtesting::refs::*;
//! use finance_query::backtesting::condition::*;
//!
//! let strategy = StrategyBuilder::new("My Strategy")
//!     .entry(rsi(14).crosses_below(30.0))
//!     .exit(rsi(14).crosses_above(70.0).or(stop_loss(0.05)))
//!     .build();
//! ```

mod builder;
pub mod prebuilt;

use std::collections::HashMap;

use crate::indicators::Indicator;
use crate::models::chart::Candle;

use super::position::{Position, PositionSide};
use super::signal::Signal;

// Re-export builder
pub use builder::{CustomStrategy, StrategyBuilder};

// Re-export prebuilt strategies
pub use prebuilt::{
    BollingerMeanReversion, DonchianBreakout, MacdSignal, RsiReversal, SmaCrossover,
    SuperTrendFollow,
};

/// Context passed to strategy on each candle.
///
/// Provides access to historical data, current position, and pre-computed indicators.
#[non_exhaustive]
pub struct StrategyContext<'a> {
    /// All candles up to and including current
    pub candles: &'a [Candle],

    /// Current candle index (0-based)
    pub index: usize,

    /// Current position (if any)
    pub position: Option<&'a Position>,

    /// Current portfolio equity
    pub equity: f64,

    /// Pre-computed indicator values (keyed by indicator name)
    pub indicators: &'a HashMap<String, Vec<Option<f64>>>,
}

impl<'a> StrategyContext<'a> {
    /// Get current candle
    pub fn current_candle(&self) -> &Candle {
        &self.candles[self.index]
    }

    /// Get previous candle (None if at start)
    pub fn previous_candle(&self) -> Option<&Candle> {
        if self.index > 0 {
            Some(&self.candles[self.index - 1])
        } else {
            None
        }
    }

    /// Get candle at specific index (None if out of bounds)
    pub fn candle_at(&self, index: usize) -> Option<&Candle> {
        self.candles.get(index)
    }

    /// Get indicator value at current index
    pub fn indicator(&self, name: &str) -> Option<f64> {
        self.indicators
            .get(name)
            .and_then(|v| v.get(self.index))
            .and_then(|&v| v)
    }

    /// Get indicator value at specific index
    pub fn indicator_at(&self, name: &str, index: usize) -> Option<f64> {
        self.indicators
            .get(name)
            .and_then(|v| v.get(index))
            .and_then(|&v| v)
    }

    /// Get indicator value at previous index
    pub fn indicator_prev(&self, name: &str) -> Option<f64> {
        if self.index > 0 {
            self.indicator_at(name, self.index - 1)
        } else {
            None
        }
    }

    /// Check if we have a position
    pub fn has_position(&self) -> bool {
        self.position.is_some()
    }

    /// Check if we have a long position
    pub fn is_long(&self) -> bool {
        self.position
            .map(|p| matches!(p.side, PositionSide::Long))
            .unwrap_or(false)
    }

    /// Check if we have a short position
    pub fn is_short(&self) -> bool {
        self.position
            .map(|p| matches!(p.side, PositionSide::Short))
            .unwrap_or(false)
    }

    /// Get current close price
    pub fn close(&self) -> f64 {
        self.current_candle().close
    }

    /// Get current open price
    pub fn open(&self) -> f64 {
        self.current_candle().open
    }

    /// Get current high price
    pub fn high(&self) -> f64 {
        self.current_candle().high
    }

    /// Get current low price
    pub fn low(&self) -> f64 {
        self.current_candle().low
    }

    /// Get current volume
    pub fn volume(&self) -> i64 {
        self.current_candle().volume
    }

    /// Get current timestamp
    pub fn timestamp(&self) -> i64 {
        self.current_candle().timestamp
    }

    /// Create a Long signal from the current candle's timestamp and close price.
    pub fn signal_long(&self) -> Signal {
        Signal::long(self.timestamp(), self.close())
    }

    /// Create a Short signal from the current candle's timestamp and close price.
    pub fn signal_short(&self) -> Signal {
        Signal::short(self.timestamp(), self.close())
    }

    /// Create an Exit signal from the current candle's timestamp and close price.
    pub fn signal_exit(&self) -> Signal {
        Signal::exit(self.timestamp(), self.close())
    }

    /// Check if crossover occurred (fast crosses above slow)
    pub fn crossed_above(&self, fast_name: &str, slow_name: &str) -> bool {
        let fast_now = self.indicator(fast_name);
        let slow_now = self.indicator(slow_name);
        let fast_prev = self.indicator_prev(fast_name);
        let slow_prev = self.indicator_prev(slow_name);

        match (fast_now, slow_now, fast_prev, slow_prev) {
            (Some(f), Some(s), Some(fp), Some(sp)) => fp < sp && f > s, // Fixed: changed <= to <
            _ => false,
        }
    }

    /// Check if crossover occurred (fast crosses below slow)
    pub fn crossed_below(&self, fast_name: &str, slow_name: &str) -> bool {
        let fast_now = self.indicator(fast_name);
        let slow_now = self.indicator(slow_name);
        let fast_prev = self.indicator_prev(fast_name);
        let slow_prev = self.indicator_prev(slow_name);

        match (fast_now, slow_now, fast_prev, slow_prev) {
            (Some(f), Some(s), Some(fp), Some(sp)) => fp > sp && f < s, // Fixed: changed >= to >
            _ => false,
        }
    }

    /// Check if indicator crossed above a threshold.
    ///
    /// Returns `true` when `prev <= threshold` **and** `current > threshold`.
    /// The inclusive lower bound (`<=`) means a signal fires even when the
    /// previous bar sat exactly on the threshold, which is the conventional
    /// "crosses above" definition.  This is intentionally asymmetric with the
    /// strict crossover check in [`crossed_above`](Self::crossed_above) where
    /// both sides use strict inequalities — threshold crossings and
    /// indicator-vs-indicator crossings have different semantics.
    pub fn indicator_crossed_above(&self, name: &str, threshold: f64) -> bool {
        let now = self.indicator(name);
        let prev = self.indicator_prev(name);

        match (now, prev) {
            (Some(n), Some(p)) => p <= threshold && n > threshold,
            _ => false,
        }
    }

    /// Check if indicator crossed below a threshold.
    ///
    /// Returns `true` when `prev >= threshold` **and** `current < threshold`.
    /// See [`indicator_crossed_above`](Self::indicator_crossed_above) for the
    /// rationale behind the inclusive/exclusive choice on each side.
    pub fn indicator_crossed_below(&self, name: &str, threshold: f64) -> bool {
        let now = self.indicator(name);
        let prev = self.indicator_prev(name);

        match (now, prev) {
            (Some(n), Some(p)) => p >= threshold && n < threshold,
            _ => false,
        }
    }
}

/// Core strategy trait - implement this for custom strategies.
///
/// # Example
///
/// ```ignore
/// use finance_query::backtesting::{Strategy, StrategyContext, Signal};
/// use finance_query::indicators::Indicator;
///
/// struct MyStrategy {
///     sma_period: usize,
/// }
///
/// impl Strategy for MyStrategy {
///     fn name(&self) -> &str {
///         "My Custom Strategy"
///     }
///
///     fn required_indicators(&self) -> Vec<(String, Indicator)> {
///         vec![
///             (format!("sma_{}", self.sma_period), Indicator::Sma(self.sma_period)),
///         ]
///     }
///
///     fn on_candle(&self, ctx: &StrategyContext) -> Signal {
///         let sma = ctx.indicator(&format!("sma_{}", self.sma_period));
///         let close = ctx.close();
///
///         match sma {
///             Some(sma_val) if close > sma_val && !ctx.has_position() => {
///                 Signal::long(ctx.timestamp(), close)
///             }
///             Some(sma_val) if close < sma_val && ctx.is_long() => {
///                 Signal::exit(ctx.timestamp(), close)
///             }
///             _ => Signal::hold(),
///         }
///     }
/// }
/// ```
pub trait Strategy: Send + Sync {
    /// Strategy name (for reporting)
    fn name(&self) -> &str;

    /// Required indicators this strategy needs.
    ///
    /// Returns list of (indicator_name, Indicator) pairs.
    /// The engine will pre-compute these and make them available via `StrategyContext::indicator()`.
    fn required_indicators(&self) -> Vec<(String, Indicator)>;

    /// Called on each candle to generate a signal.
    ///
    /// Return `Signal::hold()` for no action, `Signal::long()` to enter long,
    /// `Signal::short()` to enter short, or `Signal::exit()` to close position.
    fn on_candle(&self, ctx: &StrategyContext) -> Signal;

    /// Optional: minimum candles required before strategy can generate signals.
    /// Default is 1 (strategy can run from first candle).
    fn warmup_period(&self) -> usize {
        1
    }
}

impl Strategy for Box<dyn Strategy> {
    fn name(&self) -> &str {
        (**self).name()
    }
    fn required_indicators(&self) -> Vec<(String, Indicator)> {
        (**self).required_indicators()
    }
    fn on_candle(&self, ctx: &StrategyContext) -> Signal {
        (**self).on_candle(ctx)
    }
    fn warmup_period(&self) -> usize {
        (**self).warmup_period()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct TestStrategy;

    impl Strategy for TestStrategy {
        fn name(&self) -> &str {
            "Test Strategy"
        }

        fn required_indicators(&self) -> Vec<(String, Indicator)> {
            vec![("sma_10".to_string(), Indicator::Sma(10))]
        }

        fn on_candle(&self, ctx: &StrategyContext) -> Signal {
            if ctx.index == 5 {
                Signal::long(ctx.timestamp(), ctx.close())
            } else {
                Signal::hold()
            }
        }
    }

    #[test]
    fn test_strategy_trait() {
        let strategy = TestStrategy;
        assert_eq!(strategy.name(), "Test Strategy");
        assert_eq!(strategy.required_indicators().len(), 1);
        assert_eq!(strategy.warmup_period(), 1);
    }

    #[test]
    fn test_context_crossover_detection() {
        let candles = vec![
            Candle {
                timestamp: 1,
                open: 100.0,
                high: 101.0,
                low: 99.0,
                close: 100.0,
                volume: 1000,
                adj_close: None,
            },
            Candle {
                timestamp: 2,
                open: 100.0,
                high: 102.0,
                low: 99.0,
                close: 101.0,
                volume: 1000,
                adj_close: None,
            },
        ];

        let mut indicators = HashMap::new();
        indicators.insert("fast".to_string(), vec![Some(9.0), Some(11.0)]);
        indicators.insert("slow".to_string(), vec![Some(10.0), Some(10.0)]);

        let ctx = StrategyContext {
            candles: &candles,
            index: 1,
            position: None,
            equity: 10000.0,
            indicators: &indicators,
        };

        // fast was 9 (below slow 10), now 11 (above slow 10) -> crossed above
        assert!(ctx.crossed_above("fast", "slow"));
        assert!(!ctx.crossed_below("fast", "slow"));
    }
}
