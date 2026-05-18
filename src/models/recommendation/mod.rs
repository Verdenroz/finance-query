//! Recommendation models.
//!
//! Contains all data structures and types for Yahoo Finance's recommendation/similar endpoint.

mod data;
pub(crate) mod response;
pub(crate) mod result;
mod symbol;

pub use data::Recommendation;
#[cfg(feature = "python")]
pub use data::PyRecommendation;
pub use symbol::SimilarSymbol;
