use crate::backtesting::strategy::StrategyContext;
use crate::indicators::Indicator;

use super::super::IndicatorRef;

/// Bull Power reference.
#[derive(Debug, Clone)]
pub struct BullPowerRef {
    pub period: usize,
    key: String,
}

/// Create a Bull Power reference.
#[inline]
pub fn bull_power(period: usize) -> BullPowerRef {
    BullPowerRef {
        period,
        key: format!("bull_power_{period}"),
    }
}

impl IndicatorRef for BullPowerRef {
    fn key(&self) -> &str {
        &self.key
    }

    fn required_indicators(&self) -> Vec<(String, Indicator)> {
        vec![(self.key.clone(), Indicator::BullBearPower(self.period))]
    }

    fn value(&self, ctx: &StrategyContext) -> Option<f64> {
        ctx.indicator(self.key())
    }

    fn prev_value(&self, ctx: &StrategyContext) -> Option<f64> {
        ctx.indicator_prev(self.key())
    }
}

/// Bear Power reference.
#[derive(Debug, Clone)]
pub struct BearPowerRef {
    pub period: usize,
    key: String,
}

/// Create a Bear Power reference.
#[inline]
pub fn bear_power(period: usize) -> BearPowerRef {
    BearPowerRef {
        period,
        key: format!("bear_power_{period}"),
    }
}

impl IndicatorRef for BearPowerRef {
    fn key(&self) -> &str {
        &self.key
    }

    fn required_indicators(&self) -> Vec<(String, Indicator)> {
        vec![(self.key.clone(), Indicator::BullBearPower(self.period))]
    }

    fn value(&self, ctx: &StrategyContext) -> Option<f64> {
        ctx.indicator(self.key())
    }

    fn prev_value(&self, ctx: &StrategyContext) -> Option<f64> {
        ctx.indicator_prev(self.key())
    }
}

/// Elder Ray Bull Power reference.
#[derive(Debug, Clone)]
pub struct ElderBullPowerRef {
    pub period: usize,
    key: String,
}

/// Create an Elder Ray Bull Power reference.
#[inline]
pub fn elder_bull_power(period: usize) -> ElderBullPowerRef {
    ElderBullPowerRef {
        period,
        key: format!("elder_bull_{period}"),
    }
}

impl IndicatorRef for ElderBullPowerRef {
    fn key(&self) -> &str {
        &self.key
    }

    fn required_indicators(&self) -> Vec<(String, Indicator)> {
        vec![(self.key.clone(), Indicator::ElderRay(self.period))]
    }

    fn value(&self, ctx: &StrategyContext) -> Option<f64> {
        ctx.indicator(self.key())
    }

    fn prev_value(&self, ctx: &StrategyContext) -> Option<f64> {
        ctx.indicator_prev(self.key())
    }
}

/// Elder Ray Bear Power reference.
#[derive(Debug, Clone)]
pub struct ElderBearPowerRef {
    pub period: usize,
    key: String,
}

/// Create an Elder Ray Bear Power reference.
#[inline]
pub fn elder_bear_power(period: usize) -> ElderBearPowerRef {
    ElderBearPowerRef {
        period,
        key: format!("elder_bear_{period}"),
    }
}

impl IndicatorRef for ElderBearPowerRef {
    fn key(&self) -> &str {
        &self.key
    }

    fn required_indicators(&self) -> Vec<(String, Indicator)> {
        vec![(self.key.clone(), Indicator::ElderRay(self.period))]
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
    fn test_power_keys() {
        assert_eq!(bull_power(13).key(), "bull_power_13");
        assert_eq!(bear_power(13).key(), "bear_power_13");
        assert_eq!(elder_bull_power(13).key(), "elder_bull_13");
        assert_eq!(elder_bear_power(13).key(), "elder_bear_13");
    }
}
