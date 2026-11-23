//! Shared data types and constants for finance-query.
//!
//! This crate backs the `finance-query` facade, which re-exports every public
//! item at its documented path. Depend on `finance-query` instead.

mod capability;
mod chart;
pub mod constants;
#[cfg(feature = "dataframe")]
mod dataframe;
#[cfg(feature = "indicators")]
pub mod indicators;
#[doc(hidden)]
pub mod perf_metrics;
mod provider;

pub use capability::Capability;
pub use chart::{Candle, Dividend};
pub use provider::{CustomId, Provider};

#[cfg(feature = "dataframe")]
pub use dataframe::ToDataFrame;
#[cfg(feature = "dataframe")]
#[doc(hidden)]
pub use dataframe::derive as __dataframe;
#[cfg(feature = "dataframe")]
pub use finance_query_derive::ToDataFrame;
#[cfg(feature = "dataframe")]
#[doc(hidden)]
pub use polars;
