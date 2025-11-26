use super::RecommendationResult;
/// Recommendation Response module
///
/// Handles parsing of Yahoo Finance recommendation API responses.
use serde::{Deserialize, Serialize};

/// Response wrapper for recommendations endpoint
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecommendationResponse {
    /// Finance container
    pub finance: RecommendationFinance,
}

/// Finance container for recommendations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecommendationFinance {
    /// Recommendation results
    pub result: Vec<RecommendationResult>,
    /// Error if any
    pub error: Option<serde_json::Value>,
}

impl RecommendationResponse {
    /// Parse from JSON value
    pub fn from_json(value: serde_json::Value) -> Result<Self, serde_json::Error> {
        serde_json::from_value(value)
    }

    /// Get the list of recommended symbols
    pub fn symbols(&self) -> Vec<&str> {
        self.finance
            .result
            .first()
            .map(|r| {
                r.recommended_symbols
                    .iter()
                    .map(|s| s.symbol.as_str())
                    .collect()
            })
            .unwrap_or_default()
    }
}
