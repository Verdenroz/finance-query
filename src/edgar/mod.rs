//! SEC EDGAR API client.
//!
//! Provides access to SEC EDGAR data including filing history,
//! structured XBRL financial data, and full-text search.
//!
//! All requests are rate-limited to 10 per second as required by SEC.

pub(crate) mod client;
mod rate_limiter;

pub use client::{EdgarClient, EdgarClientBuilder};
