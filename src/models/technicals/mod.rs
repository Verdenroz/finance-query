//! Technical analysis indicator models.
//!
//! Canonical public types for technical indicators (SMA, EMA, RSI, MACD, etc.),
//! shared across Polygon, FMP, and Alpha Vantage providers.
//!
//! Most types are scaffolding for upcoming provider implementations.
#![allow(dead_code)]

use serde::{Deserialize, Serialize};

/// The type of technical indicator.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub enum IndicatorType {
    /// Simple Moving Average
    Sma,
    /// Exponential Moving Average
    Ema,
    /// Weighted Moving Average
    Wma,
    /// Double Exponential Moving Average
    Dema,
    /// Triple Exponential Moving Average
    Tema,
    /// Relative Strength Index
    Rsi,
    /// Moving Average Convergence Divergence
    Macd,
    /// Average Directional Index
    Adx,
    /// Williams %R
    WilliamsR,
    /// Bollinger Bands
    Bollinger,
    /// Stochastic Oscillator
    Stochastic,
}

impl IndicatorType {
    /// Human-readable indicator name.
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Sma => "SMA",
            Self::Ema => "EMA",
            Self::Wma => "WMA",
            Self::Dema => "DEMA",
            Self::Tema => "TEMA",
            Self::Rsi => "RSI",
            Self::Macd => "MACD",
            Self::Adx => "ADX",
            Self::WilliamsR => "Williams %R",
            Self::Bollinger => "Bollinger Bands",
            Self::Stochastic => "Stochastic",
        }
    }
}

/// A single data point from a technical indicator calculation.
///
/// Obtain via [`Ticker::indicator`](crate::Ticker::indicator) (future).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct IndicatorValue {
    /// Date of the data point as YYYY-MM-DD
    pub date: String,
    /// The primary indicator value (e.g., SMA value, RSI reading)
    pub value: Option<f64>,
    /// Secondary/signal line value (e.g., MACD signal line)
    pub signal: Option<f64>,
    /// Histogram value (e.g., MACD histogram)
    pub histogram: Option<f64>,
    /// Upper band (e.g., Bollinger upper, stochastic overbought)
    pub upper_band: Option<f64>,
    /// Lower band (e.g., Bollinger lower, stochastic oversold)
    pub lower_band: Option<f64>,
}
