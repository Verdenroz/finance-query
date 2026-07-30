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

mod ichimoku;
mod moving_averages;
mod oscillators;
mod power;
mod trend;
mod volatility;
mod volume;

pub use ichimoku::*;
pub use moving_averages::*;
pub use oscillators::*;
pub use power::*;
pub use trend::*;
pub use volatility::*;
pub use volume::*;
