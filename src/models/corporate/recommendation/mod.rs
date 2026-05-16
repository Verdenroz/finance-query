//! Recommendation models.
//!
//! Contains all data structures and types for Yahoo Finance's recommendation/similar endpoint.

mod data;
pub(crate) mod response;
pub(crate) mod result;
mod symbol;

pub use data::Recommendation;
pub use symbol::SimilarSymbol;
