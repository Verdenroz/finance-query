//! Screener models.
//!
//! Types for Yahoo Finance predefined screener data (most actives, day gainers,
//! day losers, most shorted, growth stocks, undervalued stocks, and more).
//!
//! ## Predefined Screeners
//!
//! The API returns a clean, flattened `ScreenerResults`:
//! ```json
//! {
//!   "quotes": [...],
//!   "type": "most_actives",
//!   "description": "Stocks ordered in descending order by intraday trade volume",
//!   "lastUpdated": 1234567890
//! }
//! ```
//!
//! ## Custom Screeners
//!
//! Use `ScreenerQuery` to build custom filters:
//! ```no_run
//! use finance_query::{ScreenerQuery, QueryCondition, screener_query::Operator};
//!
//! let query = ScreenerQuery::new()
//!     .size(25)
//!     .add_condition(QueryCondition::new("region", Operator::Eq).value_str("us"))
//!     .add_condition(QueryCondition::new("avgdailyvol3m", Operator::Gt).value(200000));
//! ```

mod query;
mod quote;
mod response;

pub use query::{QueryCondition, QueryGroup, QueryOperand, QueryValue, ScreenerQuery};
pub use quote::ScreenerQuote;
pub use response::ScreenerResults;
