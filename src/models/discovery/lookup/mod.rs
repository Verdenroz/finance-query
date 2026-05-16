//! Lookup models.
//!
//! Contains all data structures for Yahoo Finance's lookup endpoint.
//! Provides type-filtered symbol discovery (equity, ETF, index, etc.).

mod quote;
mod response;

pub use quote::LookupQuote;
pub use response::LookupResults;
