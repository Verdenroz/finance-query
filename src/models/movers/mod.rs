//! Market movers models
//!
//! Types for market movers data (most actives, day gainers, day losers).

mod quote;
mod response;

pub use quote::MoverQuote;
pub use response::{MoversFinance, MoversResponse, MoversResult};
