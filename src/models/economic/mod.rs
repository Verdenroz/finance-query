//! Macro-economic data models.
//!
//! Canonical public types for FRED series and US Treasury yield curve data.

use serde::{Deserialize, Serialize};

#[cfg(feature = "fiscaldata")]
mod auctions;
mod catalog;
#[cfg(feature = "fiscaldata")]
pub use auctions::{TreasuryAuction, TreasuryAuctionQuery, UpcomingAuction};
pub use catalog::{EconomicCategory, EconomicRelease, EconomicSeriesMatch};

/// A provider-agnostic economic data series with metadata.
///
/// Obtain via `Providers::economic` then
/// `.series()`. Supported providers:
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
#[cfg_attr(feature = "dataframe", derive(crate::ToDataFrame))]
#[cfg_attr(feature = "dataframe", dataframe(crate = "crate"))]
#[non_exhaustive]
pub struct MacroObservation {
    /// Date of the observation as `YYYY-MM-DD`
    pub date: String,
    /// Observation value. `None` when FRED reports a missing value (`"."`).
    pub value: Option<f64>,
}

/// A FRED macro-economic time series with all its observations.
///
/// Obtain via `fred::series`.
#[cfg(feature = "fred")]
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
/// Obtain via `fred::treasury_yields`.
#[cfg(feature = "fred")]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "dataframe", derive(crate::ToDataFrame))]
#[cfg_attr(feature = "dataframe", dataframe(crate = "crate"))]
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

#[cfg(feature = "dataframe")]
fn observations_frame(
    series_id: &str,
    observations: &[MacroObservation],
) -> ::polars::prelude::PolarsResult<::polars::prelude::DataFrame> {
    use polars::prelude::*;

    let mut frame = MacroObservation::vec_to_dataframe(observations)?;
    let id = Series::new("series_id".into(), vec![series_id; frame.height()]);
    frame.insert_column(0, id.into())?;
    Ok(frame)
}

#[cfg(feature = "dataframe")]
impl EconomicSeries {
    /// Converts the observations to a polars DataFrame, one row per observation.
    ///
    /// Columns are `series_id`, `date` (strings) and `value` (f64, null for missing values).
    pub fn to_dataframe(&self) -> ::polars::prelude::PolarsResult<::polars::prelude::DataFrame> {
        observations_frame(&self.series_id, &self.observations)
    }
}

#[cfg(all(feature = "dataframe", feature = "fred"))]
impl MacroSeries {
    /// Converts the observations to a polars DataFrame, one row per observation.
    ///
    /// Columns are `series_id`, `date` (strings) and `value` (f64, null for missing values).
    pub fn to_dataframe(&self) -> ::polars::prelude::PolarsResult<::polars::prelude::DataFrame> {
        observations_frame(&self.id, &self.observations)
    }
}

#[cfg(all(test, feature = "dataframe"))]
mod dataframe_tests;
