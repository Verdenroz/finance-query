//! Timeseries Module
//!
//! Contains all data structures for Yahoo Finance's fundamentals timeseries endpoint.
//! Provides historical financial data (revenue, income, assets, etc.)

mod data_point;
pub mod fundamental_types;
mod meta;
mod response;

pub use data_point::{ReportedValue, TimeseriesDataPoint};
pub use meta::TimeseriesMeta;
pub use response::{TimeseriesContainer, TimeseriesResponse, TimeseriesResult};
