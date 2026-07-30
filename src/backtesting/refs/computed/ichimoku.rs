use crate::backtesting::strategy::StrategyContext;
use crate::indicators::Indicator;

use super::super::IndicatorRef;

/// Average Directional Index reference.
/// Ichimoku Cloud configuration.
#[derive(Debug, Clone, Copy)]
pub struct IchimokuConfig {
    pub conversion: usize,
    pub base: usize,
    pub lagging: usize,
    pub displacement: usize,
}

impl IchimokuConfig {
    /// Get the Tenkan-sen (Conversion Line) reference.
    pub fn conversion_line(&self) -> IchimokuConversionRef {
        IchimokuConversionRef::new(self.conversion, self.base, self.lagging, self.displacement)
    }

    /// Get the Kijun-sen (Base Line) reference.
    pub fn base_line(&self) -> IchimokuBaseRef {
        IchimokuBaseRef::new(self.conversion, self.base, self.lagging, self.displacement)
    }

    /// Get the Senkou Span A (Leading Span A) reference.
    pub fn leading_span_a(&self) -> IchimokuLeadingARef {
        IchimokuLeadingARef::new(self.conversion, self.base, self.lagging, self.displacement)
    }

    /// Get the Senkou Span B (Leading Span B) reference.
    pub fn leading_span_b(&self) -> IchimokuLeadingBRef {
        IchimokuLeadingBRef::new(self.conversion, self.base, self.lagging, self.displacement)
    }

    /// Get the Chikou Span (Lagging Span) reference.
    pub fn lagging_span(&self) -> IchimokuLaggingRef {
        IchimokuLaggingRef::new(self.conversion, self.base, self.lagging, self.displacement)
    }
}

/// Create an Ichimoku Cloud configuration with default periods (9, 26, 52, 26).
#[inline]
pub fn ichimoku() -> IchimokuConfig {
    IchimokuConfig {
        conversion: 9,
        base: 26,
        lagging: 52,
        displacement: 26,
    }
}

/// Create an Ichimoku Cloud configuration with custom periods.
#[inline]
pub fn ichimoku_custom(
    conversion: usize,
    base: usize,
    lagging: usize,
    displacement: usize,
) -> IchimokuConfig {
    IchimokuConfig {
        conversion,
        base,
        lagging,
        displacement,
    }
}

/// Ichimoku Conversion Line (Tenkan-sen) reference.
#[derive(Debug, Clone)]
pub struct IchimokuConversionRef {
    pub conversion: usize,
    pub base: usize,
    pub lagging: usize,
    pub displacement: usize,
    key: String,
}

impl IchimokuConversionRef {
    fn new(conversion: usize, base: usize, lagging: usize, displacement: usize) -> Self {
        Self {
            conversion,
            base,
            lagging,
            displacement,
            key: format!("ichimoku_conversion_{conversion}_{base}_{lagging}_{displacement}"),
        }
    }
}

impl IndicatorRef for IchimokuConversionRef {
    fn key(&self) -> &str {
        &self.key
    }

    fn required_indicators(&self) -> Vec<(String, Indicator)> {
        vec![(
            self.key.clone(),
            Indicator::Ichimoku {
                conversion: self.conversion,
                base: self.base,
                lagging: self.lagging,
                displacement: self.displacement,
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

/// Ichimoku Base Line (Kijun-sen) reference.
#[derive(Debug, Clone)]
pub struct IchimokuBaseRef {
    pub conversion: usize,
    pub base: usize,
    pub lagging: usize,
    pub displacement: usize,
    key: String,
}

impl IchimokuBaseRef {
    fn new(conversion: usize, base: usize, lagging: usize, displacement: usize) -> Self {
        Self {
            conversion,
            base,
            lagging,
            displacement,
            key: format!("ichimoku_base_{conversion}_{base}_{lagging}_{displacement}"),
        }
    }
}

impl IndicatorRef for IchimokuBaseRef {
    fn key(&self) -> &str {
        &self.key
    }

    fn required_indicators(&self) -> Vec<(String, Indicator)> {
        vec![(
            self.key.clone(),
            Indicator::Ichimoku {
                conversion: self.conversion,
                base: self.base,
                lagging: self.lagging,
                displacement: self.displacement,
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

/// Ichimoku Leading Span A (Senkou Span A) reference.
#[derive(Debug, Clone)]
pub struct IchimokuLeadingARef {
    pub conversion: usize,
    pub base: usize,
    pub lagging: usize,
    pub displacement: usize,
    key: String,
}

impl IchimokuLeadingARef {
    fn new(conversion: usize, base: usize, lagging: usize, displacement: usize) -> Self {
        Self {
            conversion,
            base,
            lagging,
            displacement,
            key: format!("ichimoku_leading_a_{conversion}_{base}_{lagging}_{displacement}"),
        }
    }
}

impl IndicatorRef for IchimokuLeadingARef {
    fn key(&self) -> &str {
        &self.key
    }

    fn required_indicators(&self) -> Vec<(String, Indicator)> {
        vec![(
            self.key.clone(),
            Indicator::Ichimoku {
                conversion: self.conversion,
                base: self.base,
                lagging: self.lagging,
                displacement: self.displacement,
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

/// Ichimoku Leading Span B (Senkou Span B) reference.
#[derive(Debug, Clone)]
pub struct IchimokuLeadingBRef {
    pub conversion: usize,
    pub base: usize,
    pub lagging: usize,
    pub displacement: usize,
    key: String,
}

impl IchimokuLeadingBRef {
    fn new(conversion: usize, base: usize, lagging: usize, displacement: usize) -> Self {
        Self {
            conversion,
            base,
            lagging,
            displacement,
            key: format!("ichimoku_leading_b_{conversion}_{base}_{lagging}_{displacement}"),
        }
    }
}

impl IndicatorRef for IchimokuLeadingBRef {
    fn key(&self) -> &str {
        &self.key
    }

    fn required_indicators(&self) -> Vec<(String, Indicator)> {
        vec![(
            self.key.clone(),
            Indicator::Ichimoku {
                conversion: self.conversion,
                base: self.base,
                lagging: self.lagging,
                displacement: self.displacement,
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

/// Ichimoku Lagging Span (Chikou Span) reference.
#[derive(Debug, Clone)]
pub struct IchimokuLaggingRef {
    pub conversion: usize,
    pub base: usize,
    pub lagging: usize,
    pub displacement: usize,
    key: String,
}

impl IchimokuLaggingRef {
    fn new(conversion: usize, base: usize, lagging: usize, displacement: usize) -> Self {
        Self {
            conversion,
            base,
            lagging,
            displacement,
            key: format!("ichimoku_lagging_{conversion}_{base}_{lagging}_{displacement}"),
        }
    }
}

impl IndicatorRef for IchimokuLaggingRef {
    fn key(&self) -> &str {
        &self.key
    }

    fn required_indicators(&self) -> Vec<(String, Indicator)> {
        vec![(
            self.key.clone(),
            Indicator::Ichimoku {
                conversion: self.conversion,
                base: self.base,
                lagging: self.lagging,
                displacement: self.displacement,
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ichimoku_keys() {
        let ich = ichimoku();
        assert_eq!(
            ich.conversion_line().key(),
            "ichimoku_conversion_9_26_52_26"
        );
        assert_eq!(ich.base_line().key(), "ichimoku_base_9_26_52_26");
        assert_eq!(ich.leading_span_a().key(), "ichimoku_leading_a_9_26_52_26");
        assert_eq!(ich.leading_span_b().key(), "ichimoku_leading_b_9_26_52_26");
        assert_eq!(ich.lagging_span().key(), "ichimoku_lagging_9_26_52_26");
    }
}
