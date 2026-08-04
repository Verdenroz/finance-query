//! BLS Public Data API wire types.
//!
//! v1 and v2 share the same response envelope; v2 additionally returns a
//! `catalog` block per series when `catalog: true` is requested.

use serde::Deserialize;

/// The envelope every BLS response uses, successful or not.
#[derive(Debug, Clone, Deserialize)]
pub(crate) struct BlsResponse {
    pub status: String,
    /// Diagnostics. Populated even on `REQUEST_SUCCEEDED` — an unknown series
    /// id comes back "succeeded" with the complaint in here and no data.
    #[serde(default)]
    pub message: Vec<String>,
    #[serde(default, rename = "Results")]
    pub results: Option<BlsResults>,
}

#[derive(Debug, Clone, Deserialize)]
pub(crate) struct BlsResults {
    #[serde(default)]
    pub series: Vec<BlsSeries>,
}

#[derive(Debug, Clone, Deserialize)]
pub(crate) struct BlsSeries {
    /// Only present when `catalog: true` was requested (v2 with a key).
    #[serde(default)]
    pub catalog: Option<BlsCatalog>,
    #[serde(default)]
    pub data: Vec<BlsDataPoint>,
}

/// Series metadata, available on the keyed v2 route only.
#[derive(Debug, Clone, Deserialize)]
pub(crate) struct BlsCatalog {
    #[serde(default)]
    pub series_title: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub(crate) struct BlsDataPoint {
    pub year: String,
    /// `M01`–`M13`, `Q01`–`Q05`, `S01`–`S03`, or `A01`.
    pub period: String,
    /// Numeric string; `"-"` marks a figure BLS could not publish.
    pub value: String,
}
