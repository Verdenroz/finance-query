/// Similar Symbol module
///
/// Contains the SimilarSymbol type representing a recommended symbol.
use serde::{Deserialize, Serialize};

#[cfg(feature = "python")]
use finance_query_derive::PyModel;

/// A similar/recommended symbol with score
///
/// Note: This struct cannot be manually constructed - obtain via `Ticker::recommendations()`.
#[non_exhaustive]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "dataframe", derive(crate::ToDataFrame))]
#[cfg_attr(feature = "python", derive(PyModel))]
#[cfg_attr(feature = "python", py_model(dataframe = "columns"))]
pub struct SimilarSymbol {
    /// Stock symbol
    pub symbol: String,
    /// Recommendation score (higher = more similar)
    pub score: f64,
}
