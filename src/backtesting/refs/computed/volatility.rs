use crate::backtesting::strategy::StrategyContext;
use crate::indicators::Indicator;

use super::super::IndicatorRef;

/// Average True Range reference.
#[derive(Debug, Clone)]
pub struct AtrRef {
    pub period: usize,
    key: String,
}

impl IndicatorRef for AtrRef {
    fn key(&self) -> &str {
        &self.key
    }

    fn required_indicators(&self) -> Vec<(String, Indicator)> {
        vec![(self.key.clone(), Indicator::Atr(self.period))]
    }

    fn value(&self, ctx: &StrategyContext) -> Option<f64> {
        ctx.indicator(self.key())
    }

    fn prev_value(&self, ctx: &StrategyContext) -> Option<f64> {
        ctx.indicator_prev(self.key())
    }
}

/// Create an Average True Range reference.
#[inline]
pub fn atr(period: usize) -> AtrRef {
    AtrRef {
        period,
        key: format!("atr_{period}"),
    }
}

/// True Range reference.
#[derive(Debug, Clone, Copy)]
pub struct TrueRangeRef;

/// Create a True Range reference.
#[inline]
pub fn true_range() -> TrueRangeRef {
    TrueRangeRef
}

impl IndicatorRef for TrueRangeRef {
    fn key(&self) -> &str {
        "true_range"
    }

    fn required_indicators(&self) -> Vec<(String, Indicator)> {
        vec![("true_range".to_string(), Indicator::TrueRange)]
    }

    fn value(&self, ctx: &StrategyContext) -> Option<f64> {
        ctx.indicator(self.key())
    }

    fn prev_value(&self, ctx: &StrategyContext) -> Option<f64> {
        ctx.indicator_prev(self.key())
    }
}

/// Bollinger Bands configuration.
#[derive(Debug, Clone, Copy)]
pub struct BollingerConfig {
    /// SMA period
    pub period: usize,
    /// Standard deviation multiplier
    pub std_dev: f64,
}

impl BollingerConfig {
    /// Get the upper band reference.
    pub fn upper(&self) -> BollingerUpperRef {
        BollingerUpperRef::new(self.period, self.std_dev)
    }

    /// Get the middle band (SMA) reference.
    pub fn middle(&self) -> BollingerMiddleRef {
        BollingerMiddleRef::new(self.period, self.std_dev)
    }

    /// Get the lower band reference.
    pub fn lower(&self) -> BollingerLowerRef {
        BollingerLowerRef::new(self.period, self.std_dev)
    }
}

/// Create a Bollinger Bands configuration.
///
/// # Example
///
/// ```ignore
/// use finance_query::backtesting::refs::*;
///
/// let bb = bollinger(20, 2.0);
/// let at_lower_band = price().below_ref(bb.lower());
/// let at_upper_band = price().above_ref(bb.upper());
/// ```
#[inline]
pub fn bollinger(period: usize, std_dev: f64) -> BollingerConfig {
    BollingerConfig { period, std_dev }
}

/// Bollinger upper band reference.
#[derive(Debug, Clone)]
pub struct BollingerUpperRef {
    /// Moving average period.
    pub period: usize,
    /// Standard deviation multiplier.
    pub std_dev: f64,
    key: String,
}

impl BollingerUpperRef {
    fn new(period: usize, std_dev: f64) -> Self {
        Self {
            period,
            std_dev,
            key: format!("bollinger_upper_{period}_{std_dev}"),
        }
    }
}

impl IndicatorRef for BollingerUpperRef {
    fn key(&self) -> &str {
        &self.key
    }

    fn required_indicators(&self) -> Vec<(String, Indicator)> {
        vec![(
            self.key.clone(),
            Indicator::Bollinger {
                period: self.period,
                std_dev: self.std_dev,
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

/// Bollinger middle band reference.
#[derive(Debug, Clone)]
pub struct BollingerMiddleRef {
    /// Moving average period.
    pub period: usize,
    /// Standard deviation multiplier.
    pub std_dev: f64,
    key: String,
}

impl BollingerMiddleRef {
    fn new(period: usize, std_dev: f64) -> Self {
        Self {
            period,
            std_dev,
            key: format!("bollinger_middle_{period}_{std_dev}"),
        }
    }
}

impl IndicatorRef for BollingerMiddleRef {
    fn key(&self) -> &str {
        &self.key
    }

    fn required_indicators(&self) -> Vec<(String, Indicator)> {
        vec![(
            self.key.clone(),
            Indicator::Bollinger {
                period: self.period,
                std_dev: self.std_dev,
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

/// Bollinger lower band reference.
#[derive(Debug, Clone)]
pub struct BollingerLowerRef {
    /// Moving average period.
    pub period: usize,
    /// Standard deviation multiplier.
    pub std_dev: f64,
    key: String,
}

impl BollingerLowerRef {
    fn new(period: usize, std_dev: f64) -> Self {
        Self {
            period,
            std_dev,
            key: format!("bollinger_lower_{period}_{std_dev}"),
        }
    }
}

impl IndicatorRef for BollingerLowerRef {
    fn key(&self) -> &str {
        &self.key
    }

    fn required_indicators(&self) -> Vec<(String, Indicator)> {
        vec![(
            self.key.clone(),
            Indicator::Bollinger {
                period: self.period,
                std_dev: self.std_dev,
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

/// Donchian Channels configuration.
#[derive(Debug, Clone, Copy)]
pub struct DonchianConfig {
    pub period: usize,
}

impl DonchianConfig {
    /// Get the upper channel reference.
    pub fn upper(&self) -> DonchianUpperRef {
        DonchianUpperRef::new(self.period)
    }

    /// Get the middle channel reference.
    pub fn middle(&self) -> DonchianMiddleRef {
        DonchianMiddleRef::new(self.period)
    }

    /// Get the lower channel reference.
    pub fn lower(&self) -> DonchianLowerRef {
        DonchianLowerRef::new(self.period)
    }
}

/// Create a Donchian Channels configuration.
#[inline]
pub fn donchian(period: usize) -> DonchianConfig {
    DonchianConfig { period }
}

/// Donchian upper channel reference.
#[derive(Debug, Clone)]
pub struct DonchianUpperRef {
    pub period: usize,
    key: String,
}

impl DonchianUpperRef {
    fn new(period: usize) -> Self {
        Self {
            period,
            key: format!("donchian_upper_{period}"),
        }
    }
}

impl IndicatorRef for DonchianUpperRef {
    fn key(&self) -> &str {
        &self.key
    }

    fn required_indicators(&self) -> Vec<(String, Indicator)> {
        vec![(self.key.clone(), Indicator::DonchianChannels(self.period))]
    }

    fn value(&self, ctx: &StrategyContext) -> Option<f64> {
        ctx.indicator(self.key())
    }

    fn prev_value(&self, ctx: &StrategyContext) -> Option<f64> {
        ctx.indicator_prev(self.key())
    }
}

/// Donchian middle channel reference.
#[derive(Debug, Clone)]
pub struct DonchianMiddleRef {
    pub period: usize,
    key: String,
}

impl DonchianMiddleRef {
    fn new(period: usize) -> Self {
        Self {
            period,
            key: format!("donchian_middle_{period}"),
        }
    }
}

impl IndicatorRef for DonchianMiddleRef {
    fn key(&self) -> &str {
        &self.key
    }

    fn required_indicators(&self) -> Vec<(String, Indicator)> {
        vec![(self.key.clone(), Indicator::DonchianChannels(self.period))]
    }

    fn value(&self, ctx: &StrategyContext) -> Option<f64> {
        ctx.indicator(self.key())
    }

    fn prev_value(&self, ctx: &StrategyContext) -> Option<f64> {
        ctx.indicator_prev(self.key())
    }
}

/// Donchian lower channel reference.
#[derive(Debug, Clone)]
pub struct DonchianLowerRef {
    pub period: usize,
    key: String,
}

impl DonchianLowerRef {
    fn new(period: usize) -> Self {
        Self {
            period,
            key: format!("donchian_lower_{period}"),
        }
    }
}

impl IndicatorRef for DonchianLowerRef {
    fn key(&self) -> &str {
        &self.key
    }

    fn required_indicators(&self) -> Vec<(String, Indicator)> {
        vec![(self.key.clone(), Indicator::DonchianChannels(self.period))]
    }

    fn value(&self, ctx: &StrategyContext) -> Option<f64> {
        ctx.indicator(self.key())
    }

    fn prev_value(&self, ctx: &StrategyContext) -> Option<f64> {
        ctx.indicator_prev(self.key())
    }
}

/// Keltner Channels configuration.
#[derive(Debug, Clone, Copy)]
pub struct KeltnerConfig {
    pub period: usize,
    pub multiplier: f64,
    pub atr_period: usize,
}

impl KeltnerConfig {
    /// Get the upper channel reference.
    pub fn upper(&self) -> KeltnerUpperRef {
        KeltnerUpperRef::new(self.period, self.multiplier, self.atr_period)
    }

    /// Get the middle channel (EMA) reference.
    pub fn middle(&self) -> KeltnerMiddleRef {
        KeltnerMiddleRef::new(self.period, self.multiplier, self.atr_period)
    }

    /// Get the lower channel reference.
    pub fn lower(&self) -> KeltnerLowerRef {
        KeltnerLowerRef::new(self.period, self.multiplier, self.atr_period)
    }
}

/// Create a Keltner Channels configuration.
#[inline]
pub fn keltner(period: usize, multiplier: f64, atr_period: usize) -> KeltnerConfig {
    KeltnerConfig {
        period,
        multiplier,
        atr_period,
    }
}

/// Keltner upper channel reference.
#[derive(Debug, Clone)]
pub struct KeltnerUpperRef {
    pub period: usize,
    pub multiplier: f64,
    pub atr_period: usize,
    key: String,
}

impl KeltnerUpperRef {
    fn new(period: usize, multiplier: f64, atr_period: usize) -> Self {
        Self {
            period,
            multiplier,
            atr_period,
            key: format!("keltner_upper_{period}_{multiplier}_{atr_period}"),
        }
    }
}

impl IndicatorRef for KeltnerUpperRef {
    fn key(&self) -> &str {
        &self.key
    }

    fn required_indicators(&self) -> Vec<(String, Indicator)> {
        vec![(
            self.key.clone(),
            Indicator::KeltnerChannels {
                period: self.period,
                multiplier: self.multiplier,
                atr_period: self.atr_period,
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

/// Keltner middle channel reference.
#[derive(Debug, Clone)]
pub struct KeltnerMiddleRef {
    pub period: usize,
    pub multiplier: f64,
    pub atr_period: usize,
    key: String,
}

impl KeltnerMiddleRef {
    fn new(period: usize, multiplier: f64, atr_period: usize) -> Self {
        Self {
            period,
            multiplier,
            atr_period,
            key: format!("keltner_middle_{period}_{multiplier}_{atr_period}"),
        }
    }
}

impl IndicatorRef for KeltnerMiddleRef {
    fn key(&self) -> &str {
        &self.key
    }

    fn required_indicators(&self) -> Vec<(String, Indicator)> {
        vec![(
            self.key.clone(),
            Indicator::KeltnerChannels {
                period: self.period,
                multiplier: self.multiplier,
                atr_period: self.atr_period,
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

/// Keltner lower channel reference.
#[derive(Debug, Clone)]
pub struct KeltnerLowerRef {
    pub period: usize,
    pub multiplier: f64,
    pub atr_period: usize,
    key: String,
}

impl KeltnerLowerRef {
    fn new(period: usize, multiplier: f64, atr_period: usize) -> Self {
        Self {
            period,
            multiplier,
            atr_period,
            key: format!("keltner_lower_{period}_{multiplier}_{atr_period}"),
        }
    }
}

impl IndicatorRef for KeltnerLowerRef {
    fn key(&self) -> &str {
        &self.key
    }

    fn required_indicators(&self) -> Vec<(String, Indicator)> {
        vec![(
            self.key.clone(),
            Indicator::KeltnerChannels {
                period: self.period,
                multiplier: self.multiplier,
                atr_period: self.atr_period,
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
    fn test_bollinger_keys() {
        let bb = bollinger(20, 2.0);
        assert_eq!(bb.upper().key(), "bollinger_upper_20_2");
        assert_eq!(bb.middle().key(), "bollinger_middle_20_2");
        assert_eq!(bb.lower().key(), "bollinger_lower_20_2");
    }

    #[test]
    fn test_donchian_keys() {
        let dc = donchian(20);
        assert_eq!(dc.upper().key(), "donchian_upper_20");
        assert_eq!(dc.middle().key(), "donchian_middle_20");
        assert_eq!(dc.lower().key(), "donchian_lower_20");
    }

    #[test]
    fn test_keltner_keys() {
        let kc = keltner(20, 2.0, 10);
        assert_eq!(kc.upper().key(), "keltner_upper_20_2_10");
        assert_eq!(kc.middle().key(), "keltner_middle_20_2_10");
        assert_eq!(kc.lower().key(), "keltner_lower_20_2_10");
    }
}
