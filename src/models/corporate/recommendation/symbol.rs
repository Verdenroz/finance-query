/// Similar Symbol module
///
/// Contains the SimilarSymbol type representing a recommended symbol.
use serde::{Deserialize, Serialize};

/// A similar/recommended symbol with score
///
/// Obtain via `Ticker::recommendations()`, or build one with `init!`.
#[non_exhaustive]
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[cfg_attr(feature = "dataframe", derive(crate::ToDataFrame))]
#[cfg_attr(feature = "dataframe", dataframe(crate = "crate"))]
pub struct SimilarSymbol {
    /// Stock symbol
    pub symbol: String,
    /// Recommendation score (higher = more similar)
    pub score: f64,
}
