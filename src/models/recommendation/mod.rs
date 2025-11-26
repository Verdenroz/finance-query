///! Recommendation Module
///!
///! Contains all data structures and types for Yahoo Finance's recommendation/similar endpoint.
mod recommendation;
mod response;
mod result;
mod symbol;

pub use recommendation::Recommendation;
pub use response::{RecommendationFinance, RecommendationResponse};
pub use result::RecommendationResult;
pub use symbol::SimilarSymbol;

// Backwards compatibility - keep RecommendedSymbol as alias
pub use symbol::SimilarSymbol as RecommendedSymbol;
