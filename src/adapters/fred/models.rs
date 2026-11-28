//! Data models for macro-economic sources (FRED, US Treasury).
//!
//! Re-exports canonical types from `finance_query_models::economic`.

pub use finance_query_models::economic::{
    MacroObservation, MacroObservationFieldsInit, MacroSeries, MacroSeriesFieldsInit,
    TreasuryYield, TreasuryYieldFieldsInit,
};

/// A single scheduled economic-data release date from the FRED
/// `releases/dates` endpoint.
#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct ReleaseDate {
    /// FRED release identifier.
    pub release_id: u64,
    /// Human-readable release name (e.g. "Consumer Price Index").
    pub release_name: String,
    /// Release date as an ISO `YYYY-MM-DD` string.
    pub date: String,
}
