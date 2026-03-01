//! Technical analysis indicators for financial data.
//!
//! This module provides common technical indicators used by traders and analysts.
//! All indicators work with time series price data from OHLCV candles.
//!
//! # Available Indicators
//!
//! ## Moving Averages
//! - [`sma`] - Simple Moving Average
//! - [`ema`] - Exponential Moving Average
//!
//! ## Momentum Oscillators
//! - [`rsi`] - Relative Strength Index
//!
//! ## Trend Indicators
//! - [`macd`] - Moving Average Convergence Divergence
//!
//! ## Volatility Indicators
//! - [`bollinger_bands`] - Bollinger Bands
//! - [`atr`] - Average True Range
//!
//! # Example
//!
//! ```no_run
//! use finance_query::{Ticker, Interval, TimeRange};
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let ticker = Ticker::new("AAPL").await?;
//! let chart = ticker.chart(Interval::OneDay, TimeRange::ThreeMonths).await?;
//!
//! // Use Chart extension methods (requires "indicators" feature)
//! let sma_20 = chart.sma(20);
//! let rsi_14 = chart.rsi(14)?;
//! let atr_14 = chart.atr(14)?;
//!
//! // Or call indicators directly
//! let closes: Vec<f64> = chart.candles.iter().map(|c| c.close).collect();
//! let ema_12 = finance_query::indicators::ema(&closes, 12);
//! # Ok(())
//! # }
//! ```

mod accumulation_distribution;
mod adx;
mod alma;
mod aroon;
mod atr;
mod awesome_oscillator;
mod balance_of_power;
mod bollinger;
mod bull_bear_power;
mod cci;
mod chaikin_oscillator;
mod choppiness_index;
mod cmf;
mod cmo;
mod coppock_curve;
mod dema;
mod donchian_channels;
mod elder_ray;
mod ema;
mod hma;
mod ichimoku;
mod keltner_channels;
mod macd;
mod mcginley_dynamic;
mod mfi;
mod momentum;
mod obv;
mod parabolic_sar;
mod patterns;
mod roc;
mod rsi;
mod sma;
mod stochastic;
mod stochastic_rsi;
mod supertrend;
mod tema;
mod true_range;
mod vwap;
mod vwma;
mod williams_r;
mod wma;

// Summary module for batch indicator calculations
pub mod summary;

// Re-export all indicators and patterns
pub use accumulation_distribution::accumulation_distribution;
pub use adx::adx;
pub use alma::alma;
pub use aroon::{AroonResult, aroon};
pub use atr::atr;
pub use awesome_oscillator::awesome_oscillator;
pub use balance_of_power::balance_of_power;
pub use bollinger::{BollingerBands, bollinger_bands};
pub use bull_bear_power::{BullBearPowerResult, bull_bear_power};
pub use cci::cci;
pub use chaikin_oscillator::chaikin_oscillator;
pub use choppiness_index::choppiness_index;
pub use cmf::cmf;
pub use cmo::cmo;
pub use coppock_curve::coppock_curve;
pub use dema::dema;
pub use donchian_channels::{DonchianChannelsResult, donchian_channels};
pub use elder_ray::{ElderRayResult, elder_ray};
pub use ema::ema;
pub use hma::hma;
pub use ichimoku::{IchimokuResult, ichimoku};
pub use keltner_channels::{KeltnerChannelsResult, keltner_channels};
pub use macd::{MacdResult, macd};
pub use mcginley_dynamic::mcginley_dynamic;
pub use mfi::mfi;
pub use momentum::momentum;
pub use obv::obv;
pub use parabolic_sar::parabolic_sar;
pub use patterns::{CandlePattern, PatternSentiment, patterns};
pub use roc::roc;
pub use rsi::rsi;
pub use sma::sma;
pub use stochastic::{StochasticResult, stochastic};
pub use stochastic_rsi::stochastic_rsi;
pub use supertrend::{SuperTrendResult, supertrend};
pub use tema::tema;
pub use true_range::true_range;
pub use vwap::vwap;
pub use vwma::vwma;
pub use williams_r::williams_r;
pub use wma::wma;

// Re-export summary types
pub use summary::{
    AroonData, BollingerBandsData, BullBearPowerData, DonchianChannelsData, ElderRayData,
    IchimokuData, IndicatorsSummary, KeltnerChannelsData, MacdData, StochasticData, SuperTrendData,
};

// Re-export Indicator enum for easy access
pub use Indicator as IndicatorType;

/// Error type for indicator calculations
#[derive(Debug, thiserror::Error)]
pub enum IndicatorError {
    /// Not enough data points to calculate the indicator
    #[error("Insufficient data: need at least {need} data points, got {got}")]
    InsufficientData {
        /// Minimum number of data points required
        need: usize,
        /// Actual number of data points provided
        got: usize,
    },

    /// Invalid period parameter provided
    #[error("Invalid period: {0}")]
    InvalidPeriod(String),
}

/// Result type for indicator calculations
pub type Result<T> = std::result::Result<T, IndicatorError>;

/// Result of an indicator calculation
///
/// Different indicators return different types of data:
/// - Simple indicators (SMA, EMA, RSI, ATR) return a time series of values
/// - Complex indicators (MACD, Bollinger Bands) return multiple series
#[derive(Debug, Clone, PartialEq)]
#[non_exhaustive]
pub enum IndicatorResult {
    /// Single value time series (SMA, EMA, RSI, ATR, OBV, VWAP)
    Series(Vec<Option<f64>>),
    /// MACD result with three series
    Macd(MacdResult),
    /// Bollinger Bands with upper, middle, lower bands
    Bollinger(BollingerBands),
    /// Stochastic Oscillator result
    Stochastic(StochasticResult),
    /// Aroon result
    Aroon(AroonResult),
    /// SuperTrend result
    SuperTrend(SuperTrendResult),
    /// Ichimoku Cloud result
    Ichimoku(IchimokuResult),
    /// Bull/Bear Power result
    BullBearPower(BullBearPowerResult),
    /// Elder Ray Index result
    ElderRay(ElderRayResult),
    /// Keltner Channels result
    Keltner(KeltnerChannelsResult),
    /// Donchian Channels result
    Donchian(DonchianChannelsResult),
}

/// Enum representing all available technical indicators.
///
/// This enum is used with `Ticker::indicator()` to calculate specific indicators
/// over a given interval and time range.
#[derive(Debug, Clone, Copy, PartialEq)]
#[non_exhaustive]
pub enum Indicator {
    /// Simple Moving Average with custom period
    Sma(usize),
    /// Exponential Moving Average with custom period
    Ema(usize),
    /// Relative Strength Index with custom period
    Rsi(usize),
    /// Moving Average Convergence Divergence (fast, slow, signal periods)
    Macd {
        /// Fast EMA period
        fast: usize,
        /// Slow EMA period
        slow: usize,
        /// Signal line EMA period
        signal: usize,
    },
    /// Bollinger Bands (period, standard deviation multiplier)
    Bollinger {
        /// SMA period
        period: usize,
        /// Standard deviation multiplier
        std_dev: f64,
    },
    /// Average True Range with custom period
    Atr(usize),
    /// On-Balance Volume
    Obv,
    /// Volume Weighted Average Price
    Vwap,
    /// Weighted Moving Average
    Wma(usize),
    /// Double Exponential Moving Average
    Dema(usize),
    /// Triple Exponential Moving Average
    Tema(usize),
    /// Hull Moving Average
    Hma(usize),
    /// Volume Weighted Moving Average
    Vwma(usize),
    /// Arnaud Legoux Moving Average
    Alma {
        /// Window period
        period: usize,
        /// Offset (typically 0.85)
        offset: f64,
        /// Sigma (typically 6.0)
        sigma: f64,
    },
    /// McGinley Dynamic
    McginleyDynamic(usize),
    /// Stochastic Oscillator
    Stochastic {
        /// %K period
        k_period: usize,
        /// %K slowing period
        k_slow: usize,
        /// %D period (SMA of %K)
        d_period: usize,
    },
    /// Stochastic RSI
    StochasticRsi {
        /// RSI period
        rsi_period: usize,
        /// Stochastic period applied to RSI
        stoch_period: usize,
        /// %K smoothing period
        k_period: usize,
        /// %D smoothing period
        d_period: usize,
    },
    /// Commodity Channel Index
    Cci(usize),
    /// Williams %R
    WilliamsR(usize),
    /// Rate of Change
    Roc(usize),
    /// Momentum
    Momentum(usize),
    /// Chande Momentum Oscillator
    Cmo(usize),
    /// Awesome Oscillator
    AwesomeOscillator {
        /// Fast SMA period
        fast: usize,
        /// Slow SMA period
        slow: usize,
    },
    /// Coppock Curve
    CoppockCurve {
        /// WMA smoothing period
        wma_period: usize,
        /// Long ROC period
        long_roc: usize,
        /// Short ROC period
        short_roc: usize,
    },
    /// Average Directional Index
    Adx(usize),
    /// Aroon
    Aroon(usize),
    /// SuperTrend
    Supertrend {
        /// ATR period
        period: usize,
        /// ATR multiplier
        multiplier: f64,
    },
    /// Ichimoku Cloud
    Ichimoku {
        /// Conversion line period (Tenkan-sen)
        conversion: usize,
        /// Base line period (Kijun-sen)
        base: usize,
        /// Lagging span period (Chikou Span)
        lagging: usize,
        /// Displacement for leading spans
        displacement: usize,
    },
    /// Parabolic SAR
    ParabolicSar {
        /// Acceleration factor step
        step: f64,
        /// Maximum acceleration factor
        max: f64,
    },
    /// Bull/Bear Power
    BullBearPower(usize),
    /// Elder Ray Index
    ElderRay(usize),
    /// Keltner Channels
    KeltnerChannels {
        /// EMA period for middle line
        period: usize,
        /// ATR multiplier for bands
        multiplier: f64,
        /// ATR period
        atr_period: usize,
    },
    /// Donchian Channels
    DonchianChannels(usize),
    /// True Range
    TrueRange,
    /// Choppiness Index
    ChoppinessIndex(usize),
    /// Money Flow Index
    Mfi(usize),
    /// Chaikin Money Flow
    Cmf(usize),
    /// Chaikin Oscillator
    ChaikinOscillator,
    /// Accumulation/Distribution
    AccumulationDistribution,
    /// Balance of Power
    BalanceOfPower(Option<usize>),
}

impl Indicator {
    /// Get a default instance with standard parameters
    ///
    /// # Examples
    ///
    /// ```
    /// use finance_query::indicators::Indicator;
    ///
    /// let rsi = Indicator::Rsi(14);  // 14-period RSI
    /// let macd = Indicator::Macd { fast: 12, slow: 26, signal: 9 };
    /// ```
    pub fn with_defaults(self) -> Self {
        match self {
            Indicator::Sma(_) => Indicator::Sma(20),
            Indicator::Ema(_) => Indicator::Ema(12),
            Indicator::Rsi(_) => Indicator::Rsi(14),
            Indicator::Macd { .. } => Indicator::Macd {
                fast: 12,
                slow: 26,
                signal: 9,
            },
            Indicator::Bollinger { .. } => Indicator::Bollinger {
                period: 20,
                std_dev: 2.0,
            },
            Indicator::Atr(_) => Indicator::Atr(14),
            ind => ind,
        }
    }

    /// Get the human-readable name of the indicator
    pub fn name(&self) -> &'static str {
        match self {
            Indicator::Sma(_) => "Simple Moving Average",
            Indicator::Ema(_) => "Exponential Moving Average",
            Indicator::Rsi(_) => "Relative Strength Index",
            Indicator::Macd { .. } => "MACD",
            Indicator::Bollinger { .. } => "Bollinger Bands",
            Indicator::Atr(_) => "Average True Range",
            Indicator::Obv => "On-Balance Volume",
            Indicator::Vwap => "VWAP",
            Indicator::Wma(_) => "Weighted Moving Average",
            Indicator::Dema(_) => "Double Exponential Moving Average",
            Indicator::Tema(_) => "Triple Exponential Moving Average",
            Indicator::Hma(_) => "Hull Moving Average",
            Indicator::Vwma(_) => "Volume Weighted Moving Average",
            Indicator::Alma { .. } => "Arnaud Legoux Moving Average",
            Indicator::McginleyDynamic(_) => "McGinley Dynamic",
            Indicator::Stochastic { .. } => "Stochastic Oscillator",
            Indicator::StochasticRsi { .. } => "Stochastic RSI",
            Indicator::Cci(_) => "Commodity Channel Index",
            Indicator::WilliamsR(_) => "Williams %R",
            Indicator::Roc(_) => "Rate of Change",
            Indicator::Momentum(_) => "Momentum",
            Indicator::Cmo(_) => "Chande Momentum Oscillator",
            Indicator::AwesomeOscillator { .. } => "Awesome Oscillator",
            Indicator::CoppockCurve { .. } => "Coppock Curve",
            Indicator::Adx(_) => "Average Directional Index",
            Indicator::Aroon(_) => "Aroon",
            Indicator::Supertrend { .. } => "SuperTrend",
            Indicator::Ichimoku { .. } => "Ichimoku Cloud",
            Indicator::ParabolicSar { .. } => "Parabolic SAR",
            Indicator::BullBearPower(_) => "Bull/Bear Power",
            Indicator::ElderRay(_) => "Elder Ray Index",
            Indicator::KeltnerChannels { .. } => "Keltner Channels",
            Indicator::DonchianChannels(_) => "Donchian Channels",
            Indicator::TrueRange => "True Range",
            Indicator::ChoppinessIndex(_) => "Choppiness Index",
            Indicator::Mfi(_) => "Money Flow Index",
            Indicator::Cmf(_) => "Chaikin Money Flow",
            Indicator::ChaikinOscillator => "Chaikin Oscillator",
            Indicator::AccumulationDistribution => "Accumulation/Distribution",
            Indicator::BalanceOfPower(_) => "Balance of Power",
        }
    }

    /// Minimum number of data bars required before this indicator produces
    /// meaningful output.
    ///
    /// Used by the backtesting engine's `CustomStrategy` to automatically
    /// compute the warmup period instead of parsing key-name suffixes.
    ///
    /// # Examples
    ///
    /// ```
    /// use finance_query::indicators::Indicator;
    ///
    /// assert_eq!(Indicator::Sma(20).warmup_bars(), 20);
    /// assert_eq!(Indicator::Macd { fast: 12, slow: 26, signal: 9 }.warmup_bars(), 35);
    /// assert_eq!(Indicator::Bollinger { period: 20, std_dev: 2.0 }.warmup_bars(), 20);
    /// ```
    pub fn warmup_bars(&self) -> usize {
        match self {
            Self::Sma(p)
            | Self::Ema(p)
            | Self::Rsi(p)
            | Self::Atr(p)
            | Self::Wma(p)
            | Self::Dema(p)
            | Self::Tema(p)
            | Self::Hma(p)
            | Self::Vwma(p)
            | Self::McginleyDynamic(p)
            | Self::Cci(p)
            | Self::WilliamsR(p)
            | Self::Roc(p)
            | Self::Momentum(p)
            | Self::Cmo(p)
            | Self::Adx(p)
            | Self::Aroon(p)
            | Self::DonchianChannels(p)
            | Self::ChoppinessIndex(p)
            | Self::Mfi(p)
            | Self::Cmf(p)
            | Self::BullBearPower(p)
            | Self::ElderRay(p) => *p,
            Self::Macd { fast, slow, signal } => *slow.max(fast) + signal,
            Self::Bollinger { period, .. } => *period,
            Self::Alma { period, .. } => *period,
            Self::Stochastic {
                k_period,
                k_slow,
                d_period,
            } => k_period + k_slow + d_period,
            Self::StochasticRsi {
                rsi_period,
                stoch_period,
                k_period,
                d_period,
            } => rsi_period + stoch_period + k_period.max(d_period),
            Self::AwesomeOscillator { slow, .. } => *slow,
            Self::CoppockCurve {
                long_roc,
                wma_period,
                ..
            } => long_roc + wma_period,
            Self::Ichimoku {
                base, displacement, ..
            } => base + displacement,
            Self::Supertrend { period, .. } => *period,
            Self::ParabolicSar { .. } => 2,
            Self::KeltnerChannels {
                period, atr_period, ..
            } => *period.max(atr_period),
            Self::BalanceOfPower(Some(p)) => *p,
            // Volume/price indicators with no meaningful lookback
            Self::Obv
            | Self::Vwap
            | Self::TrueRange
            | Self::ChaikinOscillator
            | Self::AccumulationDistribution
            | Self::BalanceOfPower(None) => 1,
        }
    }
}

/// Helper function to extract the last non-None value from a vector.
///
/// Useful for converting historical indicator values to latest value only.
///
/// # Example
///
/// ```
/// use finance_query::indicators::last_value;
///
/// let values = vec![None, None, Some(10.0), Some(20.0)];
/// assert_eq!(last_value(&values), Some(20.0));
/// ```
pub fn last_value(values: &[Option<f64>]) -> Option<f64> {
    values.iter().rev().find_map(|&v| v)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_last_value() {
        assert_eq!(last_value(&[None, None, Some(1.0), Some(2.0)]), Some(2.0));
        assert_eq!(last_value(&[None, None, Some(1.0), None]), Some(1.0));
        assert_eq!(last_value(&[None, None, None]), None);
        assert_eq!(last_value(&[]), None);
    }
}
