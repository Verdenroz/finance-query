//! Macro-economic data models.
//!
//! Canonical public types for FRED series and US Treasury yield curve data.

use serde::{Deserialize, Serialize};

/// A provider-agnostic economic data series with metadata.
///
/// Obtain via [`Ticker::economic_series`](crate::Ticker::economic_series). Supported providers:
/// Alpha Vantage, Polygon, FRED.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct EconomicSeries {
    /// Series identifier (e.g., `"REAL_GDP"`, `"FEDFUNDS"`, `"inflation"`)
    pub series_id: String,
    /// Human-readable series title
    pub title: Option<String>,
    /// Unit of measurement (e.g., `"Billions of Dollars"`, `"Percent"`)
    pub units: Option<String>,
    /// Reporting frequency (e.g., `"Annual"`, `"Monthly"`)
    pub frequency: Option<String>,
    /// Chronologically ordered observations
    pub observations: Vec<MacroObservation>,
}

/// A single observation in a FRED data series.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct MacroObservation {
    /// Date of the observation as `YYYY-MM-DD`
    pub date: String,
    /// Observation value. `None` when FRED reports a missing value (`"."`).
    pub value: Option<f64>,
}

/// A FRED macro-economic time series with all its observations.
///
/// Obtain via [`fred::series`](crate::fred::series).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct MacroSeries {
    /// FRED series ID (e.g., `"FEDFUNDS"`, `"CPIAUCSL"`, `"DGS10"`)
    pub id: String,
    /// Chronologically ordered observations
    pub observations: Vec<MacroObservation>,
}

/// One day of US Treasury yield curve rates.
///
/// Maturities with no published rate on a given date are `None`.
/// Obtain via [`fred::treasury_yields`](crate::fred::treasury_yields).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct TreasuryYield {
    /// Date as `MM/DD/YYYY` (Treasury's native format)
    pub date: String,
    /// 1-month Treasury yield (%)
    pub y1m: Option<f64>,
    /// 2-month Treasury yield (%)
    pub y2m: Option<f64>,
    /// 3-month Treasury yield (%)
    pub y3m: Option<f64>,
    /// 4-month Treasury yield (%)
    pub y4m: Option<f64>,
    /// 6-month Treasury yield (%)
    pub y6m: Option<f64>,
    /// 1-year Treasury yield (%)
    pub y1: Option<f64>,
    /// 2-year Treasury yield (%)
    pub y2: Option<f64>,
    /// 3-year Treasury yield (%)
    pub y3: Option<f64>,
    /// 5-year Treasury yield (%)
    pub y5: Option<f64>,
    /// 7-year Treasury yield (%)
    pub y7: Option<f64>,
    /// 10-year Treasury yield (%)
    pub y10: Option<f64>,
    /// 20-year Treasury yield (%)
    pub y20: Option<f64>,
    /// 30-year Treasury yield (%)
    pub y30: Option<f64>,
}
