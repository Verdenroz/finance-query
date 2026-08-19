use crate::backtesting::strategy::StrategyContext;
use crate::indicators::Indicator;

use super::IndicatorRef;

/// Relative Strength Index reference.
#[derive(Debug, Clone)]
pub struct RsiRef {
    pub period: usize,
    key: String,
}

impl IndicatorRef for RsiRef {
    fn key(&self) -> &str {
        &self.key
    }

    fn required_indicators(&self) -> Vec<(String, Indicator)> {
        vec![(self.key.clone(), Indicator::Rsi(self.period))]
    }

    fn value(&self, ctx: &StrategyContext) -> Option<f64> {
        ctx.indicator(self.key())
    }

    fn prev_value(&self, ctx: &StrategyContext) -> Option<f64> {
        ctx.indicator_prev(self.key())
    }
}

/// Create a Relative Strength Index reference.
///
/// # Example
///
/// ```ignore
/// use finance_query::backtesting::refs::*;
///
/// let oversold = rsi(14).below(30.0);
/// let overbought = rsi(14).above(70.0);
/// let exit_oversold = rsi(14).crosses_above(30.0);
/// ```
#[inline]
pub fn rsi(period: usize) -> RsiRef {
    RsiRef {
        period,
        key: format!("rsi_{period}"),
    }
}

/// Commodity Channel Index reference.
#[derive(Debug, Clone)]
pub struct CciRef {
    pub period: usize,
    key: String,
}

impl IndicatorRef for CciRef {
    fn key(&self) -> &str {
        &self.key
    }

    fn required_indicators(&self) -> Vec<(String, Indicator)> {
        vec![(self.key.clone(), Indicator::Cci(self.period))]
    }

    fn value(&self, ctx: &StrategyContext) -> Option<f64> {
        ctx.indicator(self.key())
    }

    fn prev_value(&self, ctx: &StrategyContext) -> Option<f64> {
        ctx.indicator_prev(self.key())
    }
}

/// Create a Commodity Channel Index reference.
#[inline]
pub fn cci(period: usize) -> CciRef {
    CciRef {
        period,
        key: format!("cci_{period}"),
    }
}

/// Williams %R reference.
#[derive(Debug, Clone)]
pub struct WilliamsRRef {
    pub period: usize,
    key: String,
}

impl IndicatorRef for WilliamsRRef {
    fn key(&self) -> &str {
        &self.key
    }

    fn required_indicators(&self) -> Vec<(String, Indicator)> {
        vec![(self.key.clone(), Indicator::WilliamsR(self.period))]
    }

    fn value(&self, ctx: &StrategyContext) -> Option<f64> {
        ctx.indicator(self.key())
    }

    fn prev_value(&self, ctx: &StrategyContext) -> Option<f64> {
        ctx.indicator_prev(self.key())
    }
}

/// Create a Williams %R reference.
#[inline]
pub fn williams_r(period: usize) -> WilliamsRRef {
    WilliamsRRef {
        period,
        key: format!("williams_r_{period}"),
    }
}

/// Chande Momentum Oscillator reference.
#[derive(Debug, Clone)]
pub struct CmoRef {
    pub period: usize,
    key: String,
}

impl IndicatorRef for CmoRef {
    fn key(&self) -> &str {
        &self.key
    }

    fn required_indicators(&self) -> Vec<(String, Indicator)> {
        vec![(self.key.clone(), Indicator::Cmo(self.period))]
    }

    fn value(&self, ctx: &StrategyContext) -> Option<f64> {
        ctx.indicator(self.key())
    }

    fn prev_value(&self, ctx: &StrategyContext) -> Option<f64> {
        ctx.indicator_prev(self.key())
    }
}

/// Create a Chande Momentum Oscillator reference.
#[inline]
pub fn cmo(period: usize) -> CmoRef {
    CmoRef {
        period,
        key: format!("cmo_{period}"),
    }
}

/// Momentum indicator reference.
#[derive(Debug, Clone)]
pub struct MomentumRef {
    pub period: usize,
    key: String,
}

impl IndicatorRef for MomentumRef {
    fn key(&self) -> &str {
        &self.key
    }

    fn required_indicators(&self) -> Vec<(String, Indicator)> {
        vec![(self.key.clone(), Indicator::Momentum(self.period))]
    }

    fn value(&self, ctx: &StrategyContext) -> Option<f64> {
        ctx.indicator(self.key())
    }

    fn prev_value(&self, ctx: &StrategyContext) -> Option<f64> {
        ctx.indicator_prev(self.key())
    }
}

/// Create a Momentum indicator reference.
#[inline]
pub fn momentum(period: usize) -> MomentumRef {
    MomentumRef {
        period,
        key: format!("momentum_{period}"),
    }
}

/// Rate of Change reference.
#[derive(Debug, Clone)]
pub struct RocRef {
    pub period: usize,
    key: String,
}

impl IndicatorRef for RocRef {
    fn key(&self) -> &str {
        &self.key
    }

    fn required_indicators(&self) -> Vec<(String, Indicator)> {
        vec![(self.key.clone(), Indicator::Roc(self.period))]
    }

    fn value(&self, ctx: &StrategyContext) -> Option<f64> {
        ctx.indicator(self.key())
    }

    fn prev_value(&self, ctx: &StrategyContext) -> Option<f64> {
        ctx.indicator_prev(self.key())
    }
}

/// Create a Rate of Change reference.
#[inline]
pub fn roc(period: usize) -> RocRef {
    RocRef {
        period,
        key: format!("roc_{period}"),
    }
}

/// Stochastic Oscillator configuration.
#[derive(Debug, Clone, Copy)]
pub struct StochasticConfig {
    pub k_period: usize,
    pub k_slow: usize,
    pub d_period: usize,
}

impl StochasticConfig {
    /// Get the %K line reference.
    pub fn k(&self) -> StochasticKRef {
        StochasticKRef::new(self.k_period, self.k_slow, self.d_period)
    }

    /// Get the %D line reference.
    pub fn d(&self) -> StochasticDRef {
        StochasticDRef::new(self.k_period, self.k_slow, self.d_period)
    }
}

/// Create a Stochastic Oscillator configuration.
#[inline]
pub fn stochastic(k_period: usize, k_slow: usize, d_period: usize) -> StochasticConfig {
    StochasticConfig {
        k_period,
        k_slow,
        d_period,
    }
}

/// Stochastic %K line reference.
#[derive(Debug, Clone)]
pub struct StochasticKRef {
    pub k_period: usize,
    pub k_slow: usize,
    pub d_period: usize,
    key: String,
}

impl StochasticKRef {
    fn new(k_period: usize, k_slow: usize, d_period: usize) -> Self {
        Self {
            k_period,
            k_slow,
            d_period,
            key: format!("stochastic_k_{k_period}_{k_slow}_{d_period}"),
        }
    }
}

impl IndicatorRef for StochasticKRef {
    fn key(&self) -> &str {
        &self.key
    }

    fn required_indicators(&self) -> Vec<(String, Indicator)> {
        vec![(
            self.key.clone(),
            Indicator::Stochastic {
                k_period: self.k_period,
                k_slow: self.k_slow,
                d_period: self.d_period,
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

/// Stochastic %D line reference.
#[derive(Debug, Clone)]
pub struct StochasticDRef {
    pub k_period: usize,
    pub k_slow: usize,
    pub d_period: usize,
    key: String,
}

impl StochasticDRef {
    fn new(k_period: usize, k_slow: usize, d_period: usize) -> Self {
        Self {
            k_period,
            k_slow,
            d_period,
            key: format!("stochastic_d_{k_period}_{k_slow}_{d_period}"),
        }
    }
}

impl IndicatorRef for StochasticDRef {
    fn key(&self) -> &str {
        &self.key
    }

    fn required_indicators(&self) -> Vec<(String, Indicator)> {
        vec![(
            self.key.clone(),
            Indicator::Stochastic {
                k_period: self.k_period,
                k_slow: self.k_slow,
                d_period: self.d_period,
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

/// Stochastic RSI configuration — entry point for building K or D line refs.
///
/// Use [`.k()`](StochasticRsiConfig::k) to reference the smoothed %K line and
/// [`.d()`](StochasticRsiConfig::d) for the %D signal line.  Both resolve
/// against the same underlying `StochasticRsi` indicator computation, so only
/// one indicator fetch is registered regardless of which lines you use.
///
/// # Example
/// ```ignore
/// let srsi = stochastic_rsi(14, 14, 3, 3);
///
/// // K crosses above D — a common bullish signal
/// StrategyBuilder::new("StochRSI K/D Cross")
///     .entry(srsi.k().crosses_above_ref(srsi.d()))
///     .exit(srsi.k().crosses_below_ref(srsi.d()))
///     .build()
/// ```
#[derive(Debug, Clone, Copy)]
pub struct StochasticRsiConfig {
    pub rsi_period: usize,
    pub stoch_period: usize,
    pub k_period: usize,
    pub d_period: usize,
}

impl StochasticRsiConfig {
    /// Reference to the smoothed %K line.
    pub fn k(&self) -> StochasticRsiRef {
        StochasticRsiRef::new(
            self.rsi_period,
            self.stoch_period,
            self.k_period,
            self.d_period,
        )
    }

    /// Reference to the %D signal line (SMA of %K).
    pub fn d(&self) -> StochasticRsiDRef {
        StochasticRsiDRef::new(
            self.rsi_period,
            self.stoch_period,
            self.k_period,
            self.d_period,
        )
    }
}

/// Create a Stochastic RSI configuration.
///
/// Returns a [`StochasticRsiConfig`] from which you can obtain
/// [`StochasticRsiConfig::k()`] or [`StochasticRsiConfig::d()`] refs.
/// Calling `stochastic_rsi(...).k()` is equivalent to the previous API that
/// returned `StochasticRsiRef` directly.
#[inline]
pub fn stochastic_rsi(
    rsi_period: usize,
    stoch_period: usize,
    k_period: usize,
    d_period: usize,
) -> StochasticRsiConfig {
    StochasticRsiConfig {
        rsi_period,
        stoch_period,
        k_period,
        d_period,
    }
}

/// Stochastic RSI %K line reference.
#[derive(Debug, Clone)]
pub struct StochasticRsiRef {
    pub rsi_period: usize,
    pub stoch_period: usize,
    pub k_period: usize,
    pub d_period: usize,
    key: String,
}

impl StochasticRsiRef {
    fn new(rsi_period: usize, stoch_period: usize, k_period: usize, d_period: usize) -> Self {
        Self {
            rsi_period,
            stoch_period,
            k_period,
            d_period,
            key: format!("stoch_rsi_k_{rsi_period}_{stoch_period}_{k_period}_{d_period}"),
        }
    }
}

impl IndicatorRef for StochasticRsiRef {
    fn key(&self) -> &str {
        &self.key
    }

    fn required_indicators(&self) -> Vec<(String, Indicator)> {
        vec![(
            self.key.clone(),
            Indicator::StochasticRsi {
                rsi_period: self.rsi_period,
                stoch_period: self.stoch_period,
                k_period: self.k_period,
                d_period: self.d_period,
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

/// Stochastic RSI %D line reference (SMA of %K).
#[derive(Debug, Clone)]
pub struct StochasticRsiDRef {
    pub rsi_period: usize,
    pub stoch_period: usize,
    pub k_period: usize,
    pub d_period: usize,
    key: String,
}

impl StochasticRsiDRef {
    fn new(rsi_period: usize, stoch_period: usize, k_period: usize, d_period: usize) -> Self {
        Self {
            rsi_period,
            stoch_period,
            k_period,
            d_period,
            key: format!("stoch_rsi_d_{rsi_period}_{stoch_period}_{k_period}_{d_period}"),
        }
    }
}

impl IndicatorRef for StochasticRsiDRef {
    fn key(&self) -> &str {
        &self.key
    }

    fn required_indicators(&self) -> Vec<(String, Indicator)> {
        // Registering the K key is sufficient — the engine computes both K and D
        // from the same StochasticRsi indicator pass and stores both keys.
        let k_key = format!(
            "stoch_rsi_k_{}_{}_{}_{}",
            self.rsi_period, self.stoch_period, self.k_period, self.d_period
        );
        vec![(
            k_key,
            Indicator::StochasticRsi {
                rsi_period: self.rsi_period,
                stoch_period: self.stoch_period,
                k_period: self.k_period,
                d_period: self.d_period,
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

/// Awesome Oscillator reference (uses default 5/34 periods).
#[derive(Debug, Clone)]
pub struct AwesomeOscillatorRef {
    pub fast: usize,
    pub slow: usize,
    key: String,
}

/// Create an Awesome Oscillator reference.
#[inline]
pub fn awesome_oscillator(fast: usize, slow: usize) -> AwesomeOscillatorRef {
    AwesomeOscillatorRef {
        fast,
        slow,
        key: format!("ao_{fast}_{slow}"),
    }
}

impl IndicatorRef for AwesomeOscillatorRef {
    fn key(&self) -> &str {
        &self.key
    }

    fn required_indicators(&self) -> Vec<(String, Indicator)> {
        vec![(
            self.key.clone(),
            Indicator::AwesomeOscillator {
                fast: self.fast,
                slow: self.slow,
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

/// Coppock Curve reference.
#[derive(Debug, Clone)]
pub struct CoppockCurveRef {
    pub wma_period: usize,
    pub long_roc: usize,
    pub short_roc: usize,
    key: String,
}

/// Create a Coppock Curve reference (uses default 10/14/11 periods).
#[inline]
pub fn coppock_curve(wma_period: usize, long_roc: usize, short_roc: usize) -> CoppockCurveRef {
    CoppockCurveRef {
        wma_period,
        long_roc,
        short_roc,
        key: format!("coppock_{wma_period}_{long_roc}_{short_roc}"),
    }
}

impl IndicatorRef for CoppockCurveRef {
    fn key(&self) -> &str {
        &self.key
    }

    fn required_indicators(&self) -> Vec<(String, Indicator)> {
        vec![(
            self.key.clone(),
            Indicator::CoppockCurve {
                wma_period: self.wma_period,
                long_roc: self.long_roc,
                short_roc: self.short_roc,
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

/// Money Flow Index reference.
#[derive(Debug, Clone)]
pub struct MfiRef {
    pub period: usize,
    key: String,
}

impl IndicatorRef for MfiRef {
    fn key(&self) -> &str {
        &self.key
    }

    fn required_indicators(&self) -> Vec<(String, Indicator)> {
        vec![(self.key.clone(), Indicator::Mfi(self.period))]
    }

    fn value(&self, ctx: &StrategyContext) -> Option<f64> {
        ctx.indicator(self.key())
    }

    fn prev_value(&self, ctx: &StrategyContext) -> Option<f64> {
        ctx.indicator_prev(self.key())
    }
}

/// Create a Money Flow Index reference.
#[inline]
pub fn mfi(period: usize) -> MfiRef {
    MfiRef {
        period,
        key: format!("mfi_{period}"),
    }
}

/// Chaikin Oscillator reference.
#[derive(Debug, Clone, Copy)]
pub struct ChaikinOscillatorRef;

/// Create a Chaikin Oscillator reference.
#[inline]
pub fn chaikin_oscillator() -> ChaikinOscillatorRef {
    ChaikinOscillatorRef
}

impl IndicatorRef for ChaikinOscillatorRef {
    fn key(&self) -> &str {
        "chaikin_osc"
    }

    fn required_indicators(&self) -> Vec<(String, Indicator)> {
        vec![("chaikin_osc".to_string(), Indicator::ChaikinOscillator)]
    }

    fn value(&self, ctx: &StrategyContext) -> Option<f64> {
        ctx.indicator(self.key())
    }

    fn prev_value(&self, ctx: &StrategyContext) -> Option<f64> {
        ctx.indicator_prev(self.key())
    }
}

/// Balance of Power reference.
#[derive(Debug, Clone)]
pub struct BalanceOfPowerRef {
    pub period: Option<usize>,
    key: String,
}

/// Create a Balance of Power reference.
#[inline]
pub fn balance_of_power(period: Option<usize>) -> BalanceOfPowerRef {
    let key = match period {
        Some(p) => format!("bop_{p}"),
        None => "bop".to_string(),
    };
    BalanceOfPowerRef { period, key }
}

impl IndicatorRef for BalanceOfPowerRef {
    fn key(&self) -> &str {
        &self.key
    }

    fn required_indicators(&self) -> Vec<(String, Indicator)> {
        vec![(self.key.clone(), Indicator::BalanceOfPower(self.period))]
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
    use crate::backtesting::refs::choppiness_index;

    #[test]
    fn test_oscillator_keys() {
        assert_eq!(rsi(14).key(), "rsi_14");
        assert_eq!(cci(20).key(), "cci_20");
        assert_eq!(williams_r(14).key(), "williams_r_14");
        assert_eq!(cmo(14).key(), "cmo_14");
        // stochastic_rsi() returns StochasticRsiConfig; .k()/.d() give line refs
        assert_eq!(
            stochastic_rsi(14, 14, 3, 3).k().key(),
            "stoch_rsi_k_14_14_3_3"
        );
        assert_eq!(
            stochastic_rsi(14, 14, 3, 3).d().key(),
            "stoch_rsi_d_14_14_3_3"
        );
        assert_eq!(awesome_oscillator(5, 34).key(), "ao_5_34");
        assert_eq!(choppiness_index(14).key(), "chop_14");
    }

    #[test]
    fn test_stochastic_keys() {
        let stoch = stochastic(14, 3, 3);
        assert_eq!(stoch.k().key(), "stochastic_k_14_3_3");
        assert_eq!(stoch.d().key(), "stochastic_d_14_3_3");
    }
}
