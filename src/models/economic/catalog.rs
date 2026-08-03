//! Discovery models for the macro-economic series universe.
//!
//! Served through the [`Capability::ECONOMIC`](crate::Capability::ECONOMIC)
//! route; FRED is currently the only provider. These answer "which series
//! exist" — [`EconomicSeries`](super::EconomicSeries) answers "what does this
//! series say", and needs an id you already know.

use serde::{Deserialize, Serialize};

/// A series matching a catalog search.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[non_exhaustive]
pub struct EconomicSeriesMatch {
    /// Series identifier (e.g. `"GDPC1"`).
    pub id: String,
    /// Human-readable title.
    pub title: Option<String>,
    /// Reporting frequency (e.g. `"Quarterly"`).
    pub frequency: Option<String>,
    /// Unit of measurement (e.g. `"Billions of Chained 2017 Dollars"`).
    pub units: Option<String>,
    /// Seasonal adjustment description.
    pub seasonal_adjustment: Option<String>,
    /// Date of the earliest observation (`YYYY-MM-DD`).
    pub observation_start: Option<String>,
    /// Date of the latest observation (`YYYY-MM-DD`).
    pub observation_end: Option<String>,
    /// Provider popularity score, useful for ranking equally-relevant matches.
    pub popularity: Option<i64>,
    /// Provider notes about the series.
    pub notes: Option<String>,
}

/// A node in the provider's series category tree.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[non_exhaustive]
pub struct EconomicCategory {
    /// Category identifier.
    pub id: i64,
    /// Category name.
    pub name: Option<String>,
    /// Parent category identifier; the root category is its own parent.
    pub parent_id: Option<i64>,
}

/// A publication that releases economic series on a schedule.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[non_exhaustive]
pub struct EconomicRelease {
    /// Release identifier.
    pub id: i64,
    /// Release name (e.g. `"Employment Situation"`).
    pub name: Option<String>,
    /// Whether the release is accompanied by a press release.
    pub press_release: Option<bool>,
    /// Link to the release on the publisher's site.
    pub link: Option<String>,
}
