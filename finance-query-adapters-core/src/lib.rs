//! Shared plumbing for the `finance-query` provider adapters.
//!
//! Internal crate of `finance-query`. It has no stability guarantee; depend on
//! `finance-query` instead.

pub mod common;
pub mod convert;
pub mod error;
pub mod html;
pub mod keys;
pub mod operation;
pub mod rate_limiter;
pub mod singleton;
pub mod stock_listing;
pub mod xml;
