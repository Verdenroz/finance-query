//! Currency models.
//!
//! Contains data structures for Yahoo Finance's currencies endpoint.

mod response;

pub use response::Currency;

#[cfg(feature = "python")]
pub use response::PyCurrency;
