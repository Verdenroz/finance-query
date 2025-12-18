/// Similar Symbol module
///
/// Contains the SimilarSymbol type representing a recommended symbol.
use serde::{Deserialize, Serialize};

/// A similar/recommended symbol with score
///
/// Note: This struct cannot be manually constructed - obtain via `Ticker::recommendations()`.
#[non_exhaustive]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimilarSymbol {
    /// Stock symbol
    pub symbol: String,
    /// Recommendation score (higher = more similar)
    pub score: f64,
}
