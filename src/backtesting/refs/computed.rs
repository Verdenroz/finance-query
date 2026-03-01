//! Computed indicator references for ALL technical indicators.
//!
//! This module provides references to pre-computed technical indicators
//! that can be used in trading conditions.
//!
//! # Categories
//!
//! - **Moving Averages**: SMA, EMA, WMA, DEMA, TEMA, HMA, VWMA, ALMA, McGinley Dynamic
//! - **Oscillators**: RSI, Stochastic, StochasticRSI, CCI, Williams %R, CMO, Awesome Oscillator
//! - **Trend**: MACD, ADX, Aroon, SuperTrend, Ichimoku, Parabolic SAR
//! - **Volatility**: ATR, Bollinger Bands, Keltner Channels, Donchian Channels
//! - **Volume**: OBV, VWAP, MFI, CMF, Chaikin Oscillator, A/D, Balance of Power
//! - **Momentum**: Momentum, ROC, Coppock Curve, Bull/Bear Power, Elder Ray

// Allow missing docs on struct fields in this file - users interact via
// fluent API functions (sma(), rsi(), etc.) rather than these internal types.
#![allow(missing_docs)]

use crate::backtesting::strategy::StrategyContext;
use crate::indicators::Indicator;

use super::IndicatorRef;

// ============================================================================
// MOVING AVERAGES
// ============================================================================

/// Simple Moving Average reference.
#[derive(Debug, Clone, Copy)]
pub struct SmaRef(pub usize);

impl IndicatorRef for SmaRef {
    fn key(&self) -> String {
        format!("sma_{}", self.0)
    }

    fn required_indicators(&self) -> Vec<(String, Indicator)> {
        vec![(self.key(), Indicator::Sma(self.0))]
    }

    fn value(&self, ctx: &StrategyContext) -> Option<f64> {
        ctx.indicator(&self.key())
    }

    fn prev_value(&self, ctx: &StrategyContext) -> Option<f64> {
        ctx.indicator_prev(&self.key())
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
    SmaRef(period)
}

/// Exponential Moving Average reference.
#[derive(Debug, Clone, Copy)]
pub struct EmaRef(pub usize);

impl IndicatorRef for EmaRef {
    fn key(&self) -> String {
        format!("ema_{}", self.0)
    }

    fn required_indicators(&self) -> Vec<(String, Indicator)> {
        vec![(self.key(), Indicator::Ema(self.0))]
    }

    fn value(&self, ctx: &StrategyContext) -> Option<f64> {
        ctx.indicator(&self.key())
    }

    fn prev_value(&self, ctx: &StrategyContext) -> Option<f64> {
        ctx.indicator_prev(&self.key())
    }
}

/// Create an Exponential Moving Average reference.
#[inline]
pub fn ema(period: usize) -> EmaRef {
    EmaRef(period)
}

/// Weighted Moving Average reference.
#[derive(Debug, Clone, Copy)]
pub struct WmaRef(pub usize);

impl IndicatorRef for WmaRef {
    fn key(&self) -> String {
        format!("wma_{}", self.0)
    }

    fn required_indicators(&self) -> Vec<(String, Indicator)> {
        vec![(self.key(), Indicator::Wma(self.0))]
    }

    fn value(&self, ctx: &StrategyContext) -> Option<f64> {
        ctx.indicator(&self.key())
    }

    fn prev_value(&self, ctx: &StrategyContext) -> Option<f64> {
        ctx.indicator_prev(&self.key())
    }
}

/// Create a Weighted Moving Average reference.
#[inline]
pub fn wma(period: usize) -> WmaRef {
    WmaRef(period)
}

/// Double Exponential Moving Average reference.
#[derive(Debug, Clone, Copy)]
pub struct DemaRef(pub usize);

impl IndicatorRef for DemaRef {
    fn key(&self) -> String {
        format!("dema_{}", self.0)
    }

    fn required_indicators(&self) -> Vec<(String, Indicator)> {
        vec![(self.key(), Indicator::Dema(self.0))]
    }

    fn value(&self, ctx: &StrategyContext) -> Option<f64> {
        ctx.indicator(&self.key())
    }

    fn prev_value(&self, ctx: &StrategyContext) -> Option<f64> {
        ctx.indicator_prev(&self.key())
    }
}

/// Create a Double Exponential Moving Average reference.
#[inline]
pub fn dema(period: usize) -> DemaRef {
    DemaRef(period)
}

/// Triple Exponential Moving Average reference.
#[derive(Debug, Clone, Copy)]
pub struct TemaRef(pub usize);

impl IndicatorRef for TemaRef {
    fn key(&self) -> String {
        format!("tema_{}", self.0)
    }

    fn required_indicators(&self) -> Vec<(String, Indicator)> {
        vec![(self.key(), Indicator::Tema(self.0))]
    }

    fn value(&self, ctx: &StrategyContext) -> Option<f64> {
        ctx.indicator(&self.key())
    }

    fn prev_value(&self, ctx: &StrategyContext) -> Option<f64> {
        ctx.indicator_prev(&self.key())
    }
}

/// Create a Triple Exponential Moving Average reference.
#[inline]
pub fn tema(period: usize) -> TemaRef {
    TemaRef(period)
}

/// Hull Moving Average reference.
#[derive(Debug, Clone, Copy)]
pub struct HmaRef(pub usize);

impl IndicatorRef for HmaRef {
    fn key(&self) -> String {
        format!("hma_{}", self.0)
    }

    fn required_indicators(&self) -> Vec<(String, Indicator)> {
        vec![(self.key(), Indicator::Hma(self.0))]
    }

    fn value(&self, ctx: &StrategyContext) -> Option<f64> {
        ctx.indicator(&self.key())
    }

    fn prev_value(&self, ctx: &StrategyContext) -> Option<f64> {
        ctx.indicator_prev(&self.key())
    }
}

/// Create a Hull Moving Average reference.
#[inline]
pub fn hma(period: usize) -> HmaRef {
    HmaRef(period)
}

/// Volume Weighted Moving Average reference.
#[derive(Debug, Clone, Copy)]
pub struct VwmaRef(pub usize);

impl IndicatorRef for VwmaRef {
    fn key(&self) -> String {
        format!("vwma_{}", self.0)
    }

    fn required_indicators(&self) -> Vec<(String, Indicator)> {
        vec![(self.key(), Indicator::Vwma(self.0))]
    }

    fn value(&self, ctx: &StrategyContext) -> Option<f64> {
        ctx.indicator(&self.key())
    }

    fn prev_value(&self, ctx: &StrategyContext) -> Option<f64> {
        ctx.indicator_prev(&self.key())
    }
}

/// Create a Volume Weighted Moving Average reference.
#[inline]
pub fn vwma(period: usize) -> VwmaRef {
    VwmaRef(period)
}

/// McGinley Dynamic indicator reference.
#[derive(Debug, Clone, Copy)]
pub struct McginleyDynamicRef(pub usize);

impl IndicatorRef for McginleyDynamicRef {
    fn key(&self) -> String {
        format!("mcginley_{}", self.0)
    }

    fn required_indicators(&self) -> Vec<(String, Indicator)> {
        vec![(self.key(), Indicator::McginleyDynamic(self.0))]
    }

    fn value(&self, ctx: &StrategyContext) -> Option<f64> {
        ctx.indicator(&self.key())
    }

    fn prev_value(&self, ctx: &StrategyContext) -> Option<f64> {
        ctx.indicator_prev(&self.key())
    }
}

/// Create a McGinley Dynamic indicator reference.
#[inline]
pub fn mcginley(period: usize) -> McginleyDynamicRef {
    McginleyDynamicRef(period)
}

// ============================================================================
// OSCILLATORS
// ============================================================================

/// Relative Strength Index reference.
#[derive(Debug, Clone, Copy)]
pub struct RsiRef(pub usize);

impl IndicatorRef for RsiRef {
    fn key(&self) -> String {
        format!("rsi_{}", self.0)
    }

    fn required_indicators(&self) -> Vec<(String, Indicator)> {
        vec![(self.key(), Indicator::Rsi(self.0))]
    }

    fn value(&self, ctx: &StrategyContext) -> Option<f64> {
        ctx.indicator(&self.key())
    }

    fn prev_value(&self, ctx: &StrategyContext) -> Option<f64> {
        ctx.indicator_prev(&self.key())
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
    RsiRef(period)
}

/// Commodity Channel Index reference.
#[derive(Debug, Clone, Copy)]
pub struct CciRef(pub usize);

impl IndicatorRef for CciRef {
    fn key(&self) -> String {
        format!("cci_{}", self.0)
    }

    fn required_indicators(&self) -> Vec<(String, Indicator)> {
        vec![(self.key(), Indicator::Cci(self.0))]
    }

    fn value(&self, ctx: &StrategyContext) -> Option<f64> {
        ctx.indicator(&self.key())
    }

    fn prev_value(&self, ctx: &StrategyContext) -> Option<f64> {
        ctx.indicator_prev(&self.key())
    }
}

/// Create a Commodity Channel Index reference.
#[inline]
pub fn cci(period: usize) -> CciRef {
    CciRef(period)
}

/// Williams %R reference.
#[derive(Debug, Clone, Copy)]
pub struct WilliamsRRef(pub usize);

impl IndicatorRef for WilliamsRRef {
    fn key(&self) -> String {
        format!("williams_r_{}", self.0)
    }

    fn required_indicators(&self) -> Vec<(String, Indicator)> {
        vec![(self.key(), Indicator::WilliamsR(self.0))]
    }

    fn value(&self, ctx: &StrategyContext) -> Option<f64> {
        ctx.indicator(&self.key())
    }

    fn prev_value(&self, ctx: &StrategyContext) -> Option<f64> {
        ctx.indicator_prev(&self.key())
    }
}

/// Create a Williams %R reference.
#[inline]
pub fn williams_r(period: usize) -> WilliamsRRef {
    WilliamsRRef(period)
}

/// Chande Momentum Oscillator reference.
#[derive(Debug, Clone, Copy)]
pub struct CmoRef(pub usize);

impl IndicatorRef for CmoRef {
    fn key(&self) -> String {
        format!("cmo_{}", self.0)
    }

    fn required_indicators(&self) -> Vec<(String, Indicator)> {
        vec![(self.key(), Indicator::Cmo(self.0))]
    }

    fn value(&self, ctx: &StrategyContext) -> Option<f64> {
        ctx.indicator(&self.key())
    }

    fn prev_value(&self, ctx: &StrategyContext) -> Option<f64> {
        ctx.indicator_prev(&self.key())
    }
}

/// Create a Chande Momentum Oscillator reference.
#[inline]
pub fn cmo(period: usize) -> CmoRef {
    CmoRef(period)
}

// ============================================================================
// MOMENTUM INDICATORS
// ============================================================================

/// Momentum indicator reference.
#[derive(Debug, Clone, Copy)]
pub struct MomentumRef(pub usize);

impl IndicatorRef for MomentumRef {
    fn key(&self) -> String {
        format!("momentum_{}", self.0)
    }

    fn required_indicators(&self) -> Vec<(String, Indicator)> {
        vec![(self.key(), Indicator::Momentum(self.0))]
    }

    fn value(&self, ctx: &StrategyContext) -> Option<f64> {
        ctx.indicator(&self.key())
    }

    fn prev_value(&self, ctx: &StrategyContext) -> Option<f64> {
        ctx.indicator_prev(&self.key())
    }
}

/// Create a Momentum indicator reference.
#[inline]
pub fn momentum(period: usize) -> MomentumRef {
    MomentumRef(period)
}

/// Rate of Change reference.
#[derive(Debug, Clone, Copy)]
pub struct RocRef(pub usize);

impl IndicatorRef for RocRef {
    fn key(&self) -> String {
        format!("roc_{}", self.0)
    }

    fn required_indicators(&self) -> Vec<(String, Indicator)> {
        vec![(self.key(), Indicator::Roc(self.0))]
    }

    fn value(&self, ctx: &StrategyContext) -> Option<f64> {
        ctx.indicator(&self.key())
    }

    fn prev_value(&self, ctx: &StrategyContext) -> Option<f64> {
        ctx.indicator_prev(&self.key())
    }
}

/// Create a Rate of Change reference.
#[inline]
pub fn roc(period: usize) -> RocRef {
    RocRef(period)
}

// ============================================================================
// TREND INDICATORS
// ============================================================================

/// Average Directional Index reference.
#[derive(Debug, Clone, Copy)]
pub struct AdxRef(pub usize);

impl IndicatorRef for AdxRef {
    fn key(&self) -> String {
        format!("adx_{}", self.0)
    }

    fn required_indicators(&self) -> Vec<(String, Indicator)> {
        vec![(self.key(), Indicator::Adx(self.0))]
    }

    fn value(&self, ctx: &StrategyContext) -> Option<f64> {
        ctx.indicator(&self.key())
    }

    fn prev_value(&self, ctx: &StrategyContext) -> Option<f64> {
        ctx.indicator_prev(&self.key())
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
    AdxRef(period)
}

// ============================================================================
// VOLATILITY INDICATORS
// ============================================================================

/// Average True Range reference.
#[derive(Debug, Clone, Copy)]
pub struct AtrRef(pub usize);

impl IndicatorRef for AtrRef {
    fn key(&self) -> String {
        format!("atr_{}", self.0)
    }

    fn required_indicators(&self) -> Vec<(String, Indicator)> {
        vec![(self.key(), Indicator::Atr(self.0))]
    }

    fn value(&self, ctx: &StrategyContext) -> Option<f64> {
        ctx.indicator(&self.key())
    }

    fn prev_value(&self, ctx: &StrategyContext) -> Option<f64> {
        ctx.indicator_prev(&self.key())
    }
}

/// Create an Average True Range reference.
#[inline]
pub fn atr(period: usize) -> AtrRef {
    AtrRef(period)
}

// ============================================================================
// VOLUME INDICATORS
// ============================================================================

/// On-Balance Volume reference.
#[derive(Debug, Clone, Copy)]
pub struct ObvRef;

impl IndicatorRef for ObvRef {
    fn key(&self) -> String {
        "obv".to_string()
    }

    fn required_indicators(&self) -> Vec<(String, Indicator)> {
        vec![(self.key(), Indicator::Obv)]
    }

    fn value(&self, ctx: &StrategyContext) -> Option<f64> {
        ctx.indicator(&self.key())
    }

    fn prev_value(&self, ctx: &StrategyContext) -> Option<f64> {
        ctx.indicator_prev(&self.key())
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
    fn key(&self) -> String {
        "vwap".to_string()
    }

    fn required_indicators(&self) -> Vec<(String, Indicator)> {
        vec![(self.key(), Indicator::Vwap)]
    }

    fn value(&self, ctx: &StrategyContext) -> Option<f64> {
        ctx.indicator(&self.key())
    }

    fn prev_value(&self, ctx: &StrategyContext) -> Option<f64> {
        ctx.indicator_prev(&self.key())
    }
}

/// Create a Volume Weighted Average Price reference.
#[inline]
pub fn vwap() -> VwapRef {
    VwapRef
}

/// Money Flow Index reference.
#[derive(Debug, Clone, Copy)]
pub struct MfiRef(pub usize);

impl IndicatorRef for MfiRef {
    fn key(&self) -> String {
        format!("mfi_{}", self.0)
    }

    fn required_indicators(&self) -> Vec<(String, Indicator)> {
        vec![(self.key(), Indicator::Mfi(self.0))]
    }

    fn value(&self, ctx: &StrategyContext) -> Option<f64> {
        ctx.indicator(&self.key())
    }

    fn prev_value(&self, ctx: &StrategyContext) -> Option<f64> {
        ctx.indicator_prev(&self.key())
    }
}

/// Create a Money Flow Index reference.
#[inline]
pub fn mfi(period: usize) -> MfiRef {
    MfiRef(period)
}

/// Chaikin Money Flow reference.
#[derive(Debug, Clone, Copy)]
pub struct CmfRef(pub usize);

impl IndicatorRef for CmfRef {
    fn key(&self) -> String {
        format!("cmf_{}", self.0)
    }

    fn required_indicators(&self) -> Vec<(String, Indicator)> {
        vec![(self.key(), Indicator::Cmf(self.0))]
    }

    fn value(&self, ctx: &StrategyContext) -> Option<f64> {
        ctx.indicator(&self.key())
    }

    fn prev_value(&self, ctx: &StrategyContext) -> Option<f64> {
        ctx.indicator_prev(&self.key())
    }
}

/// Create a Chaikin Money Flow reference.
#[inline]
pub fn cmf(period: usize) -> CmfRef {
    CmfRef(period)
}

// ============================================================================
// MULTI-VALUE INDICATORS (with builders)
// ============================================================================

// --- MACD ---

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
        MacdLineRef {
            fast: self.fast,
            slow: self.slow,
            signal: self.signal,
        }
    }

    /// Get the MACD signal line reference.
    pub fn signal_line(&self) -> MacdSignalRef {
        MacdSignalRef {
            fast: self.fast,
            slow: self.slow,
            signal: self.signal,
        }
    }

    /// Get the MACD histogram reference.
    pub fn histogram(&self) -> MacdHistogramRef {
        MacdHistogramRef {
            fast: self.fast,
            slow: self.slow,
            signal: self.signal,
        }
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
#[derive(Debug, Clone, Copy)]
pub struct MacdLineRef {
    /// Fast EMA period.
    pub fast: usize,
    /// Slow EMA period.
    pub slow: usize,
    /// Signal line period.
    pub signal: usize,
}

impl IndicatorRef for MacdLineRef {
    fn key(&self) -> String {
        format!("macd_line_{}_{}_{}", self.fast, self.slow, self.signal)
    }

    fn required_indicators(&self) -> Vec<(String, Indicator)> {
        vec![(
            self.key(),
            Indicator::Macd {
                fast: self.fast,
                slow: self.slow,
                signal: self.signal,
            },
        )]
    }

    fn value(&self, ctx: &StrategyContext) -> Option<f64> {
        ctx.indicator(&self.key())
    }

    fn prev_value(&self, ctx: &StrategyContext) -> Option<f64> {
        ctx.indicator_prev(&self.key())
    }
}

/// MACD Signal Line reference.
#[derive(Debug, Clone, Copy)]
pub struct MacdSignalRef {
    /// Fast EMA period.
    pub fast: usize,
    /// Slow EMA period.
    pub slow: usize,
    /// Signal line period.
    pub signal: usize,
}

impl IndicatorRef for MacdSignalRef {
    fn key(&self) -> String {
        format!("macd_signal_{}_{}_{}", self.fast, self.slow, self.signal)
    }

    fn required_indicators(&self) -> Vec<(String, Indicator)> {
        vec![(
            self.key(),
            Indicator::Macd {
                fast: self.fast,
                slow: self.slow,
                signal: self.signal,
            },
        )]
    }

    fn value(&self, ctx: &StrategyContext) -> Option<f64> {
        ctx.indicator(&self.key())
    }

    fn prev_value(&self, ctx: &StrategyContext) -> Option<f64> {
        ctx.indicator_prev(&self.key())
    }
}

/// MACD Histogram reference.
#[derive(Debug, Clone, Copy)]
pub struct MacdHistogramRef {
    /// Fast EMA period.
    pub fast: usize,
    /// Slow EMA period.
    pub slow: usize,
    /// Signal line period.
    pub signal: usize,
}

impl IndicatorRef for MacdHistogramRef {
    fn key(&self) -> String {
        format!("macd_histogram_{}_{}_{}", self.fast, self.slow, self.signal)
    }

    fn required_indicators(&self) -> Vec<(String, Indicator)> {
        vec![(
            self.key(),
            Indicator::Macd {
                fast: self.fast,
                slow: self.slow,
                signal: self.signal,
            },
        )]
    }

    fn value(&self, ctx: &StrategyContext) -> Option<f64> {
        ctx.indicator(&self.key())
    }

    fn prev_value(&self, ctx: &StrategyContext) -> Option<f64> {
        ctx.indicator_prev(&self.key())
    }
}

// --- Bollinger Bands ---

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
        BollingerUpperRef {
            period: self.period,
            std_dev: self.std_dev,
        }
    }

    /// Get the middle band (SMA) reference.
    pub fn middle(&self) -> BollingerMiddleRef {
        BollingerMiddleRef {
            period: self.period,
            std_dev: self.std_dev,
        }
    }

    /// Get the lower band reference.
    pub fn lower(&self) -> BollingerLowerRef {
        BollingerLowerRef {
            period: self.period,
            std_dev: self.std_dev,
        }
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
#[derive(Debug, Clone, Copy)]
pub struct BollingerUpperRef {
    /// Moving average period.
    pub period: usize,
    /// Standard deviation multiplier.
    pub std_dev: f64,
}

impl IndicatorRef for BollingerUpperRef {
    fn key(&self) -> String {
        format!("bollinger_upper_{}_{}", self.period, self.std_dev)
    }

    fn required_indicators(&self) -> Vec<(String, Indicator)> {
        vec![(
            self.key(),
            Indicator::Bollinger {
                period: self.period,
                std_dev: self.std_dev,
            },
        )]
    }

    fn value(&self, ctx: &StrategyContext) -> Option<f64> {
        ctx.indicator(&self.key())
    }

    fn prev_value(&self, ctx: &StrategyContext) -> Option<f64> {
        ctx.indicator_prev(&self.key())
    }
}

/// Bollinger middle band reference.
#[derive(Debug, Clone, Copy)]
pub struct BollingerMiddleRef {
    /// Moving average period.
    pub period: usize,
    /// Standard deviation multiplier.
    pub std_dev: f64,
}

impl IndicatorRef for BollingerMiddleRef {
    fn key(&self) -> String {
        format!("bollinger_middle_{}_{}", self.period, self.std_dev)
    }

    fn required_indicators(&self) -> Vec<(String, Indicator)> {
        vec![(
            self.key(),
            Indicator::Bollinger {
                period: self.period,
                std_dev: self.std_dev,
            },
        )]
    }

    fn value(&self, ctx: &StrategyContext) -> Option<f64> {
        ctx.indicator(&self.key())
    }

    fn prev_value(&self, ctx: &StrategyContext) -> Option<f64> {
        ctx.indicator_prev(&self.key())
    }
}

/// Bollinger lower band reference.
#[derive(Debug, Clone, Copy)]
pub struct BollingerLowerRef {
    /// Moving average period.
    pub period: usize,
    /// Standard deviation multiplier.
    pub std_dev: f64,
}

impl IndicatorRef for BollingerLowerRef {
    fn key(&self) -> String {
        format!("bollinger_lower_{}_{}", self.period, self.std_dev)
    }

    fn required_indicators(&self) -> Vec<(String, Indicator)> {
        vec![(
            self.key(),
            Indicator::Bollinger {
                period: self.period,
                std_dev: self.std_dev,
            },
        )]
    }

    fn value(&self, ctx: &StrategyContext) -> Option<f64> {
        ctx.indicator(&self.key())
    }

    fn prev_value(&self, ctx: &StrategyContext) -> Option<f64> {
        ctx.indicator_prev(&self.key())
    }
}

// --- Donchian Channels ---

/// Donchian Channels configuration.
#[derive(Debug, Clone, Copy)]
pub struct DonchianConfig {
    pub period: usize,
}

impl DonchianConfig {
    /// Get the upper channel reference.
    pub fn upper(&self) -> DonchianUpperRef {
        DonchianUpperRef {
            period: self.period,
        }
    }

    /// Get the middle channel reference.
    pub fn middle(&self) -> DonchianMiddleRef {
        DonchianMiddleRef {
            period: self.period,
        }
    }

    /// Get the lower channel reference.
    pub fn lower(&self) -> DonchianLowerRef {
        DonchianLowerRef {
            period: self.period,
        }
    }
}

/// Create a Donchian Channels configuration.
#[inline]
pub fn donchian(period: usize) -> DonchianConfig {
    DonchianConfig { period }
}

/// Donchian upper channel reference.
#[derive(Debug, Clone, Copy)]
pub struct DonchianUpperRef {
    pub period: usize,
}

impl IndicatorRef for DonchianUpperRef {
    fn key(&self) -> String {
        format!("donchian_upper_{}", self.period)
    }

    fn required_indicators(&self) -> Vec<(String, Indicator)> {
        vec![(self.key(), Indicator::DonchianChannels(self.period))]
    }

    fn value(&self, ctx: &StrategyContext) -> Option<f64> {
        ctx.indicator(&self.key())
    }

    fn prev_value(&self, ctx: &StrategyContext) -> Option<f64> {
        ctx.indicator_prev(&self.key())
    }
}

/// Donchian middle channel reference.
#[derive(Debug, Clone, Copy)]
pub struct DonchianMiddleRef {
    pub period: usize,
}

impl IndicatorRef for DonchianMiddleRef {
    fn key(&self) -> String {
        format!("donchian_middle_{}", self.period)
    }

    fn required_indicators(&self) -> Vec<(String, Indicator)> {
        vec![(self.key(), Indicator::DonchianChannels(self.period))]
    }

    fn value(&self, ctx: &StrategyContext) -> Option<f64> {
        ctx.indicator(&self.key())
    }

    fn prev_value(&self, ctx: &StrategyContext) -> Option<f64> {
        ctx.indicator_prev(&self.key())
    }
}

/// Donchian lower channel reference.
#[derive(Debug, Clone, Copy)]
pub struct DonchianLowerRef {
    pub period: usize,
}

impl IndicatorRef for DonchianLowerRef {
    fn key(&self) -> String {
        format!("donchian_lower_{}", self.period)
    }

    fn required_indicators(&self) -> Vec<(String, Indicator)> {
        vec![(self.key(), Indicator::DonchianChannels(self.period))]
    }

    fn value(&self, ctx: &StrategyContext) -> Option<f64> {
        ctx.indicator(&self.key())
    }

    fn prev_value(&self, ctx: &StrategyContext) -> Option<f64> {
        ctx.indicator_prev(&self.key())
    }
}

// --- SuperTrend ---

/// SuperTrend configuration.
#[derive(Debug, Clone, Copy)]
pub struct SupertrendConfig {
    pub period: usize,
    pub multiplier: f64,
}

impl SupertrendConfig {
    /// Get the SuperTrend value reference.
    pub fn value(&self) -> SupertrendValueRef {
        SupertrendValueRef {
            period: self.period,
            multiplier: self.multiplier,
        }
    }

    /// Get the SuperTrend uptrend indicator (1.0 = uptrend, 0.0 = downtrend).
    pub fn uptrend(&self) -> SupertrendUptrendRef {
        SupertrendUptrendRef {
            period: self.period,
            multiplier: self.multiplier,
        }
    }
}

/// Create a SuperTrend configuration.
#[inline]
pub fn supertrend(period: usize, multiplier: f64) -> SupertrendConfig {
    SupertrendConfig { period, multiplier }
}

/// SuperTrend value reference.
#[derive(Debug, Clone, Copy)]
pub struct SupertrendValueRef {
    pub period: usize,
    pub multiplier: f64,
}

impl IndicatorRef for SupertrendValueRef {
    fn key(&self) -> String {
        format!("supertrend_value_{}_{}", self.period, self.multiplier)
    }

    fn required_indicators(&self) -> Vec<(String, Indicator)> {
        vec![(
            self.key(),
            Indicator::Supertrend {
                period: self.period,
                multiplier: self.multiplier,
            },
        )]
    }

    fn value(&self, ctx: &StrategyContext) -> Option<f64> {
        ctx.indicator(&self.key())
    }

    fn prev_value(&self, ctx: &StrategyContext) -> Option<f64> {
        ctx.indicator_prev(&self.key())
    }
}

/// SuperTrend uptrend indicator reference.
/// Returns 1.0 for uptrend, 0.0 for downtrend.
#[derive(Debug, Clone, Copy)]
pub struct SupertrendUptrendRef {
    pub period: usize,
    pub multiplier: f64,
}

impl IndicatorRef for SupertrendUptrendRef {
    fn key(&self) -> String {
        format!("supertrend_uptrend_{}_{}", self.period, self.multiplier)
    }

    fn required_indicators(&self) -> Vec<(String, Indicator)> {
        vec![(
            self.key(),
            Indicator::Supertrend {
                period: self.period,
                multiplier: self.multiplier,
            },
        )]
    }

    fn value(&self, ctx: &StrategyContext) -> Option<f64> {
        ctx.indicator(&self.key())
    }

    fn prev_value(&self, ctx: &StrategyContext) -> Option<f64> {
        ctx.indicator_prev(&self.key())
    }
}

// --- Stochastic ---

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
        StochasticKRef {
            k_period: self.k_period,
            k_slow: self.k_slow,
            d_period: self.d_period,
        }
    }

    /// Get the %D line reference.
    pub fn d(&self) -> StochasticDRef {
        StochasticDRef {
            k_period: self.k_period,
            k_slow: self.k_slow,
            d_period: self.d_period,
        }
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
#[derive(Debug, Clone, Copy)]
pub struct StochasticKRef {
    pub k_period: usize,
    pub k_slow: usize,
    pub d_period: usize,
}

impl IndicatorRef for StochasticKRef {
    fn key(&self) -> String {
        format!(
            "stochastic_k_{}_{}_{}",
            self.k_period, self.k_slow, self.d_period
        )
    }

    fn required_indicators(&self) -> Vec<(String, Indicator)> {
        vec![(
            self.key(),
            Indicator::Stochastic {
                k_period: self.k_period,
                k_slow: self.k_slow,
                d_period: self.d_period,
            },
        )]
    }

    fn value(&self, ctx: &StrategyContext) -> Option<f64> {
        ctx.indicator(&self.key())
    }

    fn prev_value(&self, ctx: &StrategyContext) -> Option<f64> {
        ctx.indicator_prev(&self.key())
    }
}

/// Stochastic %D line reference.
#[derive(Debug, Clone, Copy)]
pub struct StochasticDRef {
    pub k_period: usize,
    pub k_slow: usize,
    pub d_period: usize,
}

impl IndicatorRef for StochasticDRef {
    fn key(&self) -> String {
        format!(
            "stochastic_d_{}_{}_{}",
            self.k_period, self.k_slow, self.d_period
        )
    }

    fn required_indicators(&self) -> Vec<(String, Indicator)> {
        vec![(
            self.key(),
            Indicator::Stochastic {
                k_period: self.k_period,
                k_slow: self.k_slow,
                d_period: self.d_period,
            },
        )]
    }

    fn value(&self, ctx: &StrategyContext) -> Option<f64> {
        ctx.indicator(&self.key())
    }

    fn prev_value(&self, ctx: &StrategyContext) -> Option<f64> {
        ctx.indicator_prev(&self.key())
    }
}

// --- Aroon ---

/// Aroon indicator configuration.
#[derive(Debug, Clone, Copy)]
pub struct AroonConfig {
    pub period: usize,
}

impl AroonConfig {
    /// Get the Aroon Up reference.
    pub fn up(&self) -> AroonUpRef {
        AroonUpRef {
            period: self.period,
        }
    }

    /// Get the Aroon Down reference.
    pub fn down(&self) -> AroonDownRef {
        AroonDownRef {
            period: self.period,
        }
    }
}

/// Create an Aroon indicator configuration.
#[inline]
pub fn aroon(period: usize) -> AroonConfig {
    AroonConfig { period }
}

/// Aroon Up reference.
#[derive(Debug, Clone, Copy)]
pub struct AroonUpRef {
    pub period: usize,
}

impl IndicatorRef for AroonUpRef {
    fn key(&self) -> String {
        format!("aroon_up_{}", self.period)
    }

    fn required_indicators(&self) -> Vec<(String, Indicator)> {
        vec![(self.key(), Indicator::Aroon(self.period))]
    }

    fn value(&self, ctx: &StrategyContext) -> Option<f64> {
        ctx.indicator(&self.key())
    }

    fn prev_value(&self, ctx: &StrategyContext) -> Option<f64> {
        ctx.indicator_prev(&self.key())
    }
}

/// Aroon Down reference.
#[derive(Debug, Clone, Copy)]
pub struct AroonDownRef {
    pub period: usize,
}

impl IndicatorRef for AroonDownRef {
    fn key(&self) -> String {
        format!("aroon_down_{}", self.period)
    }

    fn required_indicators(&self) -> Vec<(String, Indicator)> {
        vec![(self.key(), Indicator::Aroon(self.period))]
    }

    fn value(&self, ctx: &StrategyContext) -> Option<f64> {
        ctx.indicator(&self.key())
    }

    fn prev_value(&self, ctx: &StrategyContext) -> Option<f64> {
        ctx.indicator_prev(&self.key())
    }
}

// --- Ichimoku Cloud ---

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
        IchimokuConversionRef {
            conversion: self.conversion,
            base: self.base,
            lagging: self.lagging,
            displacement: self.displacement,
        }
    }

    /// Get the Kijun-sen (Base Line) reference.
    pub fn base_line(&self) -> IchimokuBaseRef {
        IchimokuBaseRef {
            conversion: self.conversion,
            base: self.base,
            lagging: self.lagging,
            displacement: self.displacement,
        }
    }

    /// Get the Senkou Span A (Leading Span A) reference.
    pub fn leading_span_a(&self) -> IchimokuLeadingARef {
        IchimokuLeadingARef {
            conversion: self.conversion,
            base: self.base,
            lagging: self.lagging,
            displacement: self.displacement,
        }
    }

    /// Get the Senkou Span B (Leading Span B) reference.
    pub fn leading_span_b(&self) -> IchimokuLeadingBRef {
        IchimokuLeadingBRef {
            conversion: self.conversion,
            base: self.base,
            lagging: self.lagging,
            displacement: self.displacement,
        }
    }

    /// Get the Chikou Span (Lagging Span) reference.
    pub fn lagging_span(&self) -> IchimokuLaggingRef {
        IchimokuLaggingRef {
            conversion: self.conversion,
            base: self.base,
            lagging: self.lagging,
            displacement: self.displacement,
        }
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
#[derive(Debug, Clone, Copy)]
pub struct IchimokuConversionRef {
    pub conversion: usize,
    pub base: usize,
    pub lagging: usize,
    pub displacement: usize,
}

impl IndicatorRef for IchimokuConversionRef {
    fn key(&self) -> String {
        format!(
            "ichimoku_conversion_{}_{}_{}_{}",
            self.conversion, self.base, self.lagging, self.displacement
        )
    }

    fn required_indicators(&self) -> Vec<(String, Indicator)> {
        vec![(
            self.key(),
            Indicator::Ichimoku {
                conversion: self.conversion,
                base: self.base,
                lagging: self.lagging,
                displacement: self.displacement,
            },
        )]
    }

    fn value(&self, ctx: &StrategyContext) -> Option<f64> {
        ctx.indicator(&self.key())
    }

    fn prev_value(&self, ctx: &StrategyContext) -> Option<f64> {
        ctx.indicator_prev(&self.key())
    }
}

/// Ichimoku Base Line (Kijun-sen) reference.
#[derive(Debug, Clone, Copy)]
pub struct IchimokuBaseRef {
    pub conversion: usize,
    pub base: usize,
    pub lagging: usize,
    pub displacement: usize,
}

impl IndicatorRef for IchimokuBaseRef {
    fn key(&self) -> String {
        format!(
            "ichimoku_base_{}_{}_{}_{}",
            self.conversion, self.base, self.lagging, self.displacement
        )
    }

    fn required_indicators(&self) -> Vec<(String, Indicator)> {
        vec![(
            self.key(),
            Indicator::Ichimoku {
                conversion: self.conversion,
                base: self.base,
                lagging: self.lagging,
                displacement: self.displacement,
            },
        )]
    }

    fn value(&self, ctx: &StrategyContext) -> Option<f64> {
        ctx.indicator(&self.key())
    }

    fn prev_value(&self, ctx: &StrategyContext) -> Option<f64> {
        ctx.indicator_prev(&self.key())
    }
}

/// Ichimoku Leading Span A (Senkou Span A) reference.
#[derive(Debug, Clone, Copy)]
pub struct IchimokuLeadingARef {
    pub conversion: usize,
    pub base: usize,
    pub lagging: usize,
    pub displacement: usize,
}

impl IndicatorRef for IchimokuLeadingARef {
    fn key(&self) -> String {
        format!(
            "ichimoku_leading_a_{}_{}_{}_{}",
            self.conversion, self.base, self.lagging, self.displacement
        )
    }

    fn required_indicators(&self) -> Vec<(String, Indicator)> {
        vec![(
            self.key(),
            Indicator::Ichimoku {
                conversion: self.conversion,
                base: self.base,
                lagging: self.lagging,
                displacement: self.displacement,
            },
        )]
    }

    fn value(&self, ctx: &StrategyContext) -> Option<f64> {
        ctx.indicator(&self.key())
    }

    fn prev_value(&self, ctx: &StrategyContext) -> Option<f64> {
        ctx.indicator_prev(&self.key())
    }
}

/// Ichimoku Leading Span B (Senkou Span B) reference.
#[derive(Debug, Clone, Copy)]
pub struct IchimokuLeadingBRef {
    pub conversion: usize,
    pub base: usize,
    pub lagging: usize,
    pub displacement: usize,
}

impl IndicatorRef for IchimokuLeadingBRef {
    fn key(&self) -> String {
        format!(
            "ichimoku_leading_b_{}_{}_{}_{}",
            self.conversion, self.base, self.lagging, self.displacement
        )
    }

    fn required_indicators(&self) -> Vec<(String, Indicator)> {
        vec![(
            self.key(),
            Indicator::Ichimoku {
                conversion: self.conversion,
                base: self.base,
                lagging: self.lagging,
                displacement: self.displacement,
            },
        )]
    }

    fn value(&self, ctx: &StrategyContext) -> Option<f64> {
        ctx.indicator(&self.key())
    }

    fn prev_value(&self, ctx: &StrategyContext) -> Option<f64> {
        ctx.indicator_prev(&self.key())
    }
}

/// Ichimoku Lagging Span (Chikou Span) reference.
#[derive(Debug, Clone, Copy)]
pub struct IchimokuLaggingRef {
    pub conversion: usize,
    pub base: usize,
    pub lagging: usize,
    pub displacement: usize,
}

impl IndicatorRef for IchimokuLaggingRef {
    fn key(&self) -> String {
        format!(
            "ichimoku_lagging_{}_{}_{}_{}",
            self.conversion, self.base, self.lagging, self.displacement
        )
    }

    fn required_indicators(&self) -> Vec<(String, Indicator)> {
        vec![(
            self.key(),
            Indicator::Ichimoku {
                conversion: self.conversion,
                base: self.base,
                lagging: self.lagging,
                displacement: self.displacement,
            },
        )]
    }

    fn value(&self, ctx: &StrategyContext) -> Option<f64> {
        ctx.indicator(&self.key())
    }

    fn prev_value(&self, ctx: &StrategyContext) -> Option<f64> {
        ctx.indicator_prev(&self.key())
    }
}

// --- Keltner Channels ---

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
        KeltnerUpperRef {
            period: self.period,
            multiplier: self.multiplier,
            atr_period: self.atr_period,
        }
    }

    /// Get the middle channel (EMA) reference.
    pub fn middle(&self) -> KeltnerMiddleRef {
        KeltnerMiddleRef {
            period: self.period,
            multiplier: self.multiplier,
            atr_period: self.atr_period,
        }
    }

    /// Get the lower channel reference.
    pub fn lower(&self) -> KeltnerLowerRef {
        KeltnerLowerRef {
            period: self.period,
            multiplier: self.multiplier,
            atr_period: self.atr_period,
        }
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
#[derive(Debug, Clone, Copy)]
pub struct KeltnerUpperRef {
    pub period: usize,
    pub multiplier: f64,
    pub atr_period: usize,
}

impl IndicatorRef for KeltnerUpperRef {
    fn key(&self) -> String {
        format!(
            "keltner_upper_{}_{}_{}",
            self.period, self.multiplier, self.atr_period
        )
    }

    fn required_indicators(&self) -> Vec<(String, Indicator)> {
        vec![(
            self.key(),
            Indicator::KeltnerChannels {
                period: self.period,
                multiplier: self.multiplier,
                atr_period: self.atr_period,
            },
        )]
    }

    fn value(&self, ctx: &StrategyContext) -> Option<f64> {
        ctx.indicator(&self.key())
    }

    fn prev_value(&self, ctx: &StrategyContext) -> Option<f64> {
        ctx.indicator_prev(&self.key())
    }
}

/// Keltner middle channel reference.
#[derive(Debug, Clone, Copy)]
pub struct KeltnerMiddleRef {
    pub period: usize,
    pub multiplier: f64,
    pub atr_period: usize,
}

impl IndicatorRef for KeltnerMiddleRef {
    fn key(&self) -> String {
        format!(
            "keltner_middle_{}_{}_{}",
            self.period, self.multiplier, self.atr_period
        )
    }

    fn required_indicators(&self) -> Vec<(String, Indicator)> {
        vec![(
            self.key(),
            Indicator::KeltnerChannels {
                period: self.period,
                multiplier: self.multiplier,
                atr_period: self.atr_period,
            },
        )]
    }

    fn value(&self, ctx: &StrategyContext) -> Option<f64> {
        ctx.indicator(&self.key())
    }

    fn prev_value(&self, ctx: &StrategyContext) -> Option<f64> {
        ctx.indicator_prev(&self.key())
    }
}

/// Keltner lower channel reference.
#[derive(Debug, Clone, Copy)]
pub struct KeltnerLowerRef {
    pub period: usize,
    pub multiplier: f64,
    pub atr_period: usize,
}

impl IndicatorRef for KeltnerLowerRef {
    fn key(&self) -> String {
        format!(
            "keltner_lower_{}_{}_{}",
            self.period, self.multiplier, self.atr_period
        )
    }

    fn required_indicators(&self) -> Vec<(String, Indicator)> {
        vec![(
            self.key(),
            Indicator::KeltnerChannels {
                period: self.period,
                multiplier: self.multiplier,
                atr_period: self.atr_period,
            },
        )]
    }

    fn value(&self, ctx: &StrategyContext) -> Option<f64> {
        ctx.indicator(&self.key())
    }

    fn prev_value(&self, ctx: &StrategyContext) -> Option<f64> {
        ctx.indicator_prev(&self.key())
    }
}

// --- Parabolic SAR ---

/// Parabolic SAR configuration.
#[derive(Debug, Clone, Copy)]
pub struct ParabolicSarConfig {
    pub step: f64,
    pub max: f64,
}

/// Create a Parabolic SAR configuration.
#[inline]
pub fn parabolic_sar(step: f64, max: f64) -> ParabolicSarRef {
    ParabolicSarRef { step, max }
}

/// Parabolic SAR reference.
#[derive(Debug, Clone, Copy)]
pub struct ParabolicSarRef {
    pub step: f64,
    pub max: f64,
}

impl IndicatorRef for ParabolicSarRef {
    fn key(&self) -> String {
        format!("psar_{}_{}", self.step, self.max)
    }

    fn required_indicators(&self) -> Vec<(String, Indicator)> {
        vec![(
            self.key(),
            Indicator::ParabolicSar {
                step: self.step,
                max: self.max,
            },
        )]
    }

    fn value(&self, ctx: &StrategyContext) -> Option<f64> {
        ctx.indicator(&self.key())
    }

    fn prev_value(&self, ctx: &StrategyContext) -> Option<f64> {
        ctx.indicator_prev(&self.key())
    }
}

// --- ALMA ---

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
    AlmaRef {
        period,
        offset,
        sigma,
    }
}

/// ALMA reference.
#[derive(Debug, Clone, Copy)]
pub struct AlmaRef {
    pub period: usize,
    pub offset: f64,
    pub sigma: f64,
}

impl IndicatorRef for AlmaRef {
    fn key(&self) -> String {
        format!("alma_{}_{}_{}", self.period, self.offset, self.sigma)
    }

    fn required_indicators(&self) -> Vec<(String, Indicator)> {
        vec![(
            self.key(),
            Indicator::Alma {
                period: self.period,
                offset: self.offset,
                sigma: self.sigma,
            },
        )]
    }

    fn value(&self, ctx: &StrategyContext) -> Option<f64> {
        ctx.indicator(&self.key())
    }

    fn prev_value(&self, ctx: &StrategyContext) -> Option<f64> {
        ctx.indicator_prev(&self.key())
    }
}

// --- Stochastic RSI ---

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
        StochasticRsiRef {
            rsi_period: self.rsi_period,
            stoch_period: self.stoch_period,
            k_period: self.k_period,
            d_period: self.d_period,
        }
    }

    /// Reference to the %D signal line (SMA of %K).
    pub fn d(&self) -> StochasticRsiDRef {
        StochasticRsiDRef {
            rsi_period: self.rsi_period,
            stoch_period: self.stoch_period,
            k_period: self.k_period,
            d_period: self.d_period,
        }
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
#[derive(Debug, Clone, Copy)]
pub struct StochasticRsiRef {
    pub rsi_period: usize,
    pub stoch_period: usize,
    pub k_period: usize,
    pub d_period: usize,
}

impl IndicatorRef for StochasticRsiRef {
    fn key(&self) -> String {
        format!(
            "stoch_rsi_k_{}_{}_{}_{}",
            self.rsi_period, self.stoch_period, self.k_period, self.d_period
        )
    }

    fn required_indicators(&self) -> Vec<(String, Indicator)> {
        vec![(
            self.key(),
            Indicator::StochasticRsi {
                rsi_period: self.rsi_period,
                stoch_period: self.stoch_period,
                k_period: self.k_period,
                d_period: self.d_period,
            },
        )]
    }

    fn value(&self, ctx: &StrategyContext) -> Option<f64> {
        ctx.indicator(&self.key())
    }

    fn prev_value(&self, ctx: &StrategyContext) -> Option<f64> {
        ctx.indicator_prev(&self.key())
    }
}

/// Stochastic RSI %D line reference (SMA of %K).
#[derive(Debug, Clone, Copy)]
pub struct StochasticRsiDRef {
    pub rsi_period: usize,
    pub stoch_period: usize,
    pub k_period: usize,
    pub d_period: usize,
}

impl IndicatorRef for StochasticRsiDRef {
    fn key(&self) -> String {
        format!(
            "stoch_rsi_d_{}_{}_{}_{}",
            self.rsi_period, self.stoch_period, self.k_period, self.d_period
        )
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
        ctx.indicator(&self.key())
    }

    fn prev_value(&self, ctx: &StrategyContext) -> Option<f64> {
        ctx.indicator_prev(&self.key())
    }
}

// --- Awesome Oscillator ---

/// Awesome Oscillator reference (uses default 5/34 periods).
#[derive(Debug, Clone, Copy)]
pub struct AwesomeOscillatorRef {
    pub fast: usize,
    pub slow: usize,
}

/// Create an Awesome Oscillator reference.
#[inline]
pub fn awesome_oscillator(fast: usize, slow: usize) -> AwesomeOscillatorRef {
    AwesomeOscillatorRef { fast, slow }
}

impl IndicatorRef for AwesomeOscillatorRef {
    fn key(&self) -> String {
        format!("ao_{}_{}", self.fast, self.slow)
    }

    fn required_indicators(&self) -> Vec<(String, Indicator)> {
        vec![(
            self.key(),
            Indicator::AwesomeOscillator {
                fast: self.fast,
                slow: self.slow,
            },
        )]
    }

    fn value(&self, ctx: &StrategyContext) -> Option<f64> {
        ctx.indicator(&self.key())
    }

    fn prev_value(&self, ctx: &StrategyContext) -> Option<f64> {
        ctx.indicator_prev(&self.key())
    }
}

// --- Coppock Curve ---

/// Coppock Curve reference.
#[derive(Debug, Clone, Copy)]
pub struct CoppockCurveRef {
    pub wma_period: usize,
    pub long_roc: usize,
    pub short_roc: usize,
}

/// Create a Coppock Curve reference (uses default 10/14/11 periods).
#[inline]
pub fn coppock_curve(wma_period: usize, long_roc: usize, short_roc: usize) -> CoppockCurveRef {
    CoppockCurveRef {
        wma_period,
        long_roc,
        short_roc,
    }
}

impl IndicatorRef for CoppockCurveRef {
    fn key(&self) -> String {
        format!(
            "coppock_{}_{}_{}",
            self.wma_period, self.long_roc, self.short_roc
        )
    }

    fn required_indicators(&self) -> Vec<(String, Indicator)> {
        vec![(
            self.key(),
            Indicator::CoppockCurve {
                wma_period: self.wma_period,
                long_roc: self.long_roc,
                short_roc: self.short_roc,
            },
        )]
    }

    fn value(&self, ctx: &StrategyContext) -> Option<f64> {
        ctx.indicator(&self.key())
    }

    fn prev_value(&self, ctx: &StrategyContext) -> Option<f64> {
        ctx.indicator_prev(&self.key())
    }
}

// --- Choppiness Index ---

/// Choppiness Index reference.
#[derive(Debug, Clone, Copy)]
pub struct ChoppinessIndexRef(pub usize);

/// Create a Choppiness Index reference.
#[inline]
pub fn choppiness_index(period: usize) -> ChoppinessIndexRef {
    ChoppinessIndexRef(period)
}

impl IndicatorRef for ChoppinessIndexRef {
    fn key(&self) -> String {
        format!("chop_{}", self.0)
    }

    fn required_indicators(&self) -> Vec<(String, Indicator)> {
        vec![(self.key(), Indicator::ChoppinessIndex(self.0))]
    }

    fn value(&self, ctx: &StrategyContext) -> Option<f64> {
        ctx.indicator(&self.key())
    }

    fn prev_value(&self, ctx: &StrategyContext) -> Option<f64> {
        ctx.indicator_prev(&self.key())
    }
}

// --- True Range ---

/// True Range reference.
#[derive(Debug, Clone, Copy)]
pub struct TrueRangeRef;

/// Create a True Range reference.
#[inline]
pub fn true_range() -> TrueRangeRef {
    TrueRangeRef
}

impl IndicatorRef for TrueRangeRef {
    fn key(&self) -> String {
        "true_range".to_string()
    }

    fn required_indicators(&self) -> Vec<(String, Indicator)> {
        vec![(self.key(), Indicator::TrueRange)]
    }

    fn value(&self, ctx: &StrategyContext) -> Option<f64> {
        ctx.indicator(&self.key())
    }

    fn prev_value(&self, ctx: &StrategyContext) -> Option<f64> {
        ctx.indicator_prev(&self.key())
    }
}

// --- Chaikin Oscillator ---

/// Chaikin Oscillator reference.
#[derive(Debug, Clone, Copy)]
pub struct ChaikinOscillatorRef;

/// Create a Chaikin Oscillator reference.
#[inline]
pub fn chaikin_oscillator() -> ChaikinOscillatorRef {
    ChaikinOscillatorRef
}

impl IndicatorRef for ChaikinOscillatorRef {
    fn key(&self) -> String {
        "chaikin_osc".to_string()
    }

    fn required_indicators(&self) -> Vec<(String, Indicator)> {
        vec![(self.key(), Indicator::ChaikinOscillator)]
    }

    fn value(&self, ctx: &StrategyContext) -> Option<f64> {
        ctx.indicator(&self.key())
    }

    fn prev_value(&self, ctx: &StrategyContext) -> Option<f64> {
        ctx.indicator_prev(&self.key())
    }
}

// --- Accumulation/Distribution ---

/// Accumulation/Distribution reference.
#[derive(Debug, Clone, Copy)]
pub struct AccumulationDistributionRef;

/// Create an Accumulation/Distribution reference.
#[inline]
pub fn accumulation_distribution() -> AccumulationDistributionRef {
    AccumulationDistributionRef
}

impl IndicatorRef for AccumulationDistributionRef {
    fn key(&self) -> String {
        "ad".to_string()
    }

    fn required_indicators(&self) -> Vec<(String, Indicator)> {
        vec![(self.key(), Indicator::AccumulationDistribution)]
    }

    fn value(&self, ctx: &StrategyContext) -> Option<f64> {
        ctx.indicator(&self.key())
    }

    fn prev_value(&self, ctx: &StrategyContext) -> Option<f64> {
        ctx.indicator_prev(&self.key())
    }
}

// --- Balance of Power ---

/// Balance of Power reference.
#[derive(Debug, Clone, Copy)]
pub struct BalanceOfPowerRef(pub Option<usize>);

/// Create a Balance of Power reference.
#[inline]
pub fn balance_of_power(period: Option<usize>) -> BalanceOfPowerRef {
    BalanceOfPowerRef(period)
}

impl IndicatorRef for BalanceOfPowerRef {
    fn key(&self) -> String {
        match self.0 {
            Some(p) => format!("bop_{}", p),
            None => "bop".to_string(),
        }
    }

    fn required_indicators(&self) -> Vec<(String, Indicator)> {
        vec![(self.key(), Indicator::BalanceOfPower(self.0))]
    }

    fn value(&self, ctx: &StrategyContext) -> Option<f64> {
        ctx.indicator(&self.key())
    }

    fn prev_value(&self, ctx: &StrategyContext) -> Option<f64> {
        ctx.indicator_prev(&self.key())
    }
}

// --- Bull/Bear Power ---

/// Bull Power reference.
#[derive(Debug, Clone, Copy)]
pub struct BullPowerRef(pub usize);

/// Create a Bull Power reference.
#[inline]
pub fn bull_power(period: usize) -> BullPowerRef {
    BullPowerRef(period)
}

impl IndicatorRef for BullPowerRef {
    fn key(&self) -> String {
        format!("bull_power_{}", self.0)
    }

    fn required_indicators(&self) -> Vec<(String, Indicator)> {
        vec![(self.key(), Indicator::BullBearPower(self.0))]
    }

    fn value(&self, ctx: &StrategyContext) -> Option<f64> {
        ctx.indicator(&self.key())
    }

    fn prev_value(&self, ctx: &StrategyContext) -> Option<f64> {
        ctx.indicator_prev(&self.key())
    }
}

/// Bear Power reference.
#[derive(Debug, Clone, Copy)]
pub struct BearPowerRef(pub usize);

/// Create a Bear Power reference.
#[inline]
pub fn bear_power(period: usize) -> BearPowerRef {
    BearPowerRef(period)
}

impl IndicatorRef for BearPowerRef {
    fn key(&self) -> String {
        format!("bear_power_{}", self.0)
    }

    fn required_indicators(&self) -> Vec<(String, Indicator)> {
        vec![(self.key(), Indicator::BullBearPower(self.0))]
    }

    fn value(&self, ctx: &StrategyContext) -> Option<f64> {
        ctx.indicator(&self.key())
    }

    fn prev_value(&self, ctx: &StrategyContext) -> Option<f64> {
        ctx.indicator_prev(&self.key())
    }
}

// --- Elder Ray ---

/// Elder Ray Bull Power reference.
#[derive(Debug, Clone, Copy)]
pub struct ElderBullPowerRef(pub usize);

/// Create an Elder Ray Bull Power reference.
#[inline]
pub fn elder_bull_power(period: usize) -> ElderBullPowerRef {
    ElderBullPowerRef(period)
}

impl IndicatorRef for ElderBullPowerRef {
    fn key(&self) -> String {
        format!("elder_bull_{}", self.0)
    }

    fn required_indicators(&self) -> Vec<(String, Indicator)> {
        vec![(self.key(), Indicator::ElderRay(self.0))]
    }

    fn value(&self, ctx: &StrategyContext) -> Option<f64> {
        ctx.indicator(&self.key())
    }

    fn prev_value(&self, ctx: &StrategyContext) -> Option<f64> {
        ctx.indicator_prev(&self.key())
    }
}

/// Elder Ray Bear Power reference.
#[derive(Debug, Clone, Copy)]
pub struct ElderBearPowerRef(pub usize);

/// Create an Elder Ray Bear Power reference.
#[inline]
pub fn elder_bear_power(period: usize) -> ElderBearPowerRef {
    ElderBearPowerRef(period)
}

impl IndicatorRef for ElderBearPowerRef {
    fn key(&self) -> String {
        format!("elder_bear_{}", self.0)
    }

    fn required_indicators(&self) -> Vec<(String, Indicator)> {
        vec![(self.key(), Indicator::ElderRay(self.0))]
    }

    fn value(&self, ctx: &StrategyContext) -> Option<f64> {
        ctx.indicator(&self.key())
    }

    fn prev_value(&self, ctx: &StrategyContext) -> Option<f64> {
        ctx.indicator_prev(&self.key())
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
    fn test_macd_keys() {
        let m = macd(12, 26, 9);
        assert_eq!(m.line().key(), "macd_line_12_26_9");
        assert_eq!(m.signal_line().key(), "macd_signal_12_26_9");
        assert_eq!(m.histogram().key(), "macd_histogram_12_26_9");
    }

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
    fn test_supertrend_keys() {
        let st = supertrend(10, 3.0);
        assert_eq!(st.value().key(), "supertrend_value_10_3");
        assert_eq!(st.uptrend().key(), "supertrend_uptrend_10_3");
    }

    #[test]
    fn test_stochastic_keys() {
        let stoch = stochastic(14, 3, 3);
        assert_eq!(stoch.k().key(), "stochastic_k_14_3_3");
        assert_eq!(stoch.d().key(), "stochastic_d_14_3_3");
    }

    #[test]
    fn test_aroon_keys() {
        let ar = aroon(25);
        assert_eq!(ar.up().key(), "aroon_up_25");
        assert_eq!(ar.down().key(), "aroon_down_25");
    }

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

    #[test]
    fn test_keltner_keys() {
        let kc = keltner(20, 2.0, 10);
        assert_eq!(kc.upper().key(), "keltner_upper_20_2_10");
        assert_eq!(kc.middle().key(), "keltner_middle_20_2_10");
        assert_eq!(kc.lower().key(), "keltner_lower_20_2_10");
    }

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

    #[test]
    fn test_power_keys() {
        assert_eq!(bull_power(13).key(), "bull_power_13");
        assert_eq!(bear_power(13).key(), "bear_power_13");
        assert_eq!(elder_bull_power(13).key(), "elder_bull_13");
        assert_eq!(elder_bear_power(13).key(), "elder_bear_13");
    }

    #[test]
    fn test_other_keys() {
        assert_eq!(parabolic_sar(0.02, 0.2).key(), "psar_0.02_0.2");
        assert_eq!(true_range().key(), "true_range");
        assert_eq!(coppock_curve(10, 14, 11).key(), "coppock_10_14_11");
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
