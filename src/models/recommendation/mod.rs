//! Recommendation Module
//!
//! Contains all data structures and types for Yahoo Finance's recommendation/similar endpoint.
mod data;
mod response;
mod result;
mod symbol;

pub use data::Recommendation;
pub use response::{RecommendationFinance, RecommendationResponse};
pub use result::RecommendationResult;
pub use symbol::SimilarSymbol;
