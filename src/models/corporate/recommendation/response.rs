use super::result::RecommendationResult;
/// Recommendation Response module
///
/// Handles parsing of Yahoo Finance recommendation API responses.
/// These types are internal implementation details and not exposed in the public API.
use serde::{Deserialize, Serialize};

/// Response wrapper for recommendations endpoint
#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct RecommendationResponse {
    /// Finance container
    pub finance: RecommendationFinance,
}

/// Finance container for recommendations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct RecommendationFinance {
    /// Recommendation results
    pub result: Vec<RecommendationResult>,
    /// Error if any
    pub error: Option<serde_json::Value>,
}

impl RecommendationResponse {
    /// Parse from JSON value
    pub(crate) fn from_json(value: serde_json::Value) -> Result<Self, serde_json::Error> {
        serde_json::from_value(value)
    }
}
