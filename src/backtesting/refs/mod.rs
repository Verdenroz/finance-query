//! Indicator reference system for building strategy conditions.
//!
//! This module provides a type-safe way to reference indicator values
//! that can be used to build trading conditions.
//!
//! # Example
//!
//! ```ignore
//! use finance_query::backtesting::refs::*;
//!
//! // Reference RSI indicator
//! let rsi_ref = rsi(14);
//!
//! // Build conditions
//! let oversold = rsi_ref.below(30.0);
//! let overbought = rsi_ref.above(70.0);
//!
//! // Compose conditions
//! let entry = rsi(14).crosses_below(30.0).and(price().above_ref(sma(200)));
//! ```

mod computed;
mod price;

pub use computed::*;
pub use price::*;

use crate::indicators::Indicator;

use super::strategy::StrategyContext;

/// A reference to a value that can be compared in conditions.
///
/// This is the building block for creating conditions. Each indicator
/// reference knows:
/// - Its unique key for storing computed values
/// - What indicators it requires
/// - How to retrieve its value from the strategy context
///
/// # Implementing Custom References
///
/// ```ignore
/// use finance_query::backtesting::refs::IndicatorRef;
///
/// #[derive(Clone)]
/// struct MyCustomRef {
///     period: usize,
/// }
///
/// impl IndicatorRef for MyCustomRef {
///     fn key(&self) -> String {
///         format!("my_custom_{}", self.period)
///     }
///
///     fn required_indicators(&self) -> Vec<(String, Indicator)> {
///         vec![(self.key(), Indicator::Sma(self.period))]
///     }
///
///     fn value(&self, ctx: &StrategyContext) -> Option<f64> {
///         ctx.indicator(&self.key())
///     }
///
///     fn prev_value(&self, ctx: &StrategyContext) -> Option<f64> {
///         ctx.indicator_prev(&self.key())
///     }
/// }
/// ```
pub trait IndicatorRef: Clone + Send + Sync + 'static {
    /// Unique key for storing computed values in the context.
    ///
    /// This key is used to look up pre-computed indicator values
    /// in the `StrategyContext::indicators` map.
    fn key(&self) -> String;

    /// Required indicators to compute this reference.
    ///
    /// Returns a list of (key, Indicator) pairs that must be
    /// pre-computed by the backtest engine before the strategy runs.
    fn required_indicators(&self) -> Vec<(String, Indicator)>;

    /// Get the value at current candle index from context.
    fn value(&self, ctx: &StrategyContext) -> Option<f64>;

    /// Get the value at the previous candle index.
    fn prev_value(&self, ctx: &StrategyContext) -> Option<f64>;
}

/// Extension trait that adds condition-building methods to all indicator references.
///
/// This trait provides a fluent API for building conditions from indicator values.
/// It is automatically implemented for all types that implement `IndicatorRef`.
///
/// # Example
///
/// ```ignore
/// use finance_query::backtesting::refs::*;
///
/// // All these methods are available on any IndicatorRef
/// let cond1 = rsi(14).above(70.0);
/// let cond2 = rsi(14).below(30.0);
/// let cond3 = rsi(14).crosses_above(30.0);
/// let cond4 = rsi(14).crosses_below(70.0);
/// let cond5 = rsi(14).between(30.0, 70.0);
/// let cond6 = sma(10).above_ref(sma(20));
/// let cond7 = sma(10).crosses_above_ref(sma(20));
/// ```
pub trait IndicatorRefExt: IndicatorRef + Sized {
    /// Create a condition that checks if this indicator is above a threshold.
    ///
    /// # Example
    ///
    /// ```ignore
    /// let overbought = rsi(14).above(70.0);
    /// ```
    fn above(self, threshold: f64) -> super::condition::Above<Self> {
        super::condition::Above::new(self, threshold)
    }

    /// Create a condition that checks if this indicator is above another indicator.
    ///
    /// # Example
    ///
    /// ```ignore
    /// let uptrend = price().above_ref(sma(200));
    /// ```
    fn above_ref<R: IndicatorRef>(self, other: R) -> super::condition::AboveRef<Self, R> {
        super::condition::AboveRef::new(self, other)
    }

    /// Create a condition that checks if this indicator is below a threshold.
    ///
    /// # Example
    ///
    /// ```ignore
    /// let oversold = rsi(14).below(30.0);
    /// ```
    fn below(self, threshold: f64) -> super::condition::Below<Self> {
        super::condition::Below::new(self, threshold)
    }

    /// Create a condition that checks if this indicator is below another indicator.
    ///
    /// # Example
    ///
    /// ```ignore
    /// let downtrend = price().below_ref(sma(200));
    /// ```
    fn below_ref<R: IndicatorRef>(self, other: R) -> super::condition::BelowRef<Self, R> {
        super::condition::BelowRef::new(self, other)
    }

    /// Create a condition that checks if this indicator crosses above a threshold.
    ///
    /// A crossover occurs when the previous value was at or below the threshold
    /// and the current value is above it.
    ///
    /// # Example
    ///
    /// ```ignore
    /// let rsi_exit_oversold = rsi(14).crosses_above(30.0);
    /// ```
    fn crosses_above(self, threshold: f64) -> super::condition::CrossesAbove<Self> {
        super::condition::CrossesAbove::new(self, threshold)
    }

    /// Create a condition that checks if this indicator crosses above another indicator.
    ///
    /// # Example
    ///
    /// ```ignore
    /// let golden_cross = sma(50).crosses_above_ref(sma(200));
    /// ```
    fn crosses_above_ref<R: IndicatorRef>(
        self,
        other: R,
    ) -> super::condition::CrossesAboveRef<Self, R> {
        super::condition::CrossesAboveRef::new(self, other)
    }

    /// Create a condition that checks if this indicator crosses below a threshold.
    ///
    /// A crossover occurs when the previous value was at or above the threshold
    /// and the current value is below it.
    ///
    /// # Example
    ///
    /// ```ignore
    /// let rsi_enter_overbought = rsi(14).crosses_below(70.0);
    /// ```
    fn crosses_below(self, threshold: f64) -> super::condition::CrossesBelow<Self> {
        super::condition::CrossesBelow::new(self, threshold)
    }

    /// Create a condition that checks if this indicator crosses below another indicator.
    ///
    /// # Example
    ///
    /// ```ignore
    /// let death_cross = sma(50).crosses_below_ref(sma(200));
    /// ```
    fn crosses_below_ref<R: IndicatorRef>(
        self,
        other: R,
    ) -> super::condition::CrossesBelowRef<Self, R> {
        super::condition::CrossesBelowRef::new(self, other)
    }

    /// Create a condition that checks if this indicator is between two thresholds.
    ///
    /// Returns true when `low < value < high`.
    ///
    /// # Example
    ///
    /// ```ignore
    /// let neutral_rsi = rsi(14).between(40.0, 60.0);
    /// ```
    fn between(self, low: f64, high: f64) -> super::condition::Between<Self> {
        super::condition::Between::new(self, low, high)
    }

    /// Create a condition that checks if this indicator equals a value (within tolerance).
    ///
    /// # Example
    ///
    /// ```ignore
    /// let at_zero = macd(12, 26, 9).histogram().equals(0.0, 0.001);
    /// ```
    fn equals(self, value: f64, tolerance: f64) -> super::condition::Equals<Self> {
        super::condition::Equals::new(self, value, tolerance)
    }
}

// Auto-implement IndicatorRefExt for all types that implement IndicatorRef
impl<T: IndicatorRef + Sized> IndicatorRefExt for T {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_indicator_ref_ext_methods_exist() {
        // This test just verifies the trait methods compile
        let _sma = sma(20);
        let _ema = ema(12);
        let _rsi = rsi(14);

        // Verify key() works
        assert_eq!(_sma.key(), "sma_20");
        assert_eq!(_ema.key(), "ema_12");
        assert_eq!(_rsi.key(), "rsi_14");
    }
}
