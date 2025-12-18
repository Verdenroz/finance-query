use super::symbol::SimilarSymbol;
/// Recommendation Result module
///
/// Contains the RecommendationResult type and conversion methods.
/// This type is internal implementation detail and not exposed in the public API.
use serde::{Deserialize, Serialize};

/// Recommendation result for a symbol
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct RecommendationResult {
    /// Symbol that was queried
    pub symbol: String,
    /// Recommended symbols
    pub recommended_symbols: Vec<SimilarSymbol>,
}
