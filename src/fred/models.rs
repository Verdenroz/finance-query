//! Data models for macro-economic sources (FRED, US Treasury).

use serde::{Deserialize, Serialize};

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
/// Obtain via [`super::series`].
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
/// Obtain via [`super::treasury_yields`].
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
