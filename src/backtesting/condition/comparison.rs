//! Comparison conditions for indicator references.
//!
//! This module provides conditions that compare indicator values
//! against thresholds or other indicators.

use crate::backtesting::refs::IndicatorRef;
use crate::backtesting::strategy::StrategyContext;
use crate::indicators::Indicator;

use super::Condition;

// ============================================================================
// THRESHOLD COMPARISONS
// ============================================================================

/// Condition: indicator is above a threshold.
#[derive(Clone)]
pub struct Above<R: IndicatorRef> {
    indicator: R,
    threshold: f64,
}

impl<R: IndicatorRef> Above<R> {
    /// Create a new Above condition.
    pub fn new(indicator: R, threshold: f64) -> Self {
        Self {
            indicator,
            threshold,
        }
    }
}

impl<R: IndicatorRef> Condition for Above<R> {
    fn evaluate(&self, ctx: &StrategyContext) -> bool {
        self.indicator
            .value(ctx)
            .map(|v| v > self.threshold)
            .unwrap_or(false)
    }

    fn required_indicators(&self) -> Vec<(String, Indicator)> {
        self.indicator.required_indicators()
    }

    fn description(&self) -> String {
        format!("{} > {:.2}", self.indicator.key(), self.threshold)
    }
}

/// Condition: indicator is below a threshold.
#[derive(Clone)]
pub struct Below<R: IndicatorRef> {
    indicator: R,
    threshold: f64,
}

impl<R: IndicatorRef> Below<R> {
    /// Create a new Below condition.
    pub fn new(indicator: R, threshold: f64) -> Self {
        Self {
            indicator,
            threshold,
        }
    }
}

impl<R: IndicatorRef> Condition for Below<R> {
    fn evaluate(&self, ctx: &StrategyContext) -> bool {
        self.indicator
            .value(ctx)
            .map(|v| v < self.threshold)
            .unwrap_or(false)
    }

    fn required_indicators(&self) -> Vec<(String, Indicator)> {
        self.indicator.required_indicators()
    }

    fn description(&self) -> String {
        format!("{} < {:.2}", self.indicator.key(), self.threshold)
    }
}

/// Condition: indicator crosses above a threshold.
///
/// True when the previous value was at or below the threshold
/// and the current value is above it.
#[derive(Clone)]
pub struct CrossesAbove<R: IndicatorRef> {
    indicator: R,
    threshold: f64,
}

impl<R: IndicatorRef> CrossesAbove<R> {
    /// Create a new CrossesAbove condition.
    pub fn new(indicator: R, threshold: f64) -> Self {
        Self {
            indicator,
            threshold,
        }
    }
}

impl<R: IndicatorRef> Condition for CrossesAbove<R> {
    fn evaluate(&self, ctx: &StrategyContext) -> bool {
        let current = self.indicator.value(ctx);
        let prev = self.indicator.prev_value(ctx);

        match (current, prev) {
            (Some(curr), Some(p)) => p <= self.threshold && curr > self.threshold,
            _ => false,
        }
    }

    fn required_indicators(&self) -> Vec<(String, Indicator)> {
        self.indicator.required_indicators()
    }

    fn description(&self) -> String {
        format!(
            "{} crosses above {:.2}",
            self.indicator.key(),
            self.threshold
        )
    }
}

/// Condition: indicator crosses below a threshold.
///
/// True when the previous value was at or above the threshold
/// and the current value is below it.
#[derive(Clone)]
pub struct CrossesBelow<R: IndicatorRef> {
    indicator: R,
    threshold: f64,
}

impl<R: IndicatorRef> CrossesBelow<R> {
    /// Create a new CrossesBelow condition.
    pub fn new(indicator: R, threshold: f64) -> Self {
        Self {
            indicator,
            threshold,
        }
    }
}

impl<R: IndicatorRef> Condition for CrossesBelow<R> {
    fn evaluate(&self, ctx: &StrategyContext) -> bool {
        let current = self.indicator.value(ctx);
        let prev = self.indicator.prev_value(ctx);

        match (current, prev) {
            (Some(curr), Some(p)) => p >= self.threshold && curr < self.threshold,
            _ => false,
        }
    }

    fn required_indicators(&self) -> Vec<(String, Indicator)> {
        self.indicator.required_indicators()
    }

    fn description(&self) -> String {
        format!(
            "{} crosses below {:.2}",
            self.indicator.key(),
            self.threshold
        )
    }
}

/// Condition: indicator is between two thresholds.
///
/// True when `low < value < high`.
#[derive(Clone)]
pub struct Between<R: IndicatorRef> {
    indicator: R,
    low: f64,
    high: f64,
}

impl<R: IndicatorRef> Between<R> {
    /// Create a new Between condition.
    pub fn new(indicator: R, low: f64, high: f64) -> Self {
        Self {
            indicator,
            low,
            high,
        }
    }
}

impl<R: IndicatorRef> Condition for Between<R> {
    fn evaluate(&self, ctx: &StrategyContext) -> bool {
        self.indicator
            .value(ctx)
            .map(|v| v > self.low && v < self.high)
            .unwrap_or(false)
    }

    fn required_indicators(&self) -> Vec<(String, Indicator)> {
        self.indicator.required_indicators()
    }

    fn description(&self) -> String {
        format!(
            "{:.2} < {} < {:.2}",
            self.low,
            self.indicator.key(),
            self.high
        )
    }
}

/// Condition: indicator equals a value (within tolerance).
#[derive(Clone)]
pub struct Equals<R: IndicatorRef> {
    indicator: R,
    value: f64,
    tolerance: f64,
}

impl<R: IndicatorRef> Equals<R> {
    /// Create a new Equals condition.
    pub fn new(indicator: R, value: f64, tolerance: f64) -> Self {
        Self {
            indicator,
            value,
            tolerance,
        }
    }
}

impl<R: IndicatorRef> Condition for Equals<R> {
    fn evaluate(&self, ctx: &StrategyContext) -> bool {
        self.indicator
            .value(ctx)
            .map(|v| (v - self.value).abs() <= self.tolerance)
            .unwrap_or(false)
    }

    fn required_indicators(&self) -> Vec<(String, Indicator)> {
        self.indicator.required_indicators()
    }

    fn description(&self) -> String {
        format!(
            "{} ≈ {:.2} (±{:.4})",
            self.indicator.key(),
            self.value,
            self.tolerance
        )
    }
}

// ============================================================================
// INDICATOR VS INDICATOR COMPARISONS
// ============================================================================

/// Condition: indicator is above another indicator.
#[derive(Clone)]
pub struct AboveRef<R1: IndicatorRef, R2: IndicatorRef> {
    indicator: R1,
    other: R2,
}

impl<R1: IndicatorRef, R2: IndicatorRef> AboveRef<R1, R2> {
    /// Create a new AboveRef condition.
    pub fn new(indicator: R1, other: R2) -> Self {
        Self { indicator, other }
    }
}

impl<R1: IndicatorRef, R2: IndicatorRef> Condition for AboveRef<R1, R2> {
    fn evaluate(&self, ctx: &StrategyContext) -> bool {
        let v1 = self.indicator.value(ctx);
        let v2 = self.other.value(ctx);

        match (v1, v2) {
            (Some(a), Some(b)) => a > b,
            _ => false,
        }
    }

    fn required_indicators(&self) -> Vec<(String, Indicator)> {
        let mut indicators = self.indicator.required_indicators();
        indicators.extend(self.other.required_indicators());
        indicators
    }

    fn description(&self) -> String {
        format!("{} > {}", self.indicator.key(), self.other.key())
    }
}

/// Condition: indicator is below another indicator.
#[derive(Clone)]
pub struct BelowRef<R1: IndicatorRef, R2: IndicatorRef> {
    indicator: R1,
    other: R2,
}

impl<R1: IndicatorRef, R2: IndicatorRef> BelowRef<R1, R2> {
    /// Create a new BelowRef condition.
    pub fn new(indicator: R1, other: R2) -> Self {
        Self { indicator, other }
    }
}

impl<R1: IndicatorRef, R2: IndicatorRef> Condition for BelowRef<R1, R2> {
    fn evaluate(&self, ctx: &StrategyContext) -> bool {
        let v1 = self.indicator.value(ctx);
        let v2 = self.other.value(ctx);

        match (v1, v2) {
            (Some(a), Some(b)) => a < b,
            _ => false,
        }
    }

    fn required_indicators(&self) -> Vec<(String, Indicator)> {
        let mut indicators = self.indicator.required_indicators();
        indicators.extend(self.other.required_indicators());
        indicators
    }

    fn description(&self) -> String {
        format!("{} < {}", self.indicator.key(), self.other.key())
    }
}

/// Condition: indicator crosses above another indicator.
///
/// True when the fast indicator was at or below the slow indicator
/// and is now above it.
#[derive(Clone)]
pub struct CrossesAboveRef<R1: IndicatorRef, R2: IndicatorRef> {
    fast: R1,
    slow: R2,
}

impl<R1: IndicatorRef, R2: IndicatorRef> CrossesAboveRef<R1, R2> {
    /// Create a new CrossesAboveRef condition.
    pub fn new(fast: R1, slow: R2) -> Self {
        Self { fast, slow }
    }
}

impl<R1: IndicatorRef, R2: IndicatorRef> Condition for CrossesAboveRef<R1, R2> {
    fn evaluate(&self, ctx: &StrategyContext) -> bool {
        let fast_now = self.fast.value(ctx);
        let slow_now = self.slow.value(ctx);
        let fast_prev = self.fast.prev_value(ctx);
        let slow_prev = self.slow.prev_value(ctx);

        match (fast_now, slow_now, fast_prev, slow_prev) {
            (Some(fn_), Some(sn), Some(fp), Some(sp)) => fp <= sp && fn_ > sn,
            _ => false,
        }
    }

    fn required_indicators(&self) -> Vec<(String, Indicator)> {
        let mut indicators = self.fast.required_indicators();
        indicators.extend(self.slow.required_indicators());
        indicators
    }

    fn description(&self) -> String {
        format!("{} crosses above {}", self.fast.key(), self.slow.key())
    }
}

/// Condition: indicator crosses below another indicator.
///
/// True when the fast indicator was at or above the slow indicator
/// and is now below it.
#[derive(Clone)]
pub struct CrossesBelowRef<R1: IndicatorRef, R2: IndicatorRef> {
    fast: R1,
    slow: R2,
}

impl<R1: IndicatorRef, R2: IndicatorRef> CrossesBelowRef<R1, R2> {
    /// Create a new CrossesBelowRef condition.
    pub fn new(fast: R1, slow: R2) -> Self {
        Self { fast, slow }
    }
}

impl<R1: IndicatorRef, R2: IndicatorRef> Condition for CrossesBelowRef<R1, R2> {
    fn evaluate(&self, ctx: &StrategyContext) -> bool {
        let fast_now = self.fast.value(ctx);
        let slow_now = self.slow.value(ctx);
        let fast_prev = self.fast.prev_value(ctx);
        let slow_prev = self.slow.prev_value(ctx);

        match (fast_now, slow_now, fast_prev, slow_prev) {
            (Some(fn_), Some(sn), Some(fp), Some(sp)) => fp >= sp && fn_ < sn,
            _ => false,
        }
    }

    fn required_indicators(&self) -> Vec<(String, Indicator)> {
        let mut indicators = self.fast.required_indicators();
        indicators.extend(self.slow.required_indicators());
        indicators
    }

    fn description(&self) -> String {
        format!("{} crosses below {}", self.fast.key(), self.slow.key())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::backtesting::refs::{rsi, sma};

    #[test]
    fn test_above_description() {
        let cond = Above::new(rsi(14), 70.0);
        assert_eq!(cond.description(), "rsi_14 > 70.00");
    }

    #[test]
    fn test_below_description() {
        let cond = Below::new(rsi(14), 30.0);
        assert_eq!(cond.description(), "rsi_14 < 30.00");
    }

    #[test]
    fn test_crosses_above_description() {
        let cond = CrossesAbove::new(rsi(14), 30.0);
        assert_eq!(cond.description(), "rsi_14 crosses above 30.00");
    }

    #[test]
    fn test_crosses_below_description() {
        let cond = CrossesBelow::new(rsi(14), 70.0);
        assert_eq!(cond.description(), "rsi_14 crosses below 70.00");
    }

    #[test]
    fn test_between_description() {
        let cond = Between::new(rsi(14), 30.0, 70.0);
        assert_eq!(cond.description(), "30.00 < rsi_14 < 70.00");
    }

    #[test]
    fn test_above_ref_description() {
        let cond = AboveRef::new(sma(10), sma(20));
        assert_eq!(cond.description(), "sma_10 > sma_20");
    }

    #[test]
    fn test_crosses_above_ref_description() {
        let cond = CrossesAboveRef::new(sma(50), sma(200));
        assert_eq!(cond.description(), "sma_50 crosses above sma_200");
    }

    #[test]
    fn test_required_indicators() {
        let cond = Above::new(rsi(14), 70.0);
        let indicators = cond.required_indicators();
        assert_eq!(indicators.len(), 1);
        assert_eq!(indicators[0].0, "rsi_14");
    }

    #[test]
    fn test_cross_ref_required_indicators() {
        let cond = CrossesAboveRef::new(sma(10), sma(20));
        let indicators = cond.required_indicators();
        assert_eq!(indicators.len(), 2);
    }
}
