use crate::backtesting::strategy::StrategyContext;
use crate::indicators::Indicator;

use super::IndicatorRef;

/// Average Directional Index reference.
#[derive(Debug, Clone)]
pub struct AdxRef {
    pub period: usize,
    key: String,
}

impl IndicatorRef for AdxRef {
    fn key(&self) -> &str {
        &self.key
    }

    fn required_indicators(&self) -> Vec<(String, Indicator)> {
        vec![(self.key.clone(), Indicator::Adx(self.period))]
    }

    fn value(&self, ctx: &StrategyContext) -> Option<f64> {
        ctx.indicator(self.key())
    }

    fn prev_value(&self, ctx: &StrategyContext) -> Option<f64> {
        ctx.indicator_prev(self.key())
    }
}

/// Create an Average Directional Index reference.
///
/// # Example
///
/// ```ignore
/// use finance_query::backtesting::refs::*;
///
/// // Strong trend filter
/// let strong_trend = adx(14).above(25.0);
/// ```
#[inline]
pub fn adx(period: usize) -> AdxRef {
    AdxRef {
        period,
        key: format!("adx_{period}"),
    }
}

/// MACD configuration for building MACD-related references.
#[derive(Debug, Clone, Copy)]
pub struct MacdConfig {
    /// Fast EMA period
    pub fast: usize,
    /// Slow EMA period
    pub slow: usize,
    /// Signal line period
    pub signal: usize,
}

impl MacdConfig {
    /// Get the MACD line reference.
    pub fn line(&self) -> MacdLineRef {
        MacdLineRef::new(self.fast, self.slow, self.signal)
    }

    /// Get the MACD signal line reference.
    pub fn signal_line(&self) -> MacdSignalRef {
        MacdSignalRef::new(self.fast, self.slow, self.signal)
    }

    /// Get the MACD histogram reference.
    pub fn histogram(&self) -> MacdHistogramRef {
        MacdHistogramRef::new(self.fast, self.slow, self.signal)
    }
}

/// Create a MACD configuration.
///
/// # Example
///
/// ```ignore
/// use finance_query::backtesting::refs::*;
///
/// let m = macd(12, 26, 9);
/// let bullish = m.line().crosses_above_ref(m.signal_line());
/// let histogram_positive = m.histogram().above(0.0);
/// ```
#[inline]
pub fn macd(fast: usize, slow: usize, signal: usize) -> MacdConfig {
    MacdConfig { fast, slow, signal }
}

/// MACD Line reference.
#[derive(Debug, Clone)]
pub struct MacdLineRef {
    /// Fast EMA period.
    pub fast: usize,
    /// Slow EMA period.
    pub slow: usize,
    /// Signal line period.
    pub signal: usize,
    key: String,
}

impl MacdLineRef {
    fn new(fast: usize, slow: usize, signal: usize) -> Self {
        Self {
            fast,
            slow,
            signal,
            key: format!("macd_line_{fast}_{slow}_{signal}"),
        }
    }
}

impl IndicatorRef for MacdLineRef {
    fn key(&self) -> &str {
        &self.key
    }

    fn required_indicators(&self) -> Vec<(String, Indicator)> {
        vec![(
            self.key.clone(),
            Indicator::Macd {
                fast: self.fast,
                slow: self.slow,
                signal: self.signal,
            },
        )]
    }

    fn value(&self, ctx: &StrategyContext) -> Option<f64> {
        ctx.indicator(self.key())
    }

    fn prev_value(&self, ctx: &StrategyContext) -> Option<f64> {
        ctx.indicator_prev(self.key())
    }
}

/// MACD Signal Line reference.
#[derive(Debug, Clone)]
pub struct MacdSignalRef {
    /// Fast EMA period.
    pub fast: usize,
    /// Slow EMA period.
    pub slow: usize,
    /// Signal line period.
    pub signal: usize,
    key: String,
}

impl MacdSignalRef {
    fn new(fast: usize, slow: usize, signal: usize) -> Self {
        Self {
            fast,
            slow,
            signal,
            key: format!("macd_signal_{fast}_{slow}_{signal}"),
        }
    }
}

impl IndicatorRef for MacdSignalRef {
    fn key(&self) -> &str {
        &self.key
    }

    fn required_indicators(&self) -> Vec<(String, Indicator)> {
        vec![(
            self.key.clone(),
            Indicator::Macd {
                fast: self.fast,
                slow: self.slow,
                signal: self.signal,
            },
        )]
    }

    fn value(&self, ctx: &StrategyContext) -> Option<f64> {
        ctx.indicator(self.key())
    }

    fn prev_value(&self, ctx: &StrategyContext) -> Option<f64> {
        ctx.indicator_prev(self.key())
    }
}

/// MACD Histogram reference.
#[derive(Debug, Clone)]
pub struct MacdHistogramRef {
    /// Fast EMA period.
    pub fast: usize,
    /// Slow EMA period.
    pub slow: usize,
    /// Signal line period.
    pub signal: usize,
    key: String,
}

impl MacdHistogramRef {
    fn new(fast: usize, slow: usize, signal: usize) -> Self {
        Self {
            fast,
            slow,
            signal,
            key: format!("macd_histogram_{fast}_{slow}_{signal}"),
        }
    }
}

impl IndicatorRef for MacdHistogramRef {
    fn key(&self) -> &str {
        &self.key
    }

    fn required_indicators(&self) -> Vec<(String, Indicator)> {
        vec![(
            self.key.clone(),
            Indicator::Macd {
                fast: self.fast,
                slow: self.slow,
                signal: self.signal,
            },
        )]
    }

    fn value(&self, ctx: &StrategyContext) -> Option<f64> {
        ctx.indicator(self.key())
    }

    fn prev_value(&self, ctx: &StrategyContext) -> Option<f64> {
        ctx.indicator_prev(self.key())
    }
}

/// SuperTrend configuration.
#[derive(Debug, Clone, Copy)]
pub struct SupertrendConfig {
    pub period: usize,
    pub multiplier: f64,
}

impl SupertrendConfig {
    /// Get the SuperTrend value reference.
    pub fn value(&self) -> SupertrendValueRef {
        SupertrendValueRef::new(self.period, self.multiplier)
    }

    /// Get the SuperTrend uptrend indicator (1.0 = uptrend, 0.0 = downtrend).
    pub fn uptrend(&self) -> SupertrendUptrendRef {
        SupertrendUptrendRef::new(self.period, self.multiplier)
    }
}

/// Create a SuperTrend configuration.
#[inline]
pub fn supertrend(period: usize, multiplier: f64) -> SupertrendConfig {
    SupertrendConfig { period, multiplier }
}

/// SuperTrend value reference.
#[derive(Debug, Clone)]
pub struct SupertrendValueRef {
    pub period: usize,
    pub multiplier: f64,
    key: String,
}

impl SupertrendValueRef {
    fn new(period: usize, multiplier: f64) -> Self {
        Self {
            period,
            multiplier,
            key: format!("supertrend_value_{period}_{multiplier}"),
        }
    }
}

impl IndicatorRef for SupertrendValueRef {
    fn key(&self) -> &str {
        &self.key
    }

    fn required_indicators(&self) -> Vec<(String, Indicator)> {
        vec![(
            self.key.clone(),
            Indicator::Supertrend {
                period: self.period,
                multiplier: self.multiplier,
            },
        )]
    }

    fn value(&self, ctx: &StrategyContext) -> Option<f64> {
        ctx.indicator(self.key())
    }

    fn prev_value(&self, ctx: &StrategyContext) -> Option<f64> {
        ctx.indicator_prev(self.key())
    }
}

/// SuperTrend uptrend indicator reference.
/// Returns 1.0 for uptrend, 0.0 for downtrend.
#[derive(Debug, Clone)]
pub struct SupertrendUptrendRef {
    pub period: usize,
    pub multiplier: f64,
    key: String,
}

impl SupertrendUptrendRef {
    fn new(period: usize, multiplier: f64) -> Self {
        Self {
            period,
            multiplier,
            key: format!("supertrend_uptrend_{period}_{multiplier}"),
        }
    }
}

impl IndicatorRef for SupertrendUptrendRef {
    fn key(&self) -> &str {
        &self.key
    }

    fn required_indicators(&self) -> Vec<(String, Indicator)> {
        vec![(
            self.key.clone(),
            Indicator::Supertrend {
                period: self.period,
                multiplier: self.multiplier,
            },
        )]
    }

    fn value(&self, ctx: &StrategyContext) -> Option<f64> {
        ctx.indicator(self.key())
    }

    fn prev_value(&self, ctx: &StrategyContext) -> Option<f64> {
        ctx.indicator_prev(self.key())
    }
}

/// Aroon indicator configuration.
#[derive(Debug, Clone, Copy)]
pub struct AroonConfig {
    pub period: usize,
}

impl AroonConfig {
    /// Get the Aroon Up reference.
    pub fn up(&self) -> AroonUpRef {
        AroonUpRef::new(self.period)
    }

    /// Get the Aroon Down reference.
    pub fn down(&self) -> AroonDownRef {
        AroonDownRef::new(self.period)
    }
}

/// Create an Aroon indicator configuration.
#[inline]
pub fn aroon(period: usize) -> AroonConfig {
    AroonConfig { period }
}

/// Aroon Up reference.
#[derive(Debug, Clone)]
pub struct AroonUpRef {
    pub period: usize,
    key: String,
}

impl AroonUpRef {
    fn new(period: usize) -> Self {
        Self {
            period,
            key: format!("aroon_up_{period}"),
        }
    }
}

impl IndicatorRef for AroonUpRef {
    fn key(&self) -> &str {
        &self.key
    }

    fn required_indicators(&self) -> Vec<(String, Indicator)> {
        vec![(self.key.clone(), Indicator::Aroon(self.period))]
    }

    fn value(&self, ctx: &StrategyContext) -> Option<f64> {
        ctx.indicator(self.key())
    }

    fn prev_value(&self, ctx: &StrategyContext) -> Option<f64> {
        ctx.indicator_prev(self.key())
    }
}

/// Aroon Down reference.
#[derive(Debug, Clone)]
pub struct AroonDownRef {
    pub period: usize,
    key: String,
}

impl AroonDownRef {
    fn new(period: usize) -> Self {
        Self {
            period,
            key: format!("aroon_down_{period}"),
        }
    }
}

impl IndicatorRef for AroonDownRef {
    fn key(&self) -> &str {
        &self.key
    }

    fn required_indicators(&self) -> Vec<(String, Indicator)> {
        vec![(self.key.clone(), Indicator::Aroon(self.period))]
    }

    fn value(&self, ctx: &StrategyContext) -> Option<f64> {
        ctx.indicator(self.key())
    }

    fn prev_value(&self, ctx: &StrategyContext) -> Option<f64> {
        ctx.indicator_prev(self.key())
    }
}

/// Parabolic SAR configuration.
#[derive(Debug, Clone, Copy)]
pub struct ParabolicSarConfig {
    pub step: f64,
    pub max: f64,
}

/// Create a Parabolic SAR configuration.
#[inline]
pub fn parabolic_sar(step: f64, max: f64) -> ParabolicSarRef {
    ParabolicSarRef::new(step, max)
}

/// Parabolic SAR reference.
#[derive(Debug, Clone)]
pub struct ParabolicSarRef {
    pub step: f64,
    pub max: f64,
    key: String,
}

impl ParabolicSarRef {
    fn new(step: f64, max: f64) -> Self {
        Self {
            step,
            max,
            key: format!("psar_{step}_{max}"),
        }
    }
}

impl IndicatorRef for ParabolicSarRef {
    fn key(&self) -> &str {
        &self.key
    }

    fn required_indicators(&self) -> Vec<(String, Indicator)> {
        vec![(
            self.key.clone(),
            Indicator::ParabolicSar {
                step: self.step,
                max: self.max,
            },
        )]
    }

    fn value(&self, ctx: &StrategyContext) -> Option<f64> {
        ctx.indicator(self.key())
    }

    fn prev_value(&self, ctx: &StrategyContext) -> Option<f64> {
        ctx.indicator_prev(self.key())
    }
}

/// Choppiness Index reference.
#[derive(Debug, Clone)]
pub struct ChoppinessIndexRef {
    pub period: usize,
    key: String,
}

/// Create a Choppiness Index reference.
#[inline]
pub fn choppiness_index(period: usize) -> ChoppinessIndexRef {
    ChoppinessIndexRef {
        period,
        key: format!("chop_{period}"),
    }
}

impl IndicatorRef for ChoppinessIndexRef {
    fn key(&self) -> &str {
        &self.key
    }

    fn required_indicators(&self) -> Vec<(String, Indicator)> {
        vec![(self.key.clone(), Indicator::ChoppinessIndex(self.period))]
    }

    fn value(&self, ctx: &StrategyContext) -> Option<f64> {
        ctx.indicator(self.key())
    }

    fn prev_value(&self, ctx: &StrategyContext) -> Option<f64> {
        ctx.indicator_prev(self.key())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::backtesting::refs::{coppock_curve, true_range};

    #[test]
    fn test_macd_keys() {
        let m = macd(12, 26, 9);
        assert_eq!(m.line().key(), "macd_line_12_26_9");
        assert_eq!(m.signal_line().key(), "macd_signal_12_26_9");
        assert_eq!(m.histogram().key(), "macd_histogram_12_26_9");
    }

    #[test]
    fn test_supertrend_keys() {
        let st = supertrend(10, 3.0);
        assert_eq!(st.value().key(), "supertrend_value_10_3");
        assert_eq!(st.uptrend().key(), "supertrend_uptrend_10_3");
    }

    #[test]
    fn test_aroon_keys() {
        let ar = aroon(25);
        assert_eq!(ar.up().key(), "aroon_up_25");
        assert_eq!(ar.down().key(), "aroon_down_25");
    }

    #[test]
    fn test_other_keys() {
        assert_eq!(parabolic_sar(0.02, 0.2).key(), "psar_0.02_0.2");
        assert_eq!(true_range().key(), "true_range");
        assert_eq!(coppock_curve(10, 14, 11).key(), "coppock_10_14_11");
    }
}
