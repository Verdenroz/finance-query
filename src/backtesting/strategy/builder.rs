//! Fluent strategy builder for creating custom strategies from conditions.
//!
//! This module provides a builder pattern for creating custom trading strategies
//! using entry and exit conditions.
//!
//! # Example
//!
//! ```ignore
//! use finance_query::backtesting::strategy::StrategyBuilder;
//! use finance_query::backtesting::refs::*;
//! use finance_query::backtesting::condition::*;
//!
//! let strategy = StrategyBuilder::new("RSI Mean Reversion")
//!     .entry(
//!         rsi(14).crosses_below(30.0)
//!             .and(price().above_ref(sma(200)))
//!     )
//!     .exit(
//!         rsi(14).crosses_above(70.0)
//!             .or(stop_loss(0.05))
//!     )
//!     .build();
//! ```

use std::collections::HashSet;

use crate::backtesting::condition::Condition;
use crate::backtesting::signal::Signal;
use crate::indicators::Indicator;

use super::{Strategy, StrategyContext};

/// Type-erased condition wrapper for storing heterogeneous conditions.
struct BoxedCondition {
    evaluate_fn: Box<dyn Fn(&StrategyContext) -> bool + Send + Sync>,
    required_indicators: Vec<(String, Indicator)>,
    description: String,
}

impl BoxedCondition {
    fn new<C: Condition>(cond: C) -> Self {
        let required_indicators = cond.required_indicators();
        let description = cond.description();
        Self {
            evaluate_fn: Box::new(move |ctx| cond.evaluate(ctx)),
            required_indicators,
            description,
        }
    }

    fn evaluate(&self, ctx: &StrategyContext) -> bool {
        (self.evaluate_fn)(ctx)
    }

    fn required_indicators(&self) -> &[(String, Indicator)] {
        &self.required_indicators
    }

    fn description(&self) -> &str {
        &self.description
    }
}

/// Builder for creating custom strategies with entry/exit conditions.
///
/// The builder enforces that both entry and exit conditions are provided
/// before a strategy can be built.
pub struct StrategyBuilder<E = (), X = ()> {
    name: String,
    entry_condition: E,
    exit_condition: X,
    short_entry_condition: Option<BoxedCondition>,
    short_exit_condition: Option<BoxedCondition>,
    warmup_override: Option<usize>,
}

impl StrategyBuilder<(), ()> {
    /// Create a new strategy builder with a name.
    ///
    /// # Example
    ///
    /// ```ignore
    /// let builder = StrategyBuilder::new("My Strategy");
    /// ```
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            entry_condition: (),
            exit_condition: (),
            short_entry_condition: None,
            short_exit_condition: None,
            warmup_override: None,
        }
    }
}

impl<X> StrategyBuilder<(), X> {
    /// Set the entry condition for long positions.
    ///
    /// # Example
    ///
    /// ```ignore
    /// let builder = StrategyBuilder::new("RSI Strategy")
    ///     .entry(rsi(14).crosses_below(30.0));
    /// ```
    pub fn entry<C: Condition>(self, condition: C) -> StrategyBuilder<C, X> {
        StrategyBuilder {
            name: self.name,
            entry_condition: condition,
            exit_condition: self.exit_condition,
            short_entry_condition: self.short_entry_condition,
            short_exit_condition: self.short_exit_condition,
            warmup_override: self.warmup_override,
        }
    }
}

impl<E> StrategyBuilder<E, ()> {
    /// Set the exit condition for long positions.
    ///
    /// # Example
    ///
    /// ```ignore
    /// let builder = StrategyBuilder::new("RSI Strategy")
    ///     .entry(rsi(14).crosses_below(30.0))
    ///     .exit(rsi(14).crosses_above(70.0));
    /// ```
    pub fn exit<C: Condition>(self, condition: C) -> StrategyBuilder<E, C> {
        StrategyBuilder {
            name: self.name,
            entry_condition: self.entry_condition,
            exit_condition: condition,
            short_entry_condition: self.short_entry_condition,
            short_exit_condition: self.short_exit_condition,
            warmup_override: self.warmup_override,
        }
    }
}

impl<E: Condition, X: Condition> StrategyBuilder<E, X> {
    /// Enable short positions with entry and exit conditions.
    ///
    /// # Example
    ///
    /// ```ignore
    /// let strategy = StrategyBuilder::new("RSI Strategy")
    ///     .entry(rsi(14).crosses_below(30.0))
    ///     .exit(rsi(14).crosses_above(70.0))
    ///     .with_short(
    ///         rsi(14).crosses_above(70.0),  // Short entry
    ///         rsi(14).crosses_below(30.0),  // Short exit
    ///     )
    ///     .build();
    /// ```
    pub fn with_short<SE: Condition, SX: Condition>(mut self, entry: SE, exit: SX) -> Self {
        self.short_entry_condition = Some(BoxedCondition::new(entry));
        self.short_exit_condition = Some(BoxedCondition::new(exit));
        self
    }

    /// Override the automatic warmup period with an explicit bar count.
    ///
    /// By default the warmup period is inferred from each indicator's
    /// [`Indicator::warmup_bars()`] method. Use this override when the
    /// automatic value doesn't match your specific needs.
    ///
    /// # Example
    ///
    /// ```ignore
    /// let strategy = StrategyBuilder::new("MACD + RSI")
    ///     .entry(macd(12, 26, 9).crosses_above_zero())
    ///     .exit(rsi(14).crosses_above(70.0))
    ///     .warmup(36) // explicit override
    ///     .build();
    /// ```
    pub fn warmup(mut self, bars: usize) -> Self {
        self.warmup_override = Some(bars);
        self
    }

    /// Build the strategy.
    ///
    /// # Example
    ///
    /// ```ignore
    /// let strategy = StrategyBuilder::new("My Strategy")
    ///     .entry(rsi(14).crosses_below(30.0))
    ///     .exit(rsi(14).crosses_above(70.0))
    ///     .build();
    /// ```
    pub fn build(self) -> CustomStrategy<E, X> {
        CustomStrategy {
            name: self.name,
            entry_condition: self.entry_condition,
            exit_condition: self.exit_condition,
            short_entry_condition: self.short_entry_condition,
            short_exit_condition: self.short_exit_condition,
            warmup_override: self.warmup_override,
        }
    }
}

/// A custom strategy built from conditions.
///
/// This strategy evaluates entry and exit conditions on each candle
/// and generates appropriate signals.
pub struct CustomStrategy<E: Condition, X: Condition> {
    name: String,
    entry_condition: E,
    exit_condition: X,
    short_entry_condition: Option<BoxedCondition>,
    short_exit_condition: Option<BoxedCondition>,
    /// Explicit warmup period set via [`StrategyBuilder::warmup`].
    ///
    /// Overrides the heuristic in [`warmup_period`] when set.
    warmup_override: Option<usize>,
}

impl<E: Condition, X: Condition> Strategy for CustomStrategy<E, X> {
    fn name(&self) -> &str {
        &self.name
    }

    fn required_indicators(&self) -> Vec<(String, Indicator)> {
        let mut indicators = self.entry_condition.required_indicators();
        indicators.extend(self.exit_condition.required_indicators());

        if let Some(ref se) = self.short_entry_condition {
            indicators.extend(se.required_indicators().iter().cloned());
        }
        if let Some(ref sx) = self.short_exit_condition {
            indicators.extend(sx.required_indicators().iter().cloned());
        }

        // Deduplicate by key
        let mut seen = HashSet::new();
        indicators.retain(|(key, _)| seen.insert(key.clone()));

        indicators
    }

    fn warmup_period(&self) -> usize {
        // Explicit override wins — use it directly.
        if let Some(n) = self.warmup_override {
            return n;
        }

        // Use each indicator's own warmup calculation instead of parsing
        // key suffixes (which fails for compound indicators like MACD and
        // Bollinger).  `.warmup(n)` on the builder still overrides this.
        let max_warmup = self
            .required_indicators()
            .iter()
            .map(|(_, indicator)| indicator.warmup_bars())
            .max()
            .unwrap_or(1);

        max_warmup + 1
    }

    fn on_candle(&self, ctx: &StrategyContext) -> Signal {
        let candle = ctx.current_candle();

        // Check exit conditions first (for existing positions)
        if ctx.is_long() && self.exit_condition.evaluate(ctx) {
            return Signal::exit(candle.timestamp, candle.close)
                .with_reason(self.exit_condition.description());
        }

        if ctx.is_short()
            && let Some(ref exit) = self.short_exit_condition
            && exit.evaluate(ctx)
        {
            return Signal::exit(candle.timestamp, candle.close)
                .with_reason(exit.description().to_string());
        }

        // Check entry conditions (when no position)
        if !ctx.has_position() {
            // Long entry
            if self.entry_condition.evaluate(ctx) {
                return Signal::long(candle.timestamp, candle.close)
                    .with_reason(self.entry_condition.description());
            }

            // Short entry
            if let Some(ref entry) = self.short_entry_condition
                && entry.evaluate(ctx)
            {
                return Signal::short(candle.timestamp, candle.close)
                    .with_reason(entry.description().to_string());
            }
        }

        Signal::hold()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::backtesting::condition::{always_false, always_true};

    #[test]
    fn test_strategy_builder() {
        let strategy = StrategyBuilder::new("Test Strategy")
            .entry(always_true())
            .exit(always_false())
            .build();

        assert_eq!(strategy.name(), "Test Strategy");
    }

    #[test]
    fn test_strategy_builder_with_short() {
        let strategy = StrategyBuilder::new("Test Strategy")
            .entry(always_true())
            .exit(always_false())
            .with_short(always_false(), always_true())
            .build();

        assert_eq!(strategy.name(), "Test Strategy");
        assert!(strategy.short_entry_condition.is_some());
        assert!(strategy.short_exit_condition.is_some());
    }

    #[test]
    fn test_required_indicators_deduplication() {
        use crate::backtesting::condition::Above;
        use crate::backtesting::refs::rsi;

        // Create two conditions using the same indicator
        let entry = Above::new(rsi(14), 70.0);
        let exit = Above::new(rsi(14), 30.0);

        let strategy = StrategyBuilder::new("Test").entry(entry).exit(exit).build();

        let indicators = strategy.required_indicators();
        // Should be deduplicated to just one rsi_14
        assert_eq!(indicators.len(), 1);
        assert_eq!(indicators[0].0, "rsi_14");
    }
}
