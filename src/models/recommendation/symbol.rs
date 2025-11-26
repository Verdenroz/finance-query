/// Similar Symbol module
///
/// Contains the SimilarSymbol type representing a recommended symbol.
use serde::{Deserialize, Serialize};

/// A similar/recommended symbol with score
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimilarSymbol {
    /// Stock symbol
    pub symbol: String,
    /// Recommendation score (higher = more similar)
    pub score: f64,
}
