//! Market Summary models.
//!
//! Contains data structures for Yahoo Finance's market summary endpoint.

mod response;

pub use response::{MarketSummaryQuote, SparkData};

#[cfg(feature = "python")]
pub use response::{PyMarketSummaryQuote, PySparkData};
