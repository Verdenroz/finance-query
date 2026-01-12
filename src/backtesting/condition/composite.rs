//! Composite conditions for combining multiple conditions.
//!
//! This module provides AND, OR, and NOT operations for combining conditions.

use crate::backtesting::strategy::StrategyContext;
use crate::indicators::Indicator;

use super::Condition;

/// Condition: both conditions must be true (AND logic).
#[derive(Clone)]
pub struct And<C1: Condition, C2: Condition> {
    left: C1,
    right: C2,
}

impl<C1: Condition, C2: Condition> And<C1, C2> {
    /// Create a new And condition.
    pub fn new(left: C1, right: C2) -> Self {
        Self { left, right }
    }
}

impl<C1: Condition, C2: Condition> Condition for And<C1, C2> {
    fn evaluate(&self, ctx: &StrategyContext) -> bool {
        self.left.evaluate(ctx) && self.right.evaluate(ctx)
    }

    fn required_indicators(&self) -> Vec<(String, Indicator)> {
        let mut indicators = self.left.required_indicators();
        indicators.extend(self.right.required_indicators());
        // Deduplicate by key
        indicators.sort_by(|a, b| a.0.cmp(&b.0));
        indicators.dedup_by(|a, b| a.0 == b.0);
        indicators
    }

    fn description(&self) -> String {
        format!(
            "({} AND {})",
            self.left.description(),
            self.right.description()
        )
    }
}

/// Condition: at least one condition must be true (OR logic).
#[derive(Clone)]
pub struct Or<C1: Condition, C2: Condition> {
    left: C1,
    right: C2,
}

impl<C1: Condition, C2: Condition> Or<C1, C2> {
    /// Create a new Or condition.
    pub fn new(left: C1, right: C2) -> Self {
        Self { left, right }
    }
}

impl<C1: Condition, C2: Condition> Condition for Or<C1, C2> {
    fn evaluate(&self, ctx: &StrategyContext) -> bool {
        self.left.evaluate(ctx) || self.right.evaluate(ctx)
    }

    fn required_indicators(&self) -> Vec<(String, Indicator)> {
        let mut indicators = self.left.required_indicators();
        indicators.extend(self.right.required_indicators());
        // Deduplicate by key
        indicators.sort_by(|a, b| a.0.cmp(&b.0));
        indicators.dedup_by(|a, b| a.0 == b.0);
        indicators
    }

    fn description(&self) -> String {
        format!(
            "({} OR {})",
            self.left.description(),
            self.right.description()
        )
    }
}

/// Condition: negation of a condition (NOT logic).
#[derive(Clone)]
pub struct Not<C: Condition> {
    inner: C,
}

impl<C: Condition> Not<C> {
    /// Create a new Not condition.
    pub fn new(inner: C) -> Self {
        Self { inner }
    }
}

impl<C: Condition> Condition for Not<C> {
    fn evaluate(&self, ctx: &StrategyContext) -> bool {
        !self.inner.evaluate(ctx)
    }

    fn required_indicators(&self) -> Vec<(String, Indicator)> {
        self.inner.required_indicators()
    }

    fn description(&self) -> String {
        format!("NOT ({})", self.inner.description())
    }
}

/// Builder for creating complex multi-condition combinations.
///
/// # Example
///
/// ```ignore
/// use finance_query::backtesting::condition::*;
/// use finance_query::backtesting::refs::*;
///
/// let conditions = ConditionBuilder::new()
///     .with_condition(rsi(14).below(30.0))
///     .with_condition(price().above_ref(sma(200)))
///     .with_condition(adx(14).above(25.0))
///     .all();  // All conditions must be true
///
/// // Or use any() for OR logic
/// let exit = ConditionBuilder::new()
///     .with_condition(rsi(14).above(70.0))
///     .with_condition(stop_loss(0.05))
///     .any();  // Any condition can be true
/// ```
#[derive(Clone)]
pub struct ConditionBuilder<C: Condition> {
    conditions: Vec<C>,
}

impl<C: Condition> Default for ConditionBuilder<C> {
    fn default() -> Self {
        Self::new()
    }
}

impl<C: Condition> ConditionBuilder<C> {
    /// Create a new condition builder.
    pub fn new() -> Self {
        Self {
            conditions: Vec::new(),
        }
    }

    /// Add a condition to the builder.
    pub fn with_condition(mut self, condition: C) -> Self {
        self.conditions.push(condition);
        self
    }
}

/// A condition that evaluates to true when ALL inner conditions are true.
#[derive(Clone)]
pub struct All<C: Condition> {
    conditions: Vec<C>,
}

impl<C: Condition> Condition for All<C> {
    fn evaluate(&self, ctx: &StrategyContext) -> bool {
        self.conditions.iter().all(|c| c.evaluate(ctx))
    }

    fn required_indicators(&self) -> Vec<(String, Indicator)> {
        let mut indicators = Vec::new();
        for c in &self.conditions {
            indicators.extend(c.required_indicators());
        }
        // Deduplicate by key
        indicators.sort_by(|a, b| a.0.cmp(&b.0));
        indicators.dedup_by(|a, b| a.0 == b.0);
        indicators
    }

    fn description(&self) -> String {
        let descs: Vec<_> = self.conditions.iter().map(|c| c.description()).collect();
        format!("ALL({})", descs.join(" AND "))
    }
}

/// A condition that evaluates to true when ANY inner condition is true.
#[derive(Clone)]
pub struct Any<C: Condition> {
    conditions: Vec<C>,
}

impl<C: Condition> Condition for Any<C> {
    fn evaluate(&self, ctx: &StrategyContext) -> bool {
        self.conditions.iter().any(|c| c.evaluate(ctx))
    }

    fn required_indicators(&self) -> Vec<(String, Indicator)> {
        let mut indicators = Vec::new();
        for c in &self.conditions {
            indicators.extend(c.required_indicators());
        }
        // Deduplicate by key
        indicators.sort_by(|a, b| a.0.cmp(&b.0));
        indicators.dedup_by(|a, b| a.0 == b.0);
        indicators
    }

    fn description(&self) -> String {
        let descs: Vec<_> = self.conditions.iter().map(|c| c.description()).collect();
        format!("ANY({})", descs.join(" OR "))
    }
}

impl<C: Condition> ConditionBuilder<C> {
    /// Build a condition that requires ALL conditions to be true.
    pub fn all(self) -> All<C> {
        All {
            conditions: self.conditions,
        }
    }

    /// Build a condition that requires ANY condition to be true.
    pub fn any(self) -> Any<C> {
        Any {
            conditions: self.conditions,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::backtesting::condition::{always_false, always_true};

    #[test]
    fn test_and_description() {
        let cond = And::new(always_true(), always_false());
        assert_eq!(cond.description(), "(always true AND always false)");
    }

    #[test]
    fn test_or_description() {
        let cond = Or::new(always_true(), always_false());
        assert_eq!(cond.description(), "(always true OR always false)");
    }

    #[test]
    fn test_not_description() {
        let cond = Not::new(always_true());
        assert_eq!(cond.description(), "NOT (always true)");
    }

    #[test]
    fn test_all_description() {
        let all = ConditionBuilder::new()
            .with_condition(always_true())
            .with_condition(always_false())
            .all();
        assert_eq!(all.description(), "ALL(always true AND always false)");
    }

    #[test]
    fn test_any_description() {
        let any = ConditionBuilder::new()
            .with_condition(always_true())
            .with_condition(always_false())
            .any();
        assert_eq!(any.description(), "ANY(always true OR always false)");
    }
}
