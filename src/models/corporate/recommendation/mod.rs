//! Recommendation models.
//!
//! Contains all data structures and types for Yahoo Finance's recommendation/similar endpoint.

mod data;
pub mod response;
pub mod result;
mod symbol;

pub use data::Recommendation;
pub use symbol::SimilarSymbol;
