use crate::backtesting::strategy::StrategyContext;
use crate::indicators::Indicator;

use super::super::IndicatorRef;

/// On-Balance Volume reference.
#[derive(Debug, Clone, Copy)]
pub struct ObvRef;

impl IndicatorRef for ObvRef {
    fn key(&self) -> &str {
        "obv"
    }

    fn required_indicators(&self) -> Vec<(String, Indicator)> {
        vec![("obv".to_string(), Indicator::Obv)]
    }

    fn value(&self, ctx: &StrategyContext) -> Option<f64> {
        ctx.indicator(self.key())
    }

    fn prev_value(&self, ctx: &StrategyContext) -> Option<f64> {
        ctx.indicator_prev(self.key())
    }
}

/// Create an On-Balance Volume reference.
#[inline]
pub fn obv() -> ObvRef {
    ObvRef
}

/// Volume Weighted Average Price reference.
#[derive(Debug, Clone, Copy)]
pub struct VwapRef;

impl IndicatorRef for VwapRef {
    fn key(&self) -> &str {
        "vwap"
    }

    fn required_indicators(&self) -> Vec<(String, Indicator)> {
        vec![("vwap".to_string(), Indicator::Vwap)]
    }

    fn value(&self, ctx: &StrategyContext) -> Option<f64> {
        ctx.indicator(self.key())
    }

    fn prev_value(&self, ctx: &StrategyContext) -> Option<f64> {
        ctx.indicator_prev(self.key())
    }
}

/// Create a Volume Weighted Average Price reference.
#[inline]
pub fn vwap() -> VwapRef {
    VwapRef
}

/// Chaikin Money Flow reference.
#[derive(Debug, Clone)]
pub struct CmfRef {
    pub period: usize,
    key: String,
}

impl IndicatorRef for CmfRef {
    fn key(&self) -> &str {
        &self.key
    }

    fn required_indicators(&self) -> Vec<(String, Indicator)> {
        vec![(self.key.clone(), Indicator::Cmf(self.period))]
    }

    fn value(&self, ctx: &StrategyContext) -> Option<f64> {
        ctx.indicator(self.key())
    }

    fn prev_value(&self, ctx: &StrategyContext) -> Option<f64> {
        ctx.indicator_prev(self.key())
    }
}

/// Create a Chaikin Money Flow reference.
#[inline]
pub fn cmf(period: usize) -> CmfRef {
    CmfRef {
        period,
        key: format!("cmf_{period}"),
    }
}

/// Accumulation/Distribution reference.
#[derive(Debug, Clone, Copy)]
pub struct AccumulationDistributionRef;

/// Create an Accumulation/Distribution reference.
#[inline]
pub fn accumulation_distribution() -> AccumulationDistributionRef {
    AccumulationDistributionRef
}

impl IndicatorRef for AccumulationDistributionRef {
    fn key(&self) -> &str {
        "ad"
    }

    fn required_indicators(&self) -> Vec<(String, Indicator)> {
        vec![("ad".to_string(), Indicator::AccumulationDistribution)]
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
    use crate::backtesting::refs::{balance_of_power, chaikin_oscillator, mfi};

    #[test]
    fn test_volume_keys() {
        assert_eq!(obv().key(), "obv");
        assert_eq!(vwap().key(), "vwap");
        assert_eq!(mfi(14).key(), "mfi_14");
        assert_eq!(cmf(20).key(), "cmf_20");
        assert_eq!(chaikin_oscillator().key(), "chaikin_osc");
        assert_eq!(accumulation_distribution().key(), "ad");
        assert_eq!(balance_of_power(Some(14)).key(), "bop_14");
        assert_eq!(balance_of_power(None).key(), "bop");
    }
}
