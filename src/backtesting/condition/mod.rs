//! Condition system for building strategy entry/exit rules.
//!
//! This module provides a composable way to define trading conditions
//! using indicator references and comparison operations.
//!
//! # Example
//!
//! ```ignore
//! use finance_query::backtesting::refs::*;
//! use finance_query::backtesting::condition::*;
//!
//! // Simple condition
//! let oversold = rsi(14).below(30.0);
//!
//! // Compound conditions
//! let entry = rsi(14).crosses_below(30.0)
//!     .and(price().above_ref(sma(200)));
//!
//! let exit = rsi(14).crosses_above(70.0)
//!     .or(stop_loss(0.05));
//! ```

mod comparison;
mod composite;
mod threshold;

pub use comparison::*;
pub use composite::*;
pub use threshold::*;

use crate::indicators::Indicator;

use super::strategy::StrategyContext;

/// A condition that can be evaluated on each candle.
///
/// Conditions are the building blocks of trading strategies.
/// They can be combined using `and()`, `or()`, and `not()` operations.
///
/// # Example
///
/// ```ignore
/// use finance_query::backtesting::condition::Condition;
///
/// fn my_custom_condition(ctx: &StrategyContext) -> bool {
///     // Custom logic here
///     true
/// }
/// ```
pub trait Condition: Clone + Send + Sync + 'static {
    /// Evaluate the condition with the current strategy context.
    ///
    /// Returns `true` if the condition is met, `false` otherwise.
    fn evaluate(&self, ctx: &StrategyContext) -> bool;

    /// Get the indicators required by this condition.
    ///
    /// The backtest engine will pre-compute these indicators
    /// before running the strategy.
    fn required_indicators(&self) -> Vec<(String, Indicator)>;

    /// Get a human-readable description of this condition.
    ///
    /// This is used for logging, debugging, and signal reporting.
    fn description(&self) -> String;

    /// Combine this condition with another using AND logic.
    ///
    /// The resulting condition is true only when both conditions are true.
    ///
    /// # Example
    ///
    /// ```ignore
    /// let entry = rsi(14).below(30.0).and(price().above_ref(sma(200)));
    /// ```
    fn and<C: Condition>(self, other: C) -> And<Self, C>
    where
        Self: Sized,
    {
        And::new(self, other)
    }

    /// Combine this condition with another using OR logic.
    ///
    /// The resulting condition is true when either condition is true.
    ///
    /// # Example
    ///
    /// ```ignore
    /// let exit = rsi(14).above(70.0).or(stop_loss(0.05));
    /// ```
    fn or<C: Condition>(self, other: C) -> Or<Self, C>
    where
        Self: Sized,
    {
        Or::new(self, other)
    }

    /// Negate this condition.
    ///
    /// The resulting condition is true when this condition is false.
    ///
    /// # Example
    ///
    /// ```ignore
    /// let not_overbought = rsi(14).above(70.0).not();
    /// ```
    fn not(self) -> Not<Self>
    where
        Self: Sized,
    {
        Not::new(self)
    }
}

/// A condition that always returns the same value.
///
/// Useful for testing or as a placeholder.
#[derive(Debug, Clone, Copy)]
pub struct ConstantCondition(bool);

impl ConstantCondition {
    /// Create a condition that always returns true.
    pub fn always_true() -> Self {
        Self(true)
    }

    /// Create a condition that always returns false.
    pub fn always_false() -> Self {
        Self(false)
    }
}

impl Condition for ConstantCondition {
    fn evaluate(&self, _ctx: &StrategyContext) -> bool {
        self.0
    }

    fn required_indicators(&self) -> Vec<(String, Indicator)> {
        vec![]
    }

    fn description(&self) -> String {
        if self.0 {
            "always true".to_string()
        } else {
            "always false".to_string()
        }
    }
}

/// Convenience function to create a condition that always returns true.
#[inline]
pub fn always_true() -> ConstantCondition {
    ConstantCondition::always_true()
}

/// Convenience function to create a condition that always returns false.
#[inline]
pub fn always_false() -> ConstantCondition {
    ConstantCondition::always_false()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_constant_conditions() {
        assert_eq!(always_true().description(), "always true");
        assert_eq!(always_false().description(), "always false");
    }
}
