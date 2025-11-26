use super::{Recommendation, SimilarSymbol};
/// Recommendation Result module
///
/// Contains the RecommendationResult type and conversion methods.
use serde::{Deserialize, Serialize};

/// Recommendation result for a symbol
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RecommendationResult {
    /// Symbol that was queried
    pub symbol: String,
    /// Recommended symbols
    pub recommended_symbols: Vec<SimilarSymbol>,
}

impl RecommendationResult {
    /// Converts this recommendation result into a Recommendation aggregate
    ///
    /// Extracts the symbol and recommendations into a clean, serializable structure.
    pub fn to_recommendation(&self) -> Recommendation {
        Recommendation {
            symbol: self.symbol.clone(),
            recommendations: self.recommended_symbols.clone(),
        }
    }
}
