//! Screener models.
//!
//! Types for Yahoo Finance predefined screener data (most actives, day gainers,
//! day losers, most shorted, growth stocks, undervalued stocks, and more).
//!
//! ## Predefined Screeners
//!
//! Use `finance::screener(Screener::DayGainers, 25)` for the 15 built-in
//! Yahoo Finance screeners.
//!
//! ## Custom Screeners
//!
//! Use [`EquityScreenerQuery`] or [`FundScreenerQuery`] with typed field enums to
//! build fully type-safe filters:
//!
//! ```no_run
//! use finance_query::{EquityField, EquityScreenerQuery, ScreenerFieldExt, finance};
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let query = EquityScreenerQuery::new()
//!     .size(25)
//!     .add_condition(EquityField::Region.eq_str("us"))
//!     .add_condition(EquityField::AvgDailyVol3M.gt(200_000.0))
//!     .add_condition(EquityField::PeRatio.between(10.0, 25.0));
//!
//! let results = finance::custom_screener(query).await?;
//! # Ok(())
//! # }
//! ```

pub mod condition;
pub mod fields;
mod query;
mod quote;
mod response;
mod values;

pub use condition::{
    ConditionValue, LogicalOperator, Operator, QueryCondition, QueryGroup, QueryOperand,
    ScreenerField, ScreenerFieldExt,
};
pub use fields::{EquityField, FundField};
pub use query::{EquityScreenerQuery, FundScreenerQuery, QuoteType, ScreenerQuery, SortType};
pub use quote::ScreenerQuote;
pub use response::ScreenerResults;
pub use values::{ScreenerFundCategory, ScreenerPeerGroup};
