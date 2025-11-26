use super::SimilarSymbol;
/// Recommendation aggregate module
///
/// Contains the fully typed Recommendation structure for similar/recommended symbols.
use serde::{Deserialize, Serialize};

/// Fully typed recommendation data
///
/// Aggregates the queried symbol and its recommendations into a single
/// convenient structure. This is the recommended type for serialization
/// and API responses.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Recommendation {
    /// Symbol that was queried
    pub symbol: String,

    /// Recommended/similar symbols with scores
    pub recommendations: Vec<SimilarSymbol>,
}

impl Recommendation {
    /// Get just the symbol strings
    pub fn symbols(&self) -> Vec<&str> {
        self.recommendations
            .iter()
            .map(|s| s.symbol.as_str())
            .collect()
    }

    /// Get the number of recommendations
    pub fn count(&self) -> usize {
        self.recommendations.len()
    }
}
