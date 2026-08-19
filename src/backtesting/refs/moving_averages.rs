use crate::backtesting::strategy::StrategyContext;
use crate::indicators::Indicator;

use super::IndicatorRef;

/// Simple Moving Average reference.
#[derive(Debug, Clone)]
pub struct SmaRef {
    pub period: usize,
    key: String,
}

impl IndicatorRef for SmaRef {
    fn key(&self) -> &str {
        &self.key
    }

    fn required_indicators(&self) -> Vec<(String, Indicator)> {
        vec![(self.key.clone(), Indicator::Sma(self.period))]
    }

    fn value(&self, ctx: &StrategyContext) -> Option<f64> {
        ctx.indicator(self.key())
    }

    fn prev_value(&self, ctx: &StrategyContext) -> Option<f64> {
        ctx.indicator_prev(self.key())
    }
}

/// Create a Simple Moving Average reference.
///
/// # Example
///
/// ```ignore
/// use finance_query::backtesting::refs::*;
///
/// let sma_20 = sma(20);
/// let golden_cross = sma(50).crosses_above_ref(sma(200));
/// ```
#[inline]
pub fn sma(period: usize) -> SmaRef {
    SmaRef {
        period,
        key: format!("sma_{period}"),
    }
}

/// Exponential Moving Average reference.
#[derive(Debug, Clone)]
pub struct EmaRef {
    pub period: usize,
    key: String,
}

impl IndicatorRef for EmaRef {
    fn key(&self) -> &str {
        &self.key
    }

    fn required_indicators(&self) -> Vec<(String, Indicator)> {
        vec![(self.key.clone(), Indicator::Ema(self.period))]
    }

    fn value(&self, ctx: &StrategyContext) -> Option<f64> {
        ctx.indicator(self.key())
    }

    fn prev_value(&self, ctx: &StrategyContext) -> Option<f64> {
        ctx.indicator_prev(self.key())
    }
}

/// Create an Exponential Moving Average reference.
#[inline]
pub fn ema(period: usize) -> EmaRef {
    EmaRef {
        period,
        key: format!("ema_{period}"),
    }
}

/// Weighted Moving Average reference.
#[derive(Debug, Clone)]
pub struct WmaRef {
    pub period: usize,
    key: String,
}

impl IndicatorRef for WmaRef {
    fn key(&self) -> &str {
        &self.key
    }

    fn required_indicators(&self) -> Vec<(String, Indicator)> {
        vec![(self.key.clone(), Indicator::Wma(self.period))]
    }

    fn value(&self, ctx: &StrategyContext) -> Option<f64> {
        ctx.indicator(self.key())
    }

    fn prev_value(&self, ctx: &StrategyContext) -> Option<f64> {
        ctx.indicator_prev(self.key())
    }
}

/// Create a Weighted Moving Average reference.
#[inline]
pub fn wma(period: usize) -> WmaRef {
    WmaRef {
        period,
        key: format!("wma_{period}"),
    }
}

/// Double Exponential Moving Average reference.
#[derive(Debug, Clone)]
pub struct DemaRef {
    pub period: usize,
    key: String,
}

impl IndicatorRef for DemaRef {
    fn key(&self) -> &str {
        &self.key
    }

    fn required_indicators(&self) -> Vec<(String, Indicator)> {
        vec![(self.key.clone(), Indicator::Dema(self.period))]
    }

    fn value(&self, ctx: &StrategyContext) -> Option<f64> {
        ctx.indicator(self.key())
    }

    fn prev_value(&self, ctx: &StrategyContext) -> Option<f64> {
        ctx.indicator_prev(self.key())
    }
}

/// Create a Double Exponential Moving Average reference.
#[inline]
pub fn dema(period: usize) -> DemaRef {
    DemaRef {
        period,
        key: format!("dema_{period}"),
    }
}

/// Triple Exponential Moving Average reference.
#[derive(Debug, Clone)]
pub struct TemaRef {
    pub period: usize,
    key: String,
}

impl IndicatorRef for TemaRef {
    fn key(&self) -> &str {
        &self.key
    }

    fn required_indicators(&self) -> Vec<(String, Indicator)> {
        vec![(self.key.clone(), Indicator::Tema(self.period))]
    }

    fn value(&self, ctx: &StrategyContext) -> Option<f64> {
        ctx.indicator(self.key())
    }

    fn prev_value(&self, ctx: &StrategyContext) -> Option<f64> {
        ctx.indicator_prev(self.key())
    }
}

/// Create a Triple Exponential Moving Average reference.
#[inline]
pub fn tema(period: usize) -> TemaRef {
    TemaRef {
        period,
        key: format!("tema_{period}"),
    }
}

/// Hull Moving Average reference.
#[derive(Debug, Clone)]
pub struct HmaRef {
    pub period: usize,
    key: String,
}

impl IndicatorRef for HmaRef {
    fn key(&self) -> &str {
        &self.key
    }

    fn required_indicators(&self) -> Vec<(String, Indicator)> {
        vec![(self.key.clone(), Indicator::Hma(self.period))]
    }

    fn value(&self, ctx: &StrategyContext) -> Option<f64> {
        ctx.indicator(self.key())
    }

    fn prev_value(&self, ctx: &StrategyContext) -> Option<f64> {
        ctx.indicator_prev(self.key())
    }
}

/// Create a Hull Moving Average reference.
#[inline]
pub fn hma(period: usize) -> HmaRef {
    HmaRef {
        period,
        key: format!("hma_{period}"),
    }
}

/// Volume Weighted Moving Average reference.
#[derive(Debug, Clone)]
pub struct VwmaRef {
    pub period: usize,
    key: String,
}

impl IndicatorRef for VwmaRef {
    fn key(&self) -> &str {
        &self.key
    }

    fn required_indicators(&self) -> Vec<(String, Indicator)> {
        vec![(self.key.clone(), Indicator::Vwma(self.period))]
    }

    fn value(&self, ctx: &StrategyContext) -> Option<f64> {
        ctx.indicator(self.key())
    }

    fn prev_value(&self, ctx: &StrategyContext) -> Option<f64> {
        ctx.indicator_prev(self.key())
    }
}

/// Create a Volume Weighted Moving Average reference.
#[inline]
pub fn vwma(period: usize) -> VwmaRef {
    VwmaRef {
        period,
        key: format!("vwma_{period}"),
    }
}

/// McGinley Dynamic indicator reference.
#[derive(Debug, Clone)]
pub struct McginleyDynamicRef {
    pub period: usize,
    key: String,
}

impl IndicatorRef for McginleyDynamicRef {
    fn key(&self) -> &str {
        &self.key
    }

    fn required_indicators(&self) -> Vec<(String, Indicator)> {
        vec![(self.key.clone(), Indicator::McginleyDynamic(self.period))]
    }

    fn value(&self, ctx: &StrategyContext) -> Option<f64> {
        ctx.indicator(self.key())
    }

    fn prev_value(&self, ctx: &StrategyContext) -> Option<f64> {
        ctx.indicator_prev(self.key())
    }
}

/// Create a McGinley Dynamic indicator reference.
#[inline]
pub fn mcginley(period: usize) -> McginleyDynamicRef {
    McginleyDynamicRef {
        period,
        key: format!("mcginley_{period}"),
    }
}

/// ALMA (Arnaud Legoux Moving Average) configuration.
#[derive(Debug, Clone, Copy)]
pub struct AlmaConfig {
    pub period: usize,
    pub offset: f64,
    pub sigma: f64,
}

/// Create an ALMA configuration.
#[inline]
pub fn alma(period: usize, offset: f64, sigma: f64) -> AlmaRef {
    AlmaRef::new(period, offset, sigma)
}

/// ALMA reference.
#[derive(Debug, Clone)]
pub struct AlmaRef {
    pub period: usize,
    pub offset: f64,
    pub sigma: f64,
    key: String,
}

impl AlmaRef {
    fn new(period: usize, offset: f64, sigma: f64) -> Self {
        Self {
            period,
            offset,
            sigma,
            key: format!("alma_{period}_{offset}_{sigma}"),
        }
    }
}

impl IndicatorRef for AlmaRef {
    fn key(&self) -> &str {
        &self.key
    }

    fn required_indicators(&self) -> Vec<(String, Indicator)> {
        vec![(
            self.key.clone(),
            Indicator::Alma {
                period: self.period,
                offset: self.offset,
                sigma: self.sigma,
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
    fn test_moving_average_keys() {
        assert_eq!(sma(20).key(), "sma_20");
        assert_eq!(ema(12).key(), "ema_12");
        assert_eq!(wma(14).key(), "wma_14");
        assert_eq!(dema(21).key(), "dema_21");
        assert_eq!(tema(21).key(), "tema_21");
        assert_eq!(hma(9).key(), "hma_9");
        assert_eq!(vwma(20).key(), "vwma_20");
        assert_eq!(mcginley(14).key(), "mcginley_14");
        assert_eq!(alma(9, 0.85, 6.0).key(), "alma_9_0.85_6");
    }

    #[test]
    fn test_required_indicators() {
        let sma_ref = sma(20);
        let indicators = sma_ref.required_indicators();
        assert_eq!(indicators.len(), 1);
        assert_eq!(indicators[0].0, "sma_20");
        assert!(matches!(indicators[0].1, Indicator::Sma(20)));
    }
}
