//! Chart models.
//!
//! Contains all data structures and types for Yahoo Finance's chart endpoint.

#[cfg(feature = "indicators")]
pub mod compute;
mod stock_bars;
pub use stock_bars::{PriceAdjustment, StockBar};
mod data;
pub mod dividend_analytics;
pub mod events;
pub use events::ChartEvents;
pub mod indicators;
mod meta;
pub mod response;
pub mod result;
/// Spark / sparkline submodule.
pub mod spark;

pub use data::Chart;
pub use dividend_analytics::DividendAnalytics;
pub use events::{CapitalGain, Dividend, Split};
pub use finance_query_types::Candle;
pub use meta::ChartMeta;

#[cfg(all(test, feature = "dataframe"))]
mod dataframe_tests;
