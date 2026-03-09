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

use crate::backtesting::condition::{Condition, HtfIndicatorSpec};
use crate::backtesting::signal::Signal;
use crate::indicators::Indicator;

use super::{Strategy, StrategyContext};

/// Type-erased condition wrapper for storing heterogeneous conditions.
struct BoxedCondition {
    evaluate_fn: Box<dyn Fn(&StrategyContext) -> bool + Send + Sync>,
    required_indicators: Vec<(String, Indicator)>,
    htf_requirements: Vec<HtfIndicatorSpec>,
    description: String,
}

impl BoxedCondition {
    fn new<C: Condition>(cond: C) -> Self {
        let required_indicators = cond.required_indicators();
        let htf_requirements = cond.htf_requirements();
        let description = cond.description();
        Self {
            evaluate_fn: Box::new(move |ctx| cond.evaluate(ctx)),
            required_indicators,
            htf_requirements,
            description,
        }
    }

    fn evaluate(&self, ctx: &StrategyContext) -> bool {
        (self.evaluate_fn)(ctx)
    }

    fn required_indicators(&self) -> &[(String, Indicator)] {
        &self.required_indicators
    }

    fn htf_requirements(&self) -> &[HtfIndicatorSpec] {
        &self.htf_requirements
    }

    fn description(&self) -> &str {
        &self.description
    }
}

/// Builder for creating custom strategies with entry/exit conditions.
///
/// The builder enforces that both entry and exit conditions are provided
/// before a strategy can be built.
///
/// An optional regime filter can be set at any point in the chain via
/// [`.regime_filter()`](StrategyBuilder::regime_filter). When set, the filter
/// is evaluated on every bar; if it returns `false`, all entry signals are
/// suppressed. Exit signals are **never** blocked by the regime filter.
pub struct StrategyBuilder<E = (), X = ()> {
    name: String,
    entry_condition: E,
    exit_condition: X,
    short_entry_condition: Option<BoxedCondition>,
    short_exit_condition: Option<BoxedCondition>,
    regime_filter: Option<BoxedCondition>,
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
            regime_filter: None,
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
            regime_filter: self.regime_filter,
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
            regime_filter: self.regime_filter,
            warmup_override: self.warmup_override,
        }
    }
}

impl<E, X> StrategyBuilder<E, X> {
    /// Set a market regime filter.
    ///
    /// When set, entry signals (long and short) are suppressed on any bar
    /// where the filter evaluates to `false`. Exit signals are **never**
    /// blocked by the regime filter, ensuring open positions can always be
    /// closed regardless of market conditions.
    ///
    /// The regime filter's indicators are included in `required_indicators()`
    /// and therefore pre-computed by the engine like any other indicator.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use finance_query::backtesting::strategy::StrategyBuilder;
    /// use finance_query::backtesting::refs::*;
    ///
    /// // Only trade when price is above the 200-period SMA
    /// let strategy = StrategyBuilder::new("Trend Following")
    ///     .regime_filter(sma(200).above_ref(sma(400)))
    ///     .entry(ema(10).crosses_above_ref(ema(30)))
    ///     .exit(ema(10).crosses_below_ref(ema(30)))
    ///     .build();
    /// ```
    pub fn regime_filter<C: Condition>(mut self, condition: C) -> Self {
        self.regime_filter = Some(BoxedCondition::new(condition));
        self
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
            regime_filter: self.regime_filter,
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
    /// Optional market regime filter.
    ///
    /// When `Some`, entry signals are suppressed on bars where the filter
    /// evaluates to `false`. Exit signals are unaffected.
    regime_filter: Option<BoxedCondition>,
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
        if let Some(ref rf) = self.regime_filter {
            indicators.extend(rf.required_indicators().iter().cloned());
        }

        // Deduplicate by key
        let mut seen = HashSet::new();
        indicators.retain(|(key, _)| seen.insert(key.clone()));

        indicators
    }

    fn htf_requirements(&self) -> Vec<HtfIndicatorSpec> {
        let mut reqs = self.entry_condition.htf_requirements();
        reqs.extend(self.exit_condition.htf_requirements());

        if let Some(ref se) = self.short_entry_condition {
            reqs.extend(se.htf_requirements().iter().cloned());
        }
        if let Some(ref sx) = self.short_exit_condition {
            reqs.extend(sx.htf_requirements().iter().cloned());
        }
        if let Some(ref rf) = self.regime_filter {
            reqs.extend(rf.htf_requirements().iter().cloned());
        }

        // Deduplicate by htf_key — same stretched array cannot be stored twice
        let mut seen = HashSet::new();
        reqs.retain(|spec| seen.insert(spec.htf_key.clone()));
        reqs
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
            // Regime filter gates all entries; exits are never suppressed.
            let regime_ok = self
                .regime_filter
                .as_ref()
                .is_none_or(|rf| rf.evaluate(ctx));

            if regime_ok {
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
        }

        Signal::hold()
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use super::*;
    use crate::backtesting::condition::{always_false, always_true};
    use crate::backtesting::signal::SignalDirection;
    use crate::models::chart::Candle;

    fn make_candle(ts: i64, close: f64) -> Candle {
        Candle {
            timestamp: ts,
            open: close,
            high: close,
            low: close,
            close,
            volume: 1000,
            adj_close: None,
        }
    }

    fn make_ctx<'a>(
        candles: &'a [Candle],
        indicators: &'a HashMap<String, Vec<Option<f64>>>,
    ) -> StrategyContext<'a> {
        StrategyContext {
            candles,
            index: 0,
            position: None,
            equity: 10_000.0,
            indicators,
        }
    }

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

    // ── Regime filter tests ────────────────────────────────────────────

    #[test]
    fn test_regime_filter_suppresses_entry_when_false() {
        let strategy = StrategyBuilder::new("Regime Test")
            .regime_filter(always_false()) // regime is never active
            .entry(always_true())
            .exit(always_false())
            .build();

        let candles = vec![make_candle(1, 100.0)];
        let indicators = HashMap::new();
        let ctx = make_ctx(&candles, &indicators);

        // Entry should be blocked by the regime filter
        assert_eq!(strategy.on_candle(&ctx).direction, SignalDirection::Hold);
    }

    #[test]
    fn test_regime_filter_allows_entry_when_true() {
        let strategy = StrategyBuilder::new("Regime Test")
            .regime_filter(always_true()) // regime always active
            .entry(always_true())
            .exit(always_false())
            .build();

        let candles = vec![make_candle(1, 100.0)];
        let indicators = HashMap::new();
        let ctx = make_ctx(&candles, &indicators);

        assert_eq!(strategy.on_candle(&ctx).direction, SignalDirection::Long);
    }

    #[test]
    fn test_no_regime_filter_behaves_normally() {
        let strategy = StrategyBuilder::new("No Regime")
            .entry(always_true())
            .exit(always_false())
            .build();

        let candles = vec![make_candle(1, 100.0)];
        let indicators = HashMap::new();
        let ctx = make_ctx(&candles, &indicators);

        assert_eq!(strategy.on_candle(&ctx).direction, SignalDirection::Long);
    }

    #[test]
    fn test_regime_filter_does_not_block_exit() {
        use crate::backtesting::position::{Position, PositionSide};

        let strategy = StrategyBuilder::new("Regime Exit Test")
            .regime_filter(always_false()) // regime is off
            .entry(always_false())
            .exit(always_true()) // exit condition always fires
            .build();

        let candles = vec![make_candle(1, 100.0)];
        let indicators = HashMap::new();

        // Simulate an open long position using the public constructor
        let position = Position::new(
            PositionSide::Long,
            1,
            90.0,
            10.0,
            0.0,
            Signal::long(1, 90.0),
        );

        let ctx = StrategyContext {
            candles: &candles,
            index: 0,
            position: Some(&position),
            equity: 10_000.0,
            indicators: &indicators,
        };

        // Exit must fire even though regime filter is false
        assert_eq!(strategy.on_candle(&ctx).direction, SignalDirection::Exit);
    }

    #[test]
    fn test_regime_filter_indicators_included_in_required() {
        use crate::backtesting::refs::{IndicatorRefExt, sma};
        use crate::indicators::Indicator;

        let strategy = StrategyBuilder::new("Regime Indicators")
            .regime_filter(sma(200).above_ref(sma(400)))
            .entry(always_true())
            .exit(always_false())
            .build();

        let indicators = strategy.required_indicators();
        let keys: Vec<&str> = indicators.iter().map(|(k, _)| k.as_str()).collect();

        assert!(
            keys.contains(&"sma_200"),
            "sma_200 must be in required_indicators"
        );
        assert!(
            keys.contains(&"sma_400"),
            "sma_400 must be in required_indicators"
        );

        // Verify correct Indicator variants
        let sma_200 = indicators.iter().find(|(k, _)| k == "sma_200").unwrap();
        assert!(matches!(sma_200.1, Indicator::Sma(200)));
    }

    #[test]
    fn test_regime_filter_callable_before_entry() {
        // Verify the builder chain compiles when regime_filter is called first
        let strategy = StrategyBuilder::new("Order Test")
            .regime_filter(always_true())
            .entry(always_true())
            .exit(always_false())
            .build();

        assert!(strategy.regime_filter.is_some());
    }

    #[test]
    fn test_regime_filter_warmup_accounts_for_filter_indicators() {
        use crate::backtesting::refs::{IndicatorRefExt, sma};

        let strategy = StrategyBuilder::new("Warmup Test")
            .regime_filter(sma(400).above_ref(sma(200)))
            .entry(always_true())
            .exit(always_false())
            .build();

        // Warmup must be at least sma(400).warmup_bars() + 1 = 401
        assert!(
            strategy.warmup_period() >= 401,
            "warmup_period must account for sma(400): got {}",
            strategy.warmup_period()
        );
    }
}
